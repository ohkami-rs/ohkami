use proc_macro2::TokenStream;
use syn::{ItemFn, ItemStruct, Result};
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

pub fn bindings(bindings_struct: TokenStream) -> Result<TokenStream> {
    let bindings_struct: ItemStruct = syn::parse2(bindings_struct)?;

    [TODO]
    - get struct name
    - check the struct has no generics
    - read wranagler.toml with toml crate
    - bind bindings in wrangler.toml to the struct fields

    Ok(quote! {

    })
}
