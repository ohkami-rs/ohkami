#![cfg(feature="worker")]

mod meta;
mod durable;
mod binding;

use crate::util;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Error, Ident, ItemFn, ItemStruct, Fields, LitStr};


pub fn worker(args: TokenStream, ohkami_fn: TokenStream) -> Result<TokenStream, syn::Error> {
    let worker_meta: meta::WorkerMeta = syn::parse2(args)?;
    let ohkami_fn: ItemFn = syn::parse2(ohkami_fn)?;

    let gen_ohkami = {
        let name     = &ohkami_fn.sig.ident;
        let awaiting = ohkami_fn.sig.asyncness.is_some().then_some(quote! {
            .await
        });

        quote! {
            #name()#awaiting
        }
    };

    let openapi_fn = cfg!(feature="openapi").then_some({
        let title   = worker_meta.title;
        let version = worker_meta.version;
        let servers = worker_meta.servers.into_iter().map(|meta::Server {
            url, description, variables
        }| {
            let mut def = quote! {
                ::ohkami::openapi::Server::at(#url)
            };
            if let Some(description) = description {
                def.extend(quote! {
                    .description(#description)
                });
            }
            if let Some(variables) = variables {
                for (name, meta::ServerVariable { r#default, r#enum, .. }) in variables {
                    let candidates = r#enum.unwrap_or_else(Vec::new);
                    def.extend(quote! {
                        .var(#name, #r#default, [ #(#candidates),* ])
                    });
                }
            }
            def
        });

        quote! {
            const _: () = {
                // `#[wasm_bindgen]` direcly references this modules in epxpaned code
                use ::worker::{wasm_bindgen, wasm_bindgen_futures};

                #[doc(hidden)]
                #[::worker::wasm_bindgen::prelude::wasm_bindgen(js_name = "OpenAPIDocumentBytes")]
                pub async fn __openapi_document_bytes__() -> Vec<u8> {
                    let ohkami: ::ohkami::Ohkami = #gen_ohkami;
                    ohkami.__openapi_document_bytes__(::ohkami::openapi::OpenAPI {
                        title:   #title,
                        version: #version,
                        servers: &[ #(#servers),* ],
                    })
                }
            };
        }
    });

    Ok(quote! {
        #ohkami_fn

        #openapi_fn

        #[::worker::event(fetch)]
        async fn main(
            req: ::worker::Request,
            env: ::worker::Env,
            ctx: ::worker::Context,
        ) -> ::worker::Result<::worker::Response> {
            let ohkami: ::ohkami::Ohkami = #gen_ohkami;
            Ok(ohkami.__worker__(req, env, ctx).await)
        }
    })
}

pub fn bindings(env_name: TokenStream, bindings_struct: TokenStream) -> Result<TokenStream, syn::Error> {
    use self::binding::Binding;

    fn callsite(msg: impl std::fmt::Display) -> Error {
        Error::new(Span::call_site(), msg)
    }
    fn invalid_wrangler_toml() -> Error {
        Error::new(Span::call_site(), "Invalid wrangler.toml")
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////

    let wrangler_toml: toml::Value = {use std::io::Read;
        let mut file = util::find_a_file_in_maybe_workspace("wrangler.toml")
            .map_err(|e| callsite(e.to_string()))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|_| callsite("wrangler.toml found but it's not readable"))?;
        toml::from_str(&buf)
            .map_err(|_| callsite("Failed to read wrangler.toml"))?
    };

    let env: &toml::Table = {
        let top_level = wrangler_toml.as_table().ok_or_else(invalid_wrangler_toml)?;
        let env_name: Option<Ident> = (!env_name.is_empty()).then(|| syn::parse2(env_name)).transpose()?;
        match env_name {
            None      => top_level,
            Some(env) => top_level.get("env")
                .and_then(|e| e.as_table())
                .and_then(|t| t.get(&env.to_string()))
                .and_then(|e| e.as_table())
                .ok_or_else(|| callsite(format!("env `{env}` is not found in wrangler.toml")))?
        }
    };

    let bindings: Vec<(Ident, Binding)> = Binding::collect_from_env(&env)?;

    let bindings_struct: ItemStruct = syn::parse2(bindings_struct)?; {
        if !bindings_struct.generics.params.is_empty() {
            return Err(Error::new(
                bindings_struct.generics.params.span(),
                "`#[bindings]` doesn't support generics"
            ))
        }
    }
    let vis  = &bindings_struct.vis;
    let name = &bindings_struct.ident;

    let named_fields = match &bindings_struct.fields {
        Fields::Unit => None,
        Fields::Named(n) => Some(n.named
            .iter()
            .map(|field| (
                field.ident.as_ref().unwrap(),
                util::extract_doc_comment(&field.attrs)
            ))
            .collect::<Vec<_>>()
        ),
        Fields::Unnamed(u) => return Err(Error::new(
            u.span(),
            "`#[bindings]` doesn't support unnamed fields"
        )),
    };

    let declare_struct = match &named_fields {
        Some(n) => {
            let mut var_field_indexes = Vec::with_capacity(n.len());
            for (i, (field_name, _)) in n.iter().enumerate() {
                let binding_type = bindings.iter()
                    .find_map(|(name, b)| (name == *field_name).then_some(b))
                    .ok_or_else(|| syn::Error::new(
                        field_name.span(),
                        format!("No binding named `{field_name}` found")
                    ))?;
                if matches!(binding_type, Binding::Variable(_)) {
                    var_field_indexes.push(i);
                }
            }

            let mut bindings_struct = bindings_struct.clone();
            for i in var_field_indexes {
                let Fields::Named(n) = &mut bindings_struct.fields else {unreachable!()};
                n.named.get_mut(i).unwrap().attrs.push(syn::Attribute {
                    pound_token: Default::default(),
                    style: syn::AttrStyle::Outer,
                    bracket_token: Default::default(),
                    meta: syn::parse_str("allow(unused)")?
                });
            }

            quote! {
                #[allow(non_snake_case)]
                #bindings_struct
            }
        }
        None => {
            let fields = bindings.iter().map(|(name, binding)| {
                let ty = binding.tokens_ty();
                quote! {
                    #vis #name: #ty
                }
            });
            quote! {
                #[allow(non_snake_case)]
                #vis struct #name {
                    #( #fields ),*
                }
            }
        }
    };

    let const_vars = {
        let consts = bindings.iter()
            .filter_map(|(name, binding)|
                match binding {
                    Binding::Variable(value) => Some((name, value)),
                    _ => None
                }
            )
            .filter_map(|(name, value)| match &named_fields {
                None => Some((name, value, None)),
                Some(n) => n.iter().find_map(|(field_name, doc)|
                    (name == *field_name).then_some((name, value, doc.as_ref()))
                )
            })
            .map(|(name, value, doc)| {
                let value = LitStr::new(&value, Span::call_site());
                let doc = doc.as_ref()
                    .map(|d| {
                        let d = LitStr::new(d, Span::call_site());
                        quote! { #[doc = #d] }
                    });
                quote! {
                    #doc
                    #vis const #name: &'static str = #value;
                }
            });

        quote! {
            #[allow(non_upper_case_globals)]
            impl #name {
                #( #consts )*
            }
        }
    };

    let impl_new = {
        let extract = bindings.iter()
            .filter(|(name, _)| match &named_fields {
                None => true,
                Some(n) => n.iter().any(|(field_name, _)| name == *field_name)
            })
            .map(|(name, binding)| {
                binding.tokens_extract_from_env(name)
            });

        quote! {
            impl #name {
                #[allow(unused)]
                #vis fn new(env: &::worker::Env) -> ::worker::Result<Self> {
                    Ok(Self { #( #extract ),* })
                }
            }
        }
    };

    let impl_from_request = {
        quote! {
            impl<'req> ::ohkami::FromRequest<'req> for #name {
                type Error = ::ohkami::Response;
                fn from_request(
                    req: &'req ::ohkami::Request
                ) -> ::std::option::Option<::std::result::Result<Self, Self::Error>> {
                    ::std::option::Option::Some(
                        Self::new(req.context.env())
                            .map_err(|e| {
                                ::worker::console_error!("FromRequest failed: {e}");
                                e.into()
                            })
                    )
                }
            }
        }
    };

    let impl_send_sync = if
        bindings.is_empty() || named_fields.is_some_and(|n| n.is_empty())
    {
        None
    } else {
        Some(quote! {
            unsafe impl ::std::marker::Send for #name {}
            unsafe impl ::std::marker::Sync for #name {}
        })
    };

    Ok(quote! {
        #declare_struct
        #const_vars
        #impl_new
        #impl_from_request
        #impl_send_sync
    })
}

#[allow(non_snake_case)]
pub fn DurableObject(args: TokenStream, object: TokenStream) -> Result<TokenStream, syn::Error> {
    use self::durable::{DurableObjectType, bindgen_methods};

    let durable_object_type = (!args.is_empty())
        .then(|| syn::parse2::<DurableObjectType>(args))
        .transpose()?;
    
    let object = syn::parse2::<ItemStruct>(object)?;

    let methods = match durable_object_type {
        // if not specified, bindgen all.
        None => vec![
            bindgen_methods::core(),
            bindgen_methods::alarm(),
            bindgen_methods::websocket(),
        ],

        // if specified, bindgen only related methods.
        Some(DurableObjectType::Fetch) => vec![
            bindgen_methods::core(),
        ],
        Some(DurableObjectType::Alarm) => vec![
            bindgen_methods::core(),
            bindgen_methods::alarm(),
        ],
        Some(DurableObjectType::WebSocket) => vec![
            bindgen_methods::core(),
            bindgen_methods::websocket(),
        ],
    };

    let name = &object.ident;
    Ok(quote! {
        #object

        impl ::ohkami::has_DurableObject_attribute for #name {}

        const _: () = {
            // `#[wasm_bindgen]` attribute fully uses this module
            use ::worker::wasm_bindgen;

            #[::worker::wasm_bindgen::prelude::wasm_bindgen]
            #[::ohkami::__internal__::consume_struct]
            #object

            #[::worker::wasm_bindgen::prelude::wasm_bindgen]
            impl #name {
                #(#methods)*
            }
        };
    })
}
