use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemStruct};

// #[allow(non_snake_case)]
// pub(super) fn JSON(struct_stream: TokenStream) -> Result<TokenStream, Error> {
// 
//     #[allow(unused)] //
// 
//     let ItemStruct { ident, generics, .. }
//         = syn::parse2(struct_stream.clone())?;
// 
//     Ok(quote!{
//         impl<'j> ohkami::components::json::Json<'j> for #ident {}
// 
//         #[derive(serde::Serialize, serde::Deserialize)]
//         #struct_stream
//     })
// }

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
