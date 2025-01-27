#![cfg(feature="worker")]

mod meta;

use crate::util;
use proc_macro2::{Span, TokenStream};
use syn::{spanned::Spanned, Error, Ident, ItemFn, ItemStruct, Result};
use quote::quote;


pub fn worker(args: TokenStream, ohkami_fn: TokenStream) -> Result<TokenStream> {
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
        let servers = worker_meta.servers.into_iter()
            .map(|meta::Server { url, description, variables }| {
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
                        servers: vec![ #(#servers),* ],
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

pub fn bindings(env: TokenStream, bindings_struct: TokenStream) -> Result<TokenStream> {
    fn callsite(msg: impl std::fmt::Display) -> Error {
        Error::new(Span::call_site(), msg)
    }
    fn invalid_wrangler_toml() -> Error {
        Error::new(Span::call_site(), "Invalid wrangler.toml")
    }

    let env: Option<Ident> = (!env.is_empty()).then(|| syn::parse2(env)).transpose()?;

    let bindings_struct: ItemStruct = syn::parse2(bindings_struct)?; {
        if !bindings_struct.generics.params.is_empty() {
            return Err(Error::new(
                bindings_struct.generics.params.span(),
                "`#[bindings]` doesn't support generics"
            ))
        }
        if !bindings_struct.fields.is_empty() {
            return Err(Error::new(
                bindings_struct.span(),
                "`#[bindings]` doesn't support input structs with fields. \
                Use unit struct like `struct Bindings;`."
            ))
        }
    }

    let wrangler_toml: toml::Value = {use std::io::Read;
        let mut file = util::find_a_file_in_maybe_workspace("wrangler.toml")
            .map_err(|e| callsite(e.to_string()))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|_| callsite("wrangler.toml found but it's not readable"))?;
        toml::from_str(&buf)
            .map_err(|_| callsite("Failed to read wrangler.toml"))?
    };

    let config: &toml::Table = {
        let top_level = wrangler_toml.as_table().ok_or_else(invalid_wrangler_toml)?;
        match env {
            None      => top_level,
            Some(env) => top_level
                .get("env").ok_or_else(|| callsite(format!("env `{env}` is not found in wrangler.toml")))?
                .as_table().ok_or_else(invalid_wrangler_toml)?
                .get(&env.to_string()).ok_or_else(|| callsite(format!("env `{env}` is not found in wrangler.toml")))?
                .as_table().ok_or_else(invalid_wrangler_toml)?
        }
    };

    enum Binding {
        Variable(String),
        AI,
        D1,
        KV,
        R2,
        Service,
        Queue,
        DurableObject,
    }

    let name = &bindings_struct.ident;
    let vis  = &bindings_struct.vis;

    let bindings: Vec<(Ident, Binding)> = {
        let mut bindings = Vec::new();

        if let Some(toml::Value::Table(vars)) = config.get("vars") {
            for (name, value) in vars {
                let value = value.as_str().ok_or_else(|| callsite("`#[bindings]` doesn't support JSON values in `vars` binding"))?;
                bindings.push((
                    syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                    Binding::Variable(value.into())
                ))
            }
        }
        if let Some(toml::Value::Table(ai)) = config.get("ai") {
            let name = ai
                .get("binding").ok_or_else(|| callsite("Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`"))?
                .as_str().ok_or_else(invalid_wrangler_toml)?;
            bindings.push((
                syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                Binding::AI
            ))
        }
        if let Some(toml::Value::Array(d1_databases)) = config.get("d1_databases") {
            for binding in d1_databases {
                let name = binding.as_table().ok_or_else(invalid_wrangler_toml)?
                    .get("binding").ok_or_else(|| callsite("Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`"))?
                    .as_str().ok_or_else(invalid_wrangler_toml)?;
                bindings.push((
                    syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                    Binding::D1
                ))
            }
        }
        if let Some(toml::Value::Array(kv_namespaces)) = config.get("kv_namespaces") {
            for binding in kv_namespaces {
                let name = binding.as_table().ok_or_else(invalid_wrangler_toml)?
                    .get("binding").ok_or_else(|| callsite("Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`"))?
                    .as_str().ok_or_else(invalid_wrangler_toml)?;
                bindings.push((
                    syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                    Binding::KV
                ))
            }
        }
        if let Some(toml::Value::Array(r2_buckets)) = config.get("r2_buckets") {
            for binding in r2_buckets {
                let name = binding.as_table().ok_or_else(invalid_wrangler_toml)?
                    .get("binding").ok_or_else(|| callsite("Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`"))?
                    .as_str().ok_or_else(invalid_wrangler_toml)?;
                bindings.push((
                    syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                    Binding::R2
                ))
            }
        }
        if let Some(toml::Value::Array(services)) = config.get("services") {
            for binding in services {
                let name = binding.as_table().ok_or_else(invalid_wrangler_toml)?
                    .get("binding").ok_or_else(|| callsite("Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`"))?
                    .as_str().ok_or_else(invalid_wrangler_toml)?;
                bindings.push((
                    syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                    Binding::Service
                ))
            }
        }
        if let Some(toml::Value::Table(queues)) = config.get("queues") {
            if let Some(toml::Value::Array(producers)) = queues.get("producers") {
                for binding in producers {
                    let name = binding.as_table().ok_or_else(invalid_wrangler_toml)?
                        .get("binding").ok_or_else(|| callsite("Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`"))?
                        .as_str().ok_or_else(invalid_wrangler_toml)?;
                    bindings.push((
                        syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                        Binding::Queue
                    ))
                }
            }
        }
        if let Some(toml::Value::Table(durable_objects)) = config.get("durable_objects") {
            if let Some(toml::Value::Array(durable_object_bindings)) = durable_objects.get("bindings") {
                for binding in durable_object_bindings {
                    let name = binding.as_table().ok_or_else(invalid_wrangler_toml)?
                        .get("name").ok_or_else(|| callsite("Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`"))?
                        .as_str().ok_or_else(invalid_wrangler_toml)?;
                    bindings.push((
                        syn::parse_str(name).map_err(|e| callsite(format!("Can't bind binding `{name}` into struct: {e}")))?,
                        Binding::DurableObject
                    ))
                }
            }
        }

        bindings
    };

    let declare_struct = {
        let fields = bindings.iter().map(|(name, binding)| {
            let ty = match binding {
                Binding::Variable(_)   => quote!(&'static str),
                Binding::AI            => quote!(::worker::Ai),
                Binding::D1            => quote!(::worker::d1::D1Database),
                Binding::KV            => quote!(::worker::kv::KvStore),
                Binding::R2            => quote!(::worker::Bucket),
                Binding::Queue         => quote!(::worker::Queue),
                Binding::Service       => quote!(::worker::Fetcher),
                Binding::DurableObject => quote!(::worker::ObjectNamespace),
            };

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
    };

    let impl_bindings = {
        let methods = bindings.iter()
            .filter_map(|(name, binding)| match binding {
                Binding::Variable(var) => Some(quote! {
                    #vis const #name: &'static str = #var;
                }),
                _ => None
            });

        quote! {
            #[allow(non_snake_case)]
            impl #name {
                #( #methods )*
            }
        }
    };

    let impl_from_request = {
        let extract = bindings.iter().map(|(name, binding)| {
            let name_str = name.to_string();

            let from_env = |get: TokenStream| quote! {
                #name: match req.env().#get {
                    Ok(binding) => binding,
                    Err(e) => {
                        ::worker::console_error!("{e}");
                        return ::std::option::Option::Some(::std::result::Result::Err(::ohkami::Response::InternalServerError()));
                    }
                }
            };

            match binding {
                Binding::Variable(value) => quote! { #name: #value },
                Binding::AI              => from_env(quote! { ai(#name_str) }),
                Binding::D1              => from_env(quote! { d1(#name_str) }),
                Binding::KV              => from_env(quote! { kv(#name_str) }),
                Binding::R2              => from_env(quote! { bucket(#name_str) }),
                Binding::Queue           => from_env(quote! { queue(#name_str) }),
                Binding::Service         => from_env(quote! { service(#name_str) }),
                Binding::DurableObject   => from_env(quote! { durable_object(#name_str) }),
            }
        });

        quote! {
            impl<'req> ::ohkami::FromRequest<'req> for #name {
                type Error = ::ohkami::Response;
                fn from_request(
                    req: &'req ::ohkami::Request
                ) -> ::std::option::Option<::std::result::Result<Self, Self::Error>> {
                    ::std::option::Option::Some(::std::result::Result::Ok(
                        Self {
                            #( #extract ),*
                        }
                    ))
                }
            }
        }
    };

    let impl_send_sync = (!bindings.is_empty()).then_some(
        quote! {
            unsafe impl ::std::marker::Send for #name {}
            unsafe impl ::std::marker::Sync for #name {}
        }
    );

    Ok(quote! {
        #declare_struct
        #impl_bindings
        #impl_from_request
        #impl_send_sync
    })
}
