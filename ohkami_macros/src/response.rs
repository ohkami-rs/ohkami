use proc_macro2::{TokenStream, Span};
use quote::{quote};
use syn::{Result, parse2, ItemStruct, Error};
use super::components::*;


#[allow(non_snake_case)]
pub(super) fn Response(format: TokenStream, data: TokenStream) -> Result<TokenStream> {
    let format = ResponseFormat::parse(format)?;
    let data   = parse2::<ItemStruct>(data)?;

    if !matches!(format, ResponseFormat::JSON) {
        return Err(Error::new(Span::call_site(),
            "Unexpected format"
        ))
    }

    let name = &data.ident;
    let generics_params = &data.generics.params;
    let generics_where  = &data.generics.where_clause;

    Ok(quote! {
        impl #generics_params ::ohkami::__internal__::ResponseBody for #name
            #generics_where
        {
            #[inline(always)] fn into_response_with(self, status: ::ohkami::http::Status) -> ::ohkami::Response {
                ::ohkami::Response::new(status).json(self)
            }
        }
    })
}

