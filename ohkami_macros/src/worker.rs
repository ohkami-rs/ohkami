use proc_macro2::{Span, TokenStream};
use syn::{spanned::Spanned, Error, Ident, ItemFn, ItemStruct, Result};
use quote::quote;


pub fn worker(ohkami_fn: TokenStream) -> Result<TokenStream> {
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

    Ok(quote! {
        #ohkami_fn

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

    let wrangler_toml: toml::Value = {use std::{io::Read, fs::File};
        let mut file = File::open("wrangler.toml").or_else(|_| {
            /* workspace mode */

            let cargo_toml = {
                let mut file = File::open("Cargo.toml").map_err(|_| callsite("Cargo.toml is not found"))?;
                let mut buf = String::new();
                file.read_to_string(&mut buf).map_err(|_| callsite("wrangler.toml found but it's not readable"))?;
                toml::from_str::<toml::Value>(&buf).map_err(|_| callsite("Failed to read wrangler.toml"))?
            };

            let Some(workspace) = cargo_toml.as_table().unwrap().get("workspace") else {
                return Err(callsite("Unexpectedly no `workspace` found in project root Cargo.toml"))
            };
            let members = workspace
                .as_table().ok_or_else(|| callsite("Invalid Cargo.toml"))?
                .get("members").ok_or_else(|| callsite("No `members` found in `[workspace]`"))?
                .as_array().ok_or_else(|| callsite("Invalid `workspace.members`"))?;

            let mut wrangler_tomls = Vec::with_capacity(1);
            for member in members {
                let path = member.as_str().ok_or_else(|| callsite("Invalid member"))?;
                if let Ok(wrangler_toml) = File::open(std::path::PathBuf::from_iter([path, "wrangler.toml"])) {
                    wrangler_tomls.push(wrangler_toml)
                }
            }

            (wrangler_tomls.len() == 1)
                .then_some(wrangler_tomls.pop().unwrap())
                .ok_or_else(|| callsite("More than one workspace members have wrangler.toml, which is not supported"))
        }).map_err(|_| callsite("\
            `wrangler.toml` doesn't exists or isn't readable. \
            Or, if you call me in a workspace, maybe more then one members have \
            `wrangler.toml`s and it's not supported. \
        "))?;

        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|_| callsite("wrangler.toml found but it's not readable"))?;
        toml::from_str(&buf).map_err(|_| callsite("Failed to read wrangler.toml"))?
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
        D1,
        KV,
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
                Binding::D1            => quote!(::worker::d1::D1Database),
                Binding::KV            => quote!(::worker::kv::KvStore),
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

            let get = match binding {
                Binding::Variable(value) => quote!(#value),
                Binding::D1 => quote! {
                    match req.env().d1(#name_str) {
                        Ok(binding) => binding, Err(e) => {::worker::console_error!("{e}");
                            return ::std::option::Option::Some(::std::result::Result::Err(::ohkami::Response::InternalServerError()))
                        }
                    }
                },
                Binding::KV => quote! {
                    match req.env().kv(#name_str) {
                        Ok(binding) => binding, Err(e) => {::worker::console_error!("{e}");
                            return ::std::option::Option::Some(::std::result::Result::Err(::ohkami::Response::InternalServerError()))
                        }
                    }
                },
                Binding::Queue => quote! {
                    match req.env().queue(#name_str) {
                        Ok(binding) => binding, Err(e) => {::worker::console_error!("{e}");
                            return ::std::option::Option::Some(::std::result::Result::Err(::ohkami::Response::InternalServerError()))
                        }
                    }
                },
                Binding::Service => quote! {
                    match req.env().service(#name_str) {
                        Ok(binding) => binding, Err(e) => {::worker::console_error!("{e}");
                            return ::std::option::Option::Some(::std::result::Result::Err(::ohkami::Response::InternalServerError()))
                        }
                    }
                },
                Binding::DurableObject => quote! {
                    match req.env().durable_object(#name_str) {
                        Ok(binding) => binding, Err(e) => {::worker::console_error!("{e}");
                            return ::std::option::Option::Some(::std::result::Result::Err(::ohkami::Response::InternalServerError()))
                        }
                    }
                },
            };

            quote! {
                #name: #get
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
