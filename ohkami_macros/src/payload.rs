use proc_macro2::{TokenStream, Ident, Span};
use syn::{Result, ItemStruct, parse_str, Type, Lifetime};
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
    // let lifetimes = &data.generics; // `parse_struct` checked this generics contains only lifetimes

    // let result_expr = if lifetimes.lifetimes().count() == 0 {
    //     quote!{
    //         ::std::result::Result::Ok(__payload__)
    //     }
    // } else {
    //     let req_lifetimes = lifetimes.lifetimes().map(|_| Lifetime::new("'req", Span::call_site()));
    //     quote!{
    //         ::std::result::Result::Ok(unsafe {
    //             ::std::mem::transmute::<
    //                 #struct_name<#( #req_lifetimes ),*>,
    //                 #struct_name #lifetimes
    //             >(__payload__)
    //         })
    //     }
    // };

    Ok(quote!{
        impl ::ohkami::FromRequest for #struct_name {
            fn parse<'req>(req: &'req ::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
                let (content_type, payload) = req.payload()
                    .ok_or_else(|| ::std::borrow::Cow::Borrowed("Expected payload"))?;
                if !content_type.is_json() {
                    return ::std::result::Result::Err(::std::borrow::Cow::Borrowed("Expected payload of `Content-Type: application/json`"))
                }
                let __payload__ = ::ohkami::internal::parse_json(payload)?;
                ::std::result::Result::Ok(__payload__)
            }
        }
    })
}

fn impl_payload_urlencoded(data: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &data.ident;
    // let lifetimes = &data.generics; // `parse_struct` checked this generics contains only lifetimes

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
            if stringified.starts_with("Option") {
                stringified.pop(/* final '>' */);
                (
                    /*
                        You know

                            `Option<Inner>`
                                |
                                (.to_token_stream)
                                |
                            TokenStram {
                                `Option`
                             -> `<`
                             -> `Inner`
                             -> `>`
                            }
                                |
                                (.to_string)
                                |
                            "Option < Inner >"
                                    
                        Take note that the string contains whitespaces as above:

                        NOT
                            "Option<"
                        BUT
                            "Option <"
                    */
                    parse_str::<Type>(stringified.strip_prefix("Option <").unwrap())?,
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
                let mut #ident = ::std::option::Option::<#ty>::None;
            }
        });

        quote!{
            #( #exprs )*
        }
    };

    let parsing_expr = {
        let arms = fields_data.iter().map(|FieldData { ident, ty, .. }| {
            let ident_str = ident.to_string();
            quote!{
                #ident_str => #ident.replace(<#ty as ::ohkami::internal::FromBuffer>::parse(v.as_bytes())?)
                    .map_or(::std::result::Result::Ok(()), |_|
                        ::std::result::Result::Err(::std::borrow::Cow::Borrowed(concat!("duplicated key: `", #ident_str,"`")))
                    )?,
            }
        });

        quote!{
            for (k, v) in ::ohkami::internal::parse_urlencoded(payload) {
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
                let ident_str = ident.to_string();
                quote!{ #ident: #ident.ok_or_else(|| ::std::borrow::Cow::Borrowed(concat!("`", #ident_str, "` is not found")))?, }
            }
        });

        quote!{
            ::std::result::Result::Ok(#struct_name {
                #( #fields )*
            })
        }
    };

    Ok(quote!{
        impl ::ohkami::FromRequest for #struct_name {
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
