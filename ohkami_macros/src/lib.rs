use proc_macro::TokenStream;
mod internals;

#[proc_macro_derive(Json)]
pub fn derive_json(struct_stream: TokenStream) -> TokenStream {
    internals::derive_json(struct_stream.into())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}