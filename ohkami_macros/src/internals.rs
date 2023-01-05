use proc_macro2::TokenStream;
use syn::Error;
use quote::quote;


pub(super) fn derive_json(struct_stream: TokenStream) -> Result<TokenStream, Error> {
    let struct_stream = syn::parse2::<syn::ItemStruct>(struct_stream)?;
    let ident = struct_stream.ident;

    if struct_stream.semi_token.is_some() {
        Ok(quote!(
            impl JSON for #ident {
                fn serialize(&self) -> String {

                }

                fn _deserialize(&str) -> Self {

                }
            }
        ))
    } else {
        Ok(quote!{
            impl JSON for #ident {
                fn serialize(&self) -> String {
                    
                }

                fn de(string: &str) -> Self {
                    
                }
            }
        })
    }
}