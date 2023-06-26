use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};
use syn::{Result};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Queries(data: TokenStream) -> Result<TokenStream> {
    let data = parse_struct("Queries", data)?;

    let impl_from_request = {
        let struct_name = &data.ident;
        let lifetime_params = &data.generics; // checked to only contains lifetimes in `parse_struct`

        let fields = data.fields.iter().map(|f| {
            let field_name = f.ident.as_ref().unwrap(/* already checked in `parse_struct` */);
            let field_type = &f.ty;

            if field_type.to_token_stream().to_string().starts_with("Option") {
                quote!{
                    #field_name: req.query::<#field_type>(concat!(#field_name))
                        .transpose()?,
                }
            } else {
                quote!{
                    #field_name: req.query::<#field_type>(concat!(#field_name)) // Option<Result<_>>
                        .ok_or_else(|| ::std::result::Result::Err(::std::borrow::Cow::Borrowed(
                            concat!("Expected query parameter `", #field_name, "`")
                        )))??,
                }
            } 
        });
        
        quote!{
            impl ::ohkami::FromRequest for #struct_name #lifetime_params {
                fn parse(req: &::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
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
