mod queries;
mod components;

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Queries(_: proc_macro::TokenStream, data: proc_macro::TokenStream) -> proc_macro::TokenStream {
    queries::Queries(data.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
