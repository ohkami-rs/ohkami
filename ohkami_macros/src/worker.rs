use proc_macro2::{Span, TokenStream};
use syn::{spanned::Spanned, Error, Ident, ItemFn, ItemStruct, Result};
use quote::{quote, ToTokens};


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

    let bindings_struct: ItemStruct = syn::parse2(bindings_struct)?;
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

    let wrangler_toml = {use std::io::Read;
        let mut file = std::fs::File::open("wrangler.toml")
            .map_err(|_| callsite("wrangler.toml doesn't exists or isn't readable"))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|_| callsite("wrangler.toml found but it's not readable"))?;
        buf
    };
    let wrangler_toml: toml::Value = toml::from_str(&wrangler_toml)
        .map_err(|_| callsite("Failed to read wrangler.toml"))?;

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

    // [TODO]
    // - get struct name
    // - check the struct has no generics
    // - read wranagler.toml with toml crate
    // - bind bindings in wrangler.toml to the struct fields

    Ok(quote! {

    })
}
