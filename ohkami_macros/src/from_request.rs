use proc_macro2::{Span, TokenStream};
use syn::{Error, ItemStruct, Result};

pub(super) fn derive_from_request(target: TokenStream) -> Result<TokenStream> {
    let s: ItemStruct = syn::parse2(target)?;

    if s.generics.lifetimes().count() >= 2 {
        return Err(Error::new(Span::call_site(), "`#[derive(FromRequest)]` doesn't support multiple lifetimes!"))
    }

    if s.semi_token.is_none() {/* struct S { 〜 } */
        

    } else {/* struct T(); */

    }

    todo!()
}
