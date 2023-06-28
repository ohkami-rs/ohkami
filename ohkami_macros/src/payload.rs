use proc_macro2::{TokenStream, Ident};
use syn::{Result, ItemStruct, parse_str, Type};
use quote::{quote, ToTokens};

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

    let mut fields_data = Vec::<FieldData>::with_capacity(data.fields.len());
    struct FieldData {
        ident:       Ident,
        ty:          Type,
        is_optional: bool,
    }
    for field in &data.fields {
        let ident = field.ident.as_ref().unwrap(/* `parse_struct` checked fields is named */).clone();
        let (ty, is_optional) = {
            let mut stringified = field.ty.to_token_stream().to_string();
            if stringified.starts_with("Option<") {
                stringified.pop(/* final '>' */);
                (
                    parse_str::<Type>(stringified.strip_prefix("Option<").unwrap())?,
                    true,
                )
            } else {
                (
                    parse_str::<Type>(&stringified)?,
                    false,
                )
            }
        };
        fields_data.push(FieldData { ident, ty, is_optional })
    }

    let declaring_exprs = {
        let exprs = fields_data.iter().map(|FieldData { ident, ty, .. }| {
            quote!{
                let mut #ident = ::std::option::Option<#ty>::None;
            }
        });

        quote!{
            #( #exprs )*
        }
    };

    let parsing_expr = {
        let arms = fields_data.iter().map(|FieldData { ident, ty, .. }| {
            let ident_asstr = ident.to_string();
            quote!{
                #ident_asstr => #ident.replace(<#ty as ::ohkami::internal::FromBuffer>::parse(v.as_bytes())?)
                    .map_or(::std::result::Result::Ok(()), |_|
                        ::std::result::Result::Err(::std::borrow::Cow::Borrowed(concat!("duplicated key: `", #ident,"`")))
                    )?,
            }
        });

        quote!{
            for (k, v) in ::ohkami::internal::parse_payload(payload) {
                match &*k {
                    #( #arms )*
                    unexpected => return ::std::result::Result::Err(::std::borrow::Cow::Owned(format!("unexpected key: `{unexpected}`")))
                }
            }
        }
    };

    let building_expr = {
        let fields = fields_data.iter().map(|FieldData { ident, is_optional, .. }| {
            if *is_optional {
                quote!{ #ident, }
            } else {
                quote!{ #ident: #ident.ok_or_else(|| ::std::borrow::Cow::Borrowed(concat!("`", #ident, "` is not found")))?, }
            }
        });

        quote!{
            ::std::result::Result::Ok(#struct_name {
                #( #fields )*
            })
        }
    };

    Ok(quote!{
        impl #lifetimes ::ohkami::FromRequest for #struct_name #lifetimes {
            fn parse(req: &::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
                let (content_type, payload) = req.payload()
                    .ok_or_else(|| ::std::borrow::Cow::Borrowed("Expected payload"))?;
                if !content_type.is_urlencoded() {
                    return ::std::result::Result::Err(::std::borrow::Cow::Borrowed("Expected payload of `Content-Type: application/x-www-form-urlencoded`"))
                }

                #declaring_exprs
                #parsing_expr
                #building_expr
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
