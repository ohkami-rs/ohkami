use proc_macro2::TokenStream;
use syn::Error;
use quote::quote;



pub(super) fn derive_json(struct_stream: TokenStream) -> Result<TokenStream, Error> {
    let struct_stream = syn::parse2::<syn::ItemStruct>(struct_stream)?;
    let ident = struct_stream.ident;
    // let generics = struct_stream.generics;
    Ok(quote!{
        impl<'j> Json<'j> for #ident {
            fn ser(self) -> String {
                serde_json::to_string(&self).unwrap()
            }

            fn de(string: &str) -> Self {
                serde_json::from_str(&string).unwrap()
            }
        }
    })
}