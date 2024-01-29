use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{Result, parse_str, Type};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Query(data: TokenStream) -> Result<TokenStream> {
    let data = parse_request_struct("Query", data)?;

    let impl_from_request = {
        let struct_name = &data.ident;

        let (impl_lifetime, struct_lifetime) = match &data.generics.lifetimes().count() {
            0 => (
                from_request_lifetime(),
                None,
            ),
            1 => (
                data.generics.params.first().unwrap().clone(),
                Some(data.generics.params.first().unwrap().clone()),
            ),
            _ => return Err(syn::Error::new(Span::call_site(), "#[Query] doesn't support multiple lifetime params"))
        };

        let fields = data.fields.iter().map(|f| {
            let field_name = f.ident.as_ref().unwrap(/* already checked in `parse_request_struct` */);
            let field_name_str = field_name.to_string();
            let field_type = &f.ty;
            let field_type_str = field_type.to_token_stream().to_string();

            if field_type_str.starts_with("Option") {
                let inner_type = parse_str::<Type>(field_type_str.strip_prefix("Option <").unwrap().strip_suffix(">").unwrap()).unwrap();
                quote!{
                    #field_name: req.query::<#inner_type>(#field_name_str) // Option<Result<_>>
                        .transpose()
                        .map_err(|e| ::ohkami::FromRequestError::Owned(e.to_string()))?,
                }
            } else {
                quote!{
                    #field_name: req.query::<#field_type>(#field_name_str) // Option<Result<_>>
                        .ok_or_else(|| ::ohkami::FromRequestError::Static(
                            concat!("Expected query parameter `", #field_name_str, "`")
                        ))?
                        .map_err(|e| ::ohkami::FromRequestError::Owned(e.to_string()))?,
                }
            } 
        });
        
        quote!{
            impl<#impl_lifetime> ::ohkami::FromRequest<#impl_lifetime> for #struct_name<#struct_lifetime> {
                type Error = ::ohkami::FromRequestError;
                #[inline] fn from_request(req: &#impl_lifetime ::ohkami::Request) -> ::std::result::Result<Self, ::ohkami::FromRequestError> {
                    ::std::result::Result::Ok(Self {
                        #( #fields )*
                    })
                }
            }
        }
    };

    Ok(quote!{
        #data
        #impl_from_request
    })
}
