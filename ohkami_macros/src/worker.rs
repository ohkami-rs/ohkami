use proc_macro2::TokenStream;
use syn::{ItemFn, Result};
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
