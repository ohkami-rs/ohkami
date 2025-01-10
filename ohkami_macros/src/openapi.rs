#![cfg(feature="openapi")]

mod attributes;

use self::attribtues::{ContainerAttributes, FieldAttributes, VariantAttributes};
use crate::util::{is_Option, inner_Option};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemStruct, ItemEnum, Fields, FieldsNamed, FieldsUnnamed, Visibility, Ident, LitInt, LitStr, Meta, MetaNameValue, Expr, ExprLit, Lit, Attribute, token, Token};

pub(super) fn derive_schema(input: TokenStream) -> syn::Result<TokenStream> {
    return match syn::parse2::<Item>(input)? {
        Item::Struct(s) => derive_schema_for_struct(s),
        Item::Enum(e)   => derive_schema_for_enum(s),
        _ => Err(syn::Error::new(syn::Span::call_site(), "#[derive(Schema)] takes struct or enum"))
    };

    fn derive_schema_for_struct(s: ItemStruct) -> syn::Result<TokenStream> {
        let name = &s.ident;
        let (impl_generics, ty_generics, where_clause) = s.generics.split_for_impl();

        let container_attrs = ContainerAttributes::new(&s.attrs);

        let mut schema = match &s.fields {
            Fields::Named(fields) => {
                let mut properties = Vec::with_capacity(fields.len());
                for f in fields {
                    let field_attrs = FieldAttributes::new(&f.attrs);

                    if field_attrs.skip {
                        continue
                    }

                    let mut ident = f.ident.clone().unwrap(/* Named */);
                    if let Some((span, case)) = container_attrs.rename_all.value()? {
                        ident = Ident::new(&case.apply_to_field(&f.ident.to_string()), span);
                    }
                    if let Some((span, ident)) = field_attrs.rename.value()? {
                        ident = Ident::new(&*ident, span);
                    }

                    let ty = &f.ty;
                    let inner_option = inner_Option(ty);

                    let is_optional_field = inner_option.is_some()
                        || field_attrs.serde.default
                        || field_attrs.serde.skip
                        || field_attrs.serde.skip_serializing
                        || field_attrs.serde.skip_deserializing
                        || field_attrs.serde.skip_serializing_if.is_some();

                    let property_schema = {
                        if let Some(inner_option) = inner_option {quote! {
                            openapi::Schema::schema(#inner_option)
                        }} else {quote! {
                            openapi::Schema::schema(#ty)
                        }}
                    };

                    if field_attrs.flatten {
                        properties.push(quote! {
                            for (property_name, property_schema, required) in #property_schema.into_properties() {
                                if required {
                                    schema = schema.property(property_name, property_schema);
                                } else {
                                    schema = schema.optional(property_name, property_schema);
                                }
                            }
                        })
                    } else {
                        let property_name = LitStr::new(ident.span(), &ident.to_string());
                                           
                        properties.push(if is_optional_field {quote! {
                            schema = schema.optional(#property_name, #property_schema);
                        }} else {quote! {
                            schema = schema.property(#property_name, #property_schema);
                        }});
                    }
                }

                quote! {
                    let mut schema = openapi::object();
                    #(#properties)*
                }
            }

            Fields::Unnamed(fields) if fields.len() == 1 => {
                let [field] = fields.try_into().unwrap();
                let ty = &field.ty;

                quote! {
                    openapi::Schema::schema()
                }
            }

            Fields::Unnamed(fields) if fields.len() == 0 | Fields::Unit => {}

            Fields::Unnamed(fields) => {assert!(fields.len() >= 2);}
        };

        if container_attrs.component.yes {
            schema =  {
                let component_name = LitStr::new(
                    name.span(),
                    container_attrs.component.name.unwrap_or(&name.to_string())
                );
                quote! {
                    openapi::component(#component_name, #schema)
                }
            }
        }

        Ok(quote! {
            impl #impl_generics ::ohkami::openapi::Schema for #name #ty_generics
            #where_clause
            {
                fn schema() -> ::ohkami::schema::Schema {
                    use ::ohkami::openapi;
                    #schema
                }
            }
        })
    }
}

pub(super) fn operation(meta: TokenStream, handler: TokenStream) -> syn::Result<TokenStream> {
    #[allow(non_snake_case)]
    struct OperationMeta {
        operationId:  Option<String>,
        descriptions: Vec<DescriptionOverride>,
    }

    struct DescriptionOverride {
        key:   DescriptionTarget,
        value: String,
    }
    enum DescriptionTarget {
        Summary,
        RequestBody,
        DefaultResponse,
        Response { status: u16 },
        Param { name: String },
    }

    mod override_keyword {
        syn::custom_keyword!(summary);
        syn::custom_keyword!(requestBody);
    }

    impl syn::parse::Parse for DescriptionOverride {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let key = if false {
            } else if input.peek(override_keyword::summary) {
                input.parse::<override_keyword::summary>()?;
                DescriptionTarget::Summary

            } else if input.peek(override_keyword::requestBody) {
                input.parse::<override_keyword::requestBody>()?;
                DescriptionTarget::RequestBody

            } else if input.peek(Token![default]) {
                input.parse::<Token![default]>()?;
                DescriptionTarget::DefaultResponse

            } else if input.peek(LitInt) {
                let status = input.parse::<LitInt>()?.base10_parse()?;
                DescriptionTarget::Response { status }
                
            } else if input.peek(Ident) {
                let name = input.parse::<Ident>()?.to_string();
                DescriptionTarget::Param { name }

            } else {
                return Err(syn::Error::new(input.span(), format!("\
                    Unepected description key: `{}`. Expected one of\n\
                    - summary       (.summary)\n\
                    - requestBody   (.requestBody.description)\n\
                    - default       (.responses.default.description)\n\
                    - <status:int>  (.responses.<status>.description)\n\
                    - <param:ident> (.parameters.<param>.description)\n\
                ",
                    input.parse2::<TokenStream>()?
                )))
            };

            input.parse::<Token![:]>()?;

            let value = input.parse::<LitStr>()?.value();

            Ok(Self { key, value })
        }
    }

    impl syn::parse::Parse for OperationMeta {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let operationId = input.peek(Ident)
                .then(|| input.parse())
                .transpose()?;

            let descriptions = input.peek(token::Brace)
                .then(|| {
                    let descriptions; syn::braced!(descriptions in input);
                    descriptions
                        .parse_terminated(DescriptionOverride::parse, Token![,])
                        .map(|iter| iter.collect::<Vec<_>>())
                })
                .transpose()?
                .unwrap_or_default();


            Ok(Self { operationId, descriptions })
        }
    }

    //////////////////////////////////////////////////////////////

    let meta = syn::parse2::<OperationMeta>(meta)?;

    let handler = syn::parse2::<ItemFn>(handler)?;
    let handler_vis  = handler.vis;
    let handler_name = handler.ident;

    let doc_attrs = handler.attrs.iter()
        .filter(|a| matches!(a.meta,
            Meta::NameValue(MetaNameValue {
                path, ..
            } if path.get_ident().is_some_and(|i| i == "doc")
        )));
    
    let handler = {
        let mut handler = handler.clone();
        handler.vis = Visibility::Public(Token![pub]);
        handler
    };

    let modify_op = {
        let mut modify_op = TokenStream::new();

        let description = doc_attrs.cloned()
            .flat_map(|a| match a.meta {
                Meta::NameValue(MetaNameValue {
                    value: Expr::Lit(ExprLit { lit: Lit::Str(value), .. }), ..
                }) => Some(value.value()),
                _ => unreachable!("invalid `#[doc = /* value */]`")
            })
            .fold(String::new(), |mut description, doc| {
                let mut unescaped_doc = String::with_capacity(doc.len()); {
                    let mut chars = doc.chars().peekable();
                    while let Some(ch) = chars.next() {
                        if ch == '\\' && chars.peek().is_some_and(char::is_ascii_punctuation) {
                            /* do nothing to unescape the next charactor */
                        } else {
                            unescaped_doc.push(ch);
                        }
                    }
                }
                description + &unescaped_doc
            });

        if !description.is_empty() {
            modify_op.extend(quote! {
                op = op.description(#description);
            });
        }

        if let Some(operationId) = meta.operationId {
            modify_op.extend(quote! {
                op = op.operationId(#operationId);
            });
        }

        for DescriptionOverride { key, value } in meta.descriptions {
            modify_op.extend(match key {
                DescriptionTarget::Summary => quote! {
                    op = op.summary(#value);
                },
                DescriptionTarget::RequestBody => quote! {
                    op.override_requestBody_description(#value);
                },
                DescriptionTarget::DefaultResponse => quote! {
                    op.override_response_description("default", #value);
                },
                DescriptionTarget::Response { status: u16 } => quote! {
                    op.override_response_description(&#status.to_string(), #value);
                },
                DescriptionTarget::Param { name: String } => quote! {
                    op.override_param_description(#name, #value);
                },
            });
        }

        modify_op
    };

    Ok(quote! {
        #(#doc_attrs)*
        #[allow(non_camelcase_types)]
        #handler_vis struct #handler_name;

        const _: () = {
            mod operation {
                use super::*;
                #handler
            }

            impl ::ohkami::handler::IntoHandler<#handler_name> for #handler_name {
                fn into_handler(self) -> ::ohkami::handler::Handler {
                    ::ohkami::handler::IntoHandler::into_handler(operation::#handler_name)
                        .map_openapi_operation(|op| { #modify_op })
                }
            }
        };
    })
}
