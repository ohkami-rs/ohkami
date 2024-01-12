use proc_macro2::{TokenStream};
use quote::{quote};
use syn::{Result};


#[allow(non_snake_case)]
pub(super) fn Serialize(data: TokenStream) -> Result<TokenStream> {
    Ok(quote! {
        #[derive(::ohkami::__internal__::serde::Serialize)]
        #[serde(crate = "::ohkami::__internal__::serde")]
        #[::ohkami::__internal__::consume_struct]
        #data
    })
}

#[allow(non_snake_case)]
pub(super) fn Deserialize(data: TokenStream) -> Result<TokenStream> {
    Ok(quote! {
        #[derive(::ohkami::__internal__::serde::Deserialize)]
        #[serde(crate = "::ohkami::__internal__::serde")]
        #[::ohkami::__internal__::consume_struct]
        #data
    })
}
