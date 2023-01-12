use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemStruct};

trait Build {
    fn build(self) -> TokenStream;
}

pub(super) fn derive_json(serde_derived_struct: TokenStream) -> Result<TokenStream, Error> {
    #[allow(unused)] // generics ...

    let ItemStruct { ident, generics, .. }
        = syn::parse2(serde_derived_struct.clone())?;

    Ok(quote!{
        impl<'j> ohkami::components::json::Json<'j> for #ident {}

        #[derive(serde::Serialize, serde::Deserialize)]
        #[ohkami::macros::consume_struct]
        #serde_derived_struct
    })
}

pub(super) fn consume_struct(serde_derived_struct: TokenStream) -> Result<TokenStream, Error> {
    let _: ItemStruct = syn::parse2(serde_derived_struct)?;
    Ok(TokenStream::new())
}

mod json_str;
pub(super) fn json_str(content: TokenStream) -> Result<TokenStream, Error> {
    Ok(syn::parse2::<json_str::JsonStr>(content)?.build())
}