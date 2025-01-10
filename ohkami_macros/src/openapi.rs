#![cfg(feature="openapi")]

mod attributes;

use self::attribtues::{ContainerAttributes, FieldAttributes, VariantAttributes};
use crate::util::{is_Option, inner_Option, extract_doc_comment};
use proc_macro2::{TokenStream, Span};
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

        let mut struct_schema = schema_of_fields({
            let mut fields = s.fields;
            if let Some((span, case)) = container_attrs.serde.rename_all.value() {
                for f in fields {
                    f.ident = Some(Ident::new(
                        span,
                        &case.apply_to_field(&f.ident.to_string())
                    ));
                }
            }
            fields
        }, &container_attrs);

        if container_attrs.openapi.component.yes {
            struct_schema = {
                let mut component_name = LitStr::new(
                    name.span(),
                    container_attrs.openapi.component.name.as_ref().unwrap_or(&name.to_string())
                );
                if let Some((span, rename)) = container_attrs.serde.rename.value()? {
                    component_name = LitStr::new(span, &rename);
                }
                quote! {
                    ::ohkami::openapi::component(#component_name, #struct_schema)
                }
            };
        }

        if let Some(description) = extract_doc_comment(&s.attrs) {
            struct_schema = {
                let description = LitStr::new(Span::call_site(), description);
                quote! {
                    #struct_schema.description(#description)
                }
            };
        }

        Ok(quote! {
            impl #impl_generics ::ohkami::openapi::Schema for #name #ty_generics
            #where_clause
            {
                fn schema() -> ::ohkami::schema::Schema {
                    #struct_schema
                }
            }
        })
    }

    fn derive_schema_for_enum(e: ItemEnum) -> syn::Result<TokenStream> {
        let name = &e.ident;
        let (impl_generics, ty_generics, where_clause) = e.generics.split_for_impl();

        let mut container_attrs = ContainerAttributes::new(&e.attrs);

        let variant_schemas = e.variants.into_iter().filter_map(|mut v| {
            let mut variant_attrs = VariantAttributes::new(&v.attrs);

            if variant_attrs.serde.skip
            || variant_attrs.serde.skip_serializing
            || variant_attrs.serde.skip_deserializing
            || variant_attrs.serde.skip_serializing_if.is_some()
            {
                return None
            }

            let mut schema = schema_of_fields({
                let mut fields = v.fields;
                if let Some((span, case)) = container_attrs.serde.rename_all_fields.value() {
                    for f in fields {
                        f.ident = Some(Ident::new(
                            span,
                            &case.apply_to_field(&f.ident.to_string())
                        ));
                    }
                }
                fields
            }, &container_attrs);
            
            if let Some((span, name)) = variant_attrs.serde.rename.value() {
                v.ident = Ident::new(
                    span,
                    &case.apply_to_variant(&v.ident.to_string())
                );
            }

            let tag = LitStr::new(v.ident.span(), &v.ident.to_string());

            Some(match (
                &*container_attrs.serde.tag,
                &*container_attrs.serde.content,
                untagged
            ) {
                (_, _, true) => {/* Untagged */
                    schema
                }

                (None, _, _) => {/* Externally tagged */
                    quote! {
                        ::ohkami::openapi::object()
                            .property(#tag, #schema)
                    }
                }

                (Some(t), None, _) => {/* Internally tagged */
                    let t = LitStr::new(Span::call_site(), t);
                    quote! {
                        #schema
                            .property(#t, #tag)
                    }
                }

                (Some(t), Some(c), _) => {/* Adjacently tagged */
                    let t = LitStr::new(Span::call_site(), t);
                    let c = LitStr::new(Span::call_site(), c);
                    quote! {
                        ::ohkami::openapi::object()
                            .property(#t, #tag)
                            .property(#c, #schema)
                    }

                }
            })
        });

        let mut enum_schema = quote! {
            ::ohkami::openapi::oneOf(
                ( #(#variant_schemas,)* )
            )
        };

        if container_attrs.openapi.component.yes {
            enum_schema =  {
                let mut component_name = LitStr::new(
                    name.span(),
                    container_attrs.openapi.component.name.as_ref().unwrap_or(&name.to_string())
                );
                if let Some((span, rename)) = container_attrs.serde.rename.value()? {
                    component_name = LitStr::new(span, &rename);
                }
                quote! {
                    ::ohkami::openapi::component(#component_name, #enum_schema)
                }
            };
        }

        if let Some(description) = extract_doc_comment(&e.attrs) {
            enum_schema = {
                let description = LitStr::new(Span::call_site(), description);
                quote! {
                    #enum_schema.description(#description)
                }
            };
        }

        Ok(quote! {
            impl #impl_generics ::ohkami::openapi::Schema for #name #ty_generics
            #where_clause
            {
                fn schema() -> ::ohkami::schema::Schema {
                    #schema
                }
            }
        })
    }

    fn schema_of_fields(fields: Fields, container_attrs: &ContainerAttributes) -> TokenStream {
        match fields {
            Fields::Named(fields) => {/* object */
                let mut properties = Vec::with_capacity(fields.len());
                for f in fields {
                    let field_attrs = FieldAttributes::new(&f.attrs);

                    if field_attrs.serde.skip {
                        continue
                    }

                    let mut ident = f.ident.clone().unwrap(/* Named */);
                    if let Some((span, case)) = container_attrs.rename_all.value()? {
                        ident = Ident::new(&case.apply_to_field(&f.ident.to_string()), span);
                    }
                    if let Some((span, ident)) = field_attrs.serde.rename.value()? {
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

                    let mut property_schema = {
                        if let Some(inner_option) = inner_option {quote! {
                            ::ohkami::openapi::Schema::schema(#inner_option)
                        }} else {quote! {
                            ::ohkami::openapi::Schema::schema(#ty)
                        }}
                    };
                    if let Some(description) = extract_doc_comment(&f.attrs) {
                        let description = LitStr::new(Span::call_site, &description);
                        property_schema.extend(quote! {
                            .description(#description)
                        })
                    }

                    if field_attrs.serde.flatten {
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
                    {
                        let mut schema = ::ohkami::openapi::object();
                        #(#properties)*
                        schema
                    }
                }
            }

            Fields::Unnamed(mut fields) if fields.len() == 1 => {/* newtype */
                let f = fields.pop().unwrap(/* fields.len() == 1 */);

                let ty = &f.ty;

                let mut schema = quote! {
                    ::ohkami::openapi::Schema::schema(#ty)
                };

                if let Some(description) = extract_doc_comment(&f.attrs) {
                    let description = LitStr::new(Span::call_site(), &description);
                    schema.extend(quote! {
                        .description(#description)
                    })
                }

                schema
            }

            Fields::Unit | Fields::Unnamed(fields) if fields.len() == 0 => {/* empty */
                quote! {
                    ::ohkami::openapi::object()
                }
            }

            Fields::Unnamed(fields) => {assert!(fields.len() >= 2);
                let type_schemas = fields.iter().map(|f| {
                    let ty = &f.ty;
                    let mut schema = quote! {
                        ::ohkami::openapi::Schema::schema(#ty)
                    };
                    if let Some(description) = extract_doc_comment(&f.attrs) {
                        let description = LitStr::new(Span::call_site(), &description);
                        schema.extend(quote! {
                            .description(#description)
                        })
                    }
                    schema
                });

                quote! {
                    ::ohkami::openapi::array(::ohkami::openapi::oneOf(
                        (#(#type_schemas,)*)
                    ))
                }
            }
        }
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

    let handler = {
        let mut handler = handler.clone();
        handler.vis = Visibility::Public(Token![pub]);
        handler
    };

    let modify_op = {
        let mut modify_op = TokenStream::new();

        if let Some(description) = extract_doc_comment(&handler.attrs) {
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
