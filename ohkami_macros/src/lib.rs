use proc_macro::TokenStream;
mod internals;


#[proc_macro]
pub fn json(content: TokenStream) -> TokenStream {
    internals::json_str(content.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
