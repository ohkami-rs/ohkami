use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{Result, parse_str, Type, GenericParam, LifetimeDef, Lifetime};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Query(data: TokenStream) -> Result<TokenStream> {
    let data = parse_struct("Query", data)?;

    let impl_from_request = {
        let struct_name = &data.ident;
        let lifetimes = &data.generics; // checked to only contains lifetimes in `parse_struct`

        let fr = GenericParam::Lifetime(
            LifetimeDef::new(
                Lifetime::new("'__impl_from_request_lifetime", Span::call_site())
            )
        );
        let lifetimes_with_fr = {
            let mut with = lifetimes.clone();
            with.params.push(fr.clone());
            with
        };

        let fields = data.fields.iter().map(|f| {
            let field_name = f.ident.as_ref().unwrap(/* already checked in `parse_struct` */);
            let field_name_str = field_name.to_string();
            let field_type = &f.ty;
            let field_type_str = field_type.to_token_stream().to_string();

            if field_type_str.starts_with("Option") {
                let inner_type = parse_str::<Type>(field_type_str.strip_prefix("Option <").unwrap().strip_suffix(">").unwrap()).unwrap();
                quote!{
                    #field_name: req.query::<#inner_type>(#field_name_str) // Option<Result<_>>
                        .transpose()
                        .map_err(|e| ::std::borrow::Cow::Owned(e.to_string()))?,
                }
            } else {
                quote!{
                    #field_name: req.query::<#field_type>(#field_name_str) // Option<Result<_>>
                        .ok_or_else(|| ::std::borrow::Cow::Borrowed(
                            concat!("Expected query parameter `", #field_name_str, "`")
                        ))?
                        .map_err(|e| ::std::borrow::Cow::Owned(e.to_string()))?,
                }
            } 
        });
        
        quote!{
            impl #lifetimes_with_fr ::ohkami::FromRequest<#fr> for #struct_name #lifetimes {
                type Error = ::std::borrow::Cow<'static, str>;
                fn parse(req: &#fr ::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
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
