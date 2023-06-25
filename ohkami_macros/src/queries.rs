use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{Result, Error, ItemStruct, parse2, Type};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Queries(data: TokenStream) -> Result<TokenStream> {
    let data = parse_struct("Queries", data)?;

    let impl_from_request = {
        let name = &data.ident;
        let lifetime_params = &data.generics; // checked to only contains lifetimes in `parse_struct`

        let mut fields = Vec::with_capacity(data.fields.len());
        for field in &data.fields {
            let field_name = field.ident.as_ref().unwrap(/* already checked in `parse_struct` */);

            let parsing_expr = match &field.ty {
                Type::Reference(r) => {
                    if (&r.elem).to_token_stream().to_string().as_str() != "str" {
                        return Err(Error::new(Span::call_site(), "Reference types other than `&str` are not supported"))
                    }
                    let lifetime = r.lifetime.as_ref().ok_or_else(||
                        Error::new(Span::call_site(), "Expected explicit lifetime name")
                    )?;

                    quote!{
                        req.query(#field_name).ok_or_else(||
                            ::std::borrow::Cow::Borrowed(
                                concat!("Expected query parameter `", #field_name, "`")
                            )
                        )?
                    }
                }
                _ => {
                    todo!(match to Option<_> or not)

                    &field.ty.to_token_stream().to_string()

                    quote!{
                        todo!()
                    }
                }
            };

            fields.push(quote!{
                #field_name: #parsing_expr,
            })
        }
        
        quote!{
            impl ::ohkami::FromRequest for #name #lifetime_params {
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
