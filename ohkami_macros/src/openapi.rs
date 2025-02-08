#![cfg(feature="openapi")]

mod attributes;

use self::attributes::{ContainerAttributes, FieldAttributes, VariantAttributes};
use crate::util::{inner_Option, extract_doc_comment, extract_doc_attrs};
use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{Item, ItemFn, ItemStruct, ItemEnum, Fields, FieldsNamed, FieldsUnnamed, Visibility, Ident, LitInt, LitStr, token, Token};

pub(super) fn derive_schema(input: TokenStream) -> syn::Result<TokenStream> {
    return match syn::parse2::<Item>(input)? {
        Item::Struct(s) => derive_schema_for_struct(s),
        Item::Enum  (e) => derive_schema_for_enum  (e),
        _ => Err(syn::Error::new(Span::call_site(), "#[derive(Schema)] takes struct or enum"))
    };

    fn derive_schema_for_struct(s: ItemStruct) -> syn::Result<TokenStream> {
        let name = &s.ident;
        let (impl_generics, ty_generics, where_clause) = s.generics.split_for_impl();

        let container_attrs = ContainerAttributes::new(&s.attrs)?;

        let mut struct_schema = schema_of_fields({
            let mut fields = s.fields;
            if let (
                Fields::Named(FieldsNamed { brace_token:_, named }),
                Some((span, case))
            ) = (
                &mut fields,
                container_attrs.serde.rename_all_fields.value()?
            ) {
                for f in named {
                    f.ident = Some(Ident::new(
                        &case.apply_to_field(&f.ident.as_ref().unwrap(/* Named */).to_string()),
                        span
                    ));
                }
            }
            fields
        }, &container_attrs)?;

        if container_attrs.openapi.component.yes {
            struct_schema = {
                let mut component_name = LitStr::new(
                    container_attrs.openapi.component.name.as_ref().unwrap_or(&name.to_string()),
                    name.span()
                );
                if let Some((span, rename)) = container_attrs.serde.rename.value()? {
                    component_name = LitStr::new(&rename, span);
                }
                quote! {
                    ::ohkami::openapi::component(#component_name, #struct_schema)
                }
            };
        }

        if let Some(description) = extract_doc_comment(&s.attrs) {
            struct_schema = {
                let description = LitStr::new(&description, Span::call_site());
                quote! {
                    #struct_schema.description(#description)
                }
            };
        }

        Ok(quote! {
            impl #impl_generics ::ohkami::openapi::Schema for #name #ty_generics
            #where_clause
            {
                fn schema() -> impl Into<::ohkami::openapi::schema::SchemaRef> {
                    #struct_schema
                }
            }
        })
    }

    fn derive_schema_for_enum(e: ItemEnum) -> syn::Result<TokenStream> {
        let name = &e.ident;
        let (impl_generics, ty_generics, where_clause) = e.generics.split_for_impl();

        let container_attrs = ContainerAttributes::new(&e.attrs)?;

        let mut variant_schemas = Vec::with_capacity(e.variants.len());
        for v in e.variants {
            let variant_attrs = VariantAttributes::new(&v.attrs)?;

            if variant_attrs.serde.skip
            || variant_attrs.serde.skip_serializing
            || variant_attrs.serde.skip_deserializing
            || variant_attrs.serde.skip_serializing_if.is_some()
            {
                continue
            }

            let mut schema = schema_of_fields({
                let mut fields = v.fields;
                if let (
                    Fields::Named(FieldsNamed { brace_token:_, named }),
                    Some((span, case))
                ) = (
                    &mut fields,
                    container_attrs.serde.rename_all_fields.value()?
                ) {
                    for f in named {
                        f.ident = Some(Ident::new(
                            &case.apply_to_field(&f.ident.as_ref().unwrap(/* Named */).to_string()),
                            span
                        ));
                    }
                }
                fields
            }, &container_attrs)?;
            
            let tag = {
                let mut ident = v.ident;
                if let Some((span, case)) = variant_attrs.serde.rename_all.value()? {
                    ident = Ident::new(&case.apply_to_variant(&ident.to_string()), span);
                }
                if let Some((span, name)) = variant_attrs.serde.rename.value()? {
                    ident = Ident::new(&*name, span);
                }
                LitStr::new(&ident.to_string(), ident.span())
            };

            schema = match (
                &*container_attrs.serde.tag,
                &*container_attrs.serde.content,
                container_attrs.serde.untagged
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
                    let t = LitStr::new(t, Span::call_site());
                    quote! {
                        #schema
                            .property(#t, #tag)
                    }
                }

                (Some(t), Some(c), _) => {/* Adjacently tagged */
                    let t = LitStr::new(t, Span::call_site());
                    let c = LitStr::new(c, Span::call_site());
                    quote! {
                        ::ohkami::openapi::object()
                            .property(#t, #tag)
                            .property(#c, #schema)
                    }

                }
            };

            if let Some(description) = extract_doc_comment(&v.attrs) {
                schema = {
                    let description = LitStr::new(&description, Span::call_site());
                    quote! {
                        #schema.description(#description)
                    }
                };
            }

            variant_schemas.push(schema)
        }

        let mut enum_schema = quote! {
            ::ohkami::openapi::oneOf(
                ( #(#variant_schemas,)* )
            )
        };

        if container_attrs.openapi.component.yes {
            enum_schema =  {
                let mut component_name = LitStr::new(
                    container_attrs.openapi.component.name.as_ref().unwrap_or(&name.to_string()),
                    name.span()
                );
                if let Some((span, rename)) = container_attrs.serde.rename.value()? {
                    component_name = LitStr::new(&rename, span);
                }
                quote! {
                    ::ohkami::openapi::component(#component_name, #enum_schema)
                }
            };
        }

        if let Some(description) = extract_doc_comment(&e.attrs) {
            enum_schema = {
                let description = LitStr::new(&*description, Span::call_site());
                quote! {
                    #enum_schema.description(#description)
                }
            };
        }

        Ok(quote! {
            impl #impl_generics ::ohkami::openapi::Schema for #name #ty_generics
            #where_clause
            {
                fn schema() -> impl Into<::ohkami::openapi::schema::SchemaRef> {
                    #enum_schema
                }
            }
        })
    }

    fn schema_of_fields(fields: Fields, container_attrs: &ContainerAttributes) -> syn::Result<TokenStream> {
        match fields {
            Fields::Named(FieldsNamed { brace_token:_, named }) => {/* object */
                let mut properties = Vec::with_capacity(named.len());
                for f in named {
                    let field_attrs = FieldAttributes::new(&f.attrs)?;

                    if field_attrs.serde.skip
                    || field_attrs.serde.skip_serializing
                    || field_attrs.serde.skip_deserializing
                    {
                        continue
                    }

                    let mut ident = f.ident.clone().unwrap(/* Named */);
                    if let Some((span, case)) = container_attrs.serde.rename_all.value()? {
                        ident = Ident::new(&case.apply_to_field(&ident.to_string()), span);
                    }
                    if let Some((span, rename)) = field_attrs.serde.rename.value()? {
                        ident = Ident::new(&rename, span);
                    }

                    let ty = &f.ty;
                    let inner_option = inner_Option(ty);

                    let is_optional_field = inner_option.is_some()
                        || field_attrs.serde.default
                        || field_attrs.serde.skip_serializing_if.is_some();

                    let mut property_schema = {
                        if let Some(inner_option) = inner_option {quote! {
                            ::ohkami::openapi::schema::Schema::<::ohkami::openapi::schema::Type::any>::from(
                                <#inner_option as ::ohkami::openapi::Schema>::schema()
                                    .into(/* SchemaRef */).into_inline().unwrap()
                            )
                        }} else {quote! {
                            ::ohkami::openapi::schema::Schema::<::ohkami::openapi::schema::Type::any>::from(
                                <#ty as ::ohkami::openapi::Schema>::schema()
                                    .into(/* SchemaRef */).into_inline().unwrap()
                            )
                        }}
                    };

                    if let Some(description) = extract_doc_comment(&f.attrs) {
                        property_schema = {
                            let description = LitStr::new(&description, Span::call_site());
                            quote! {
                                #property_schema.description(#description)
                            }
                        };
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
                        let property_name = LitStr::new(&ident.to_string(), ident.span());

                        properties.push(if is_optional_field {quote! {
                            schema = schema.optional(#property_name, #property_schema);
                        }} else {quote! {
                            schema = schema.property(#property_name, #property_schema);
                        }});
                    }
                }

                Ok(quote! {
                    {
                        let mut schema = ::ohkami::openapi::object();
                        #(#properties)*
                        schema
                    }
                })
            }

            Fields::Unnamed(FieldsUnnamed { paren_token:_, unnamed }) if unnamed.len() == 1 => {/* newtype */
                let f = unnamed.into_iter().next().unwrap(/* unnamed.len() == 1 */);

                let ty = &f.ty;

                let mut schema = quote! {
                    ::ohkami::openapi::schema::Schema::<::ohkami::openapi::schema::Type::any>::from(
                        <#ty as ::ohkami::openapi::Schema>::schema()
                            .into(/* SchemaRef */).into_inline().unwrap()
                    )
                };

                if let Some(description) = extract_doc_comment(&f.attrs) {
                    schema = {
                        let description = LitStr::new(&description, Span::call_site());
                        quote! {
                            #schema.description(#description)
                        }
                    };
                }

                Ok(schema)
            }

            Fields::Unnamed(FieldsUnnamed { paren_token:_, unnamed }) if unnamed.len() == 0 => {/* empty */
                Ok(quote! {
                    ::ohkami::openapi::object()
                })
            }
            Fields::Unit => {/* empty */
                Ok(quote! {
                    ::ohkami::openapi::object()
                })
            }

            Fields::Unnamed(FieldsUnnamed { paren_token:_, unnamed }) => {assert!(unnamed.len() >= 2);/* array of oneOf */
                let mut type_schemas = Vec::with_capacity(unnamed.len());
                for u in unnamed {
                    let field_attrs = FieldAttributes::new(&u.attrs)?;

                    if field_attrs.serde.skip
                    || field_attrs.serde.skip_serializing
                    || field_attrs.serde.skip_deserializing
                    {
                        continue
                    }

                    let ty = match inner_Option(&u.ty) {
                        Some(inner_option) => inner_option,
                        None => u.ty.clone()
                    };

                    let mut schema = quote! {
                        ::ohkami::openapi::schema::Schema::<::ohkami::openapi::schema::Type::any>::from(
                            <#ty as ::ohkami::openapi::Schema>::schema()
                                .into(/* SchemaRef */).into_inline().unwrap()
                        )
                    };

                    if let Some(description) = extract_doc_comment(&u.attrs) {
                        schema = {
                            let description = LitStr::new(&description, Span::call_site());
                            quote! {
                                #schema.description(#description)
                            }
                        };
                    }

                    type_schemas.push(schema)
                }

                Ok(quote! {
                    ::ohkami::openapi::array(::ohkami::openapi::oneOf(
                        (#(#type_schemas,)*)
                    ))
                })
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
            let key = if input.peek(override_keyword::summary) {
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
                    input.parse::<TokenStream>()?
                )))
            };

            input.parse::<Token![:]>()?;

            let value = input.parse::<LitStr>()?.value();

            Ok(Self { key, value })
        }
    }

    impl syn::parse::Parse for OperationMeta {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            #[allow(non_snake_case)]
            let operationId = input.peek(Ident)
                .then(|| input.parse::<Ident>().map(|i| i.to_string()))
                .transpose()?;

            let descriptions = input.peek(token::Brace)
                .then(|| {
                    let descriptions; syn::braced!(descriptions in input);
                    descriptions
                        .parse_terminated(DescriptionOverride::parse, Token![,])
                        .map(|punctuated| punctuated.into_iter().collect::<Vec<_>>())
                })
                .transpose()?
                .unwrap_or_default();


            Ok(Self { operationId, descriptions })
        }
    }

    //////////////////////////////////////////////////////////////

    let meta = syn::parse2::<OperationMeta>(meta)?;

    let handler = syn::parse2::<ItemFn>(handler)?;

    let handler_vis  = &handler.vis;
    let handler_name = &handler.sig.ident;

    // for generated struct
    let doc_attrs_copy = extract_doc_attrs(&handler.attrs);

    let handler = {
        let mut handler = handler.clone();
        handler.vis = Visibility::Public(Default::default());
        handler
    };

    let modify_op = {
        let mut modify_op = TokenStream::new();

        let operation_id = match meta.operationId {
            Some(operation_id) => LitStr::new(&operation_id, Span::call_site()),
            None => LitStr::new(&handler.sig.ident.to_string(), handler.sig.ident.span())
        };
        modify_op.extend(quote! {
            op = op.operationId(#operation_id);
        });

        if let Some(description) = extract_doc_comment(&handler.attrs) {
            modify_op.extend(quote! {
                op = op.description(#description);
            });
        }

        for DescriptionOverride { key, value } in meta.descriptions {
            modify_op.extend(match key {
                DescriptionTarget::Summary => {
                    quote! {
                        op = op.summary(#value);
                    }
                },
                DescriptionTarget::RequestBody => {
                    quote! {
                        op = op.requestBody_description(#value);
                    }
                },
                DescriptionTarget::DefaultResponse => {
                    quote! {
                        op = op.response_description("default", #value);
                    }
                },
                DescriptionTarget::Response { status } => {
                    quote! {
                        op = op.response_description(#status, #value);
                    }
                },
                DescriptionTarget::Param { name } => {
                    let name = LitStr::new(&name, Span::call_site());
                    quote! {
                        op = op.param_description(#name, #value);
                    }
                },
            });
        }

        modify_op
    };

    Ok(quote! {
        #(#doc_attrs_copy)*
        #[allow(non_camel_case_types)]
        #handler_vis struct #handler_name;

        const _: () = {
            mod operation {
                use super::*;
                #handler
            }

            impl ::ohkami::handler::IntoHandler<#handler_name> for #handler_name {
                fn into_handler(self) -> ::ohkami::handler::Handler {
                    ::ohkami::handler::IntoHandler::into_handler(operation::#handler_name)
                        .map_openapi_operation(|mut op| { #modify_op op })
                }
            }
        };
    })
}
