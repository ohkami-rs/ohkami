use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};
use syn::{Result, parse2, ItemStruct};
use super::components::*;


#[allow(non_snake_case)]
pub(super) fn ResponseBody(format: TokenStream, data: TokenStream) -> Result<TokenStream> {
    let format = ResponseFormat::parse(format)?;
    let data   = parse2::<ItemStruct>(data)?;

    let name = &data.ident;
    let generics_params = &data.generics.params;
    let generics_where  = &data.generics.where_clause;

    Ok(match &format {
        ResponseFormat::JSON | ResponseFormat::JSONS => {
            let derive_serialize = matches!(format, ResponseFormat::JSONS).then_some(quote! {
                #[derive(::ohkami::serde::Serialize)]
                #[::ohkami::__internal__::consume_struct]
                #data
            });

            let data = {
                let mut data = data.clone();
                if matches!(format, ResponseFormat::JSONS) {
                    data.attrs.retain(|a| a.path.to_token_stream().to_string() != "serde");
                    for f in &mut data.fields {
                        f.attrs.retain(|a| a.path.to_token_stream().to_string() != "serde")
                    }
                }
                data
            };

            quote! {
                #derive_serialize
                #data

                impl<#generics_params> ::ohkami::typed::ResponseBody for #name<#generics_params>
                    #generics_where
                {
                    type Type = ::ohkami::typed::body_type::JSON;
                    #[inline(always)] fn into_response_with(self, status: ::ohkami::Status) -> ::ohkami::Response {
                        ::ohkami::Response::with(status).json(self)
                    }
                }
            }
        }
    })
}
