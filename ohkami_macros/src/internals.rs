use proc_macro2::TokenStream;
use syn::Error;

trait Build {
    fn build(self) -> TokenStream;
}

mod json;
pub(super) fn json_str(content: TokenStream) -> Result<TokenStream, Error> {
    Ok(syn::parse2::<json::JsonStr>(content)?.build())
}
