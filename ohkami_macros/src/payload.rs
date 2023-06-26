use proc_macro2::{TokenStream};
use syn::{Result, ItemStruct};
use quote::{quote};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Payload(format: TokenStream, data: TokenStream) -> Result<TokenStream> {
    let format = Format::parse(format)?;
    let data = parse_struct("Payload", data)?;

    let impl_payload = match format {
        Format::JSON       => impl_payload_json(&data),
        Format::Form       => impl_payload_form(&data),
        Format::URLEncoded => impl_payload_urlencoded(&data),
    }?;

    Ok(quote!{
        #data
        #impl_payload
    })
}

fn impl_payload_json(data: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &data.ident;
    let lifetimes = &data.generics; // `parse_struct` checked this generics contains only lifetimes

    Ok(quote!{
        impl #lifetimes ::ohkami::FromRequest for #struct_name #lifetimes {
            fn parse(req: &::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
                let (content_type, payload) = req.payload()
                    .ok_or_else(|| ::std::borrow::Cow::Borrowed("Expected payload"))?;
                if !content_type.is_json() {
                    return ::std::result::Result::Err(::std::borrow::Cow::Borrowed("Expected payload of `Content-Type: application/json`"))
                }
                ::ohkami::internal::parse_json(payload)
            }
        }
    })
}

fn impl_payload_urlencoded(data: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &data.ident;
    let lifetimes = &data.generics; // `parse_struct` checked this generics contains only lifetimes

    Ok(quote!{
        impl #lifetimes ::ohkami::FromRequest for #struct_name #lifetimes {
            fn parse(req: &::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
                let (content_type, payload) = req.payload()
                    .ok_or_else(|| ::std::borrow::Cow::Borrowed("Expected payload"))?;
                if !content_type.is_urlencoded() {
                    return ::std::result::Result::Err(::std::borrow::Cow::Borrowed("Expected payload of `Content-Type: application/json`"))
                }
                
                ===== TODO =====
            }
        }
    })
}

#[allow(unused)] //
fn impl_payload_form(data: &ItemStruct) -> Result<TokenStream> {
    Ok(quote!{
        unimplemented!("`#[Payload(Form)]` is not implemented yet. Please wait for development, or, if you need this imediately, you can implement and create a [Pull request](https://github.com/kana-rus/ohkami/pulls) !")
    })
}
