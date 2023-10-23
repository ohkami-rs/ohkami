use proc_macro2::{TokenStream, Span};
use syn::{Result, ItemStruct};
use quote::{quote, ToTokens};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Payload(format: TokenStream, data: TokenStream) -> Result<TokenStream> {
    let format = Format::parse(format)?;
    let data = parse_struct("Payload", data)?;

    let impl_payload = match format {
        Format::JSON       => impl_payload_json(&data),
        Format::URLEncoded => impl_payload_urlencoded(&data),
        Format::Form       => impl_payload_formdata(&data),
    }?;

    Ok(quote!{
        #data
        #impl_payload
    })
}

fn impl_payload_json(data: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &data.ident;
    
    Ok(quote!{
        impl ::ohkami::FromRequest for #struct_name {
            fn parse<'req>(req: &'req ::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
                let (content_type, payload) = req.payload()
                    .ok_or_else(|| ::std::borrow::Cow::Borrowed("Expected payload"))?;
                if !content_type.is_json() {
                    return ::std::result::Result::Err(::std::borrow::Cow::Borrowed("Expected payload of `Content-Type: application/json`"))
                }
                let __payload__ = ::ohkami::__internal__::parse_json(payload)?;
                ::std::result::Result::Ok(__payload__)
            }
        }
    })
}

fn impl_payload_urlencoded(data: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &data.ident;
    let fields_data = FieldData::collect_from_struct_fields(&data.fields)?;

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
                #ident_str => #ident.replace(<#ty as ::ohkami::__internal__::FromBuffer>::parse(v.as_bytes())?)
                    .map_or(::std::result::Result::Ok(()), |_|
                        ::std::result::Result::Err(::std::borrow::Cow::Borrowed(concat!("duplicated key: `", #ident_str,"`")))
                    )?,
            }
        });

        quote!{
            for (k, v) in ::ohkami::__internal__::parse_urlencoded(payload) {
                match &*k {
                    #( #arms )*
                    unexpected => return ::std::result::Result::Err(::std::borrow::Cow::Owned(::std::format!("unexpected key: `{unexpected}`")))
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
                quote!{ #ident: #ident.ok_or_else(|| ::std::borrow::Cow::Borrowed(::std::concat!("`", #ident_str, "` is not found")))?, }
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
                    .ok_or_else(|| ::std::borrow::Cow::Borrowed("Expected a payload"))?;
                if !content_type.is_urlencoded() {
                    return ::std::result::Result::Err(::std::borrow::Cow::Borrowed("Expected a `application/x-www-form-urlencoded` payload"))
                }

                #declaring_exprs
                #parsing_expr
                #building_expr
            }
        }
    })
}

#[allow(unused)]
fn impl_payload_formdata(data: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &data.ident;
    let fields_data = FieldData::collect_from_struct_fields(&data.fields)?;

    // `#[Payload(FormData)]` doesn't accept optional fields
    if fields_data.iter().any(|FieldData { is_optional, .. }| *is_optional) {
        return Err(syn::Error::new(Span::mixed_site(), "`Option<_>` is not available in `#[Payload(FormData)]`"))
    }

    let declaring_exprs = {
        let exprs = fields_data.iter().map(|FieldData { ident, .. }| quote!{
            let mut #ident = ::std::option::Option::None;
        });

        quote!{
            #( #exprs )*
        }
    };
    
    let parsing_expr = {
        enum PartType { Field, Files, File }
        impl PartType {
            fn into_method_call(&self) -> TokenStream {
                match self {
                    Self::Field => quote!{ form_part.into_field()?.text().map_err(|e| ::std::borrow::Cow::Owned(::std::format!("Invalid form text: {e}")))? },
                    Self::Files => quote!{ form_part.into_files()? },
                    Self::File  => quote!{ form_part.into_file()? },
                }
            }
        }

        let arms = fields_data.iter().map(|FieldData { ident, ty, .. }| {
            let part_name = ident.to_string().replace("_", "-");

            let into_the_field = match &*ty.to_token_stream().to_string().split_ascii_whitespace().collect::<String>() {
                "String" => PartType::Field,
                "File" | "utils::File" | "ohkami::File" | "::ohkami::File" => PartType::File,
                "Vec<File>" | "Vec<utils::File>" | "Vec<ohkami::utils::File>" | "Vec<::ohkami::utils::File>" => PartType::Files,
                unexpected  => return Err(syn::Error::new(Span::call_site(), &format!("Unexpected field type `{unexpected}` : `#[Payload(FormData)]` supports only `String`, `File` or `Vec<File>` as field type")))
            }.into_method_call();

            Ok(quote!{
                #part_name => #ident = ::std::option::Option::Some(#into_the_field),
            })
        }).collect::<Result<Vec<_>>>()?;

        quote!{
            for form_part in ::ohkami::__internal__::parse_formparts(payload, &boundary)? {
                match form_part.name() {
                    #( #arms )*
                    unexpected => return ::std::result::Result::Err(::std::borrow::Cow::Owned(::std::format!("unexpected part in form-data: `{unexpected}`")))
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
                quote!{ #ident: #ident.ok_or_else(|| ::std::borrow::Cow::Borrowed(::std::concat!("Field `", #ident_str, "` is not found in the form-data")))?, }
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
                    .ok_or_else(|| ::std::borrow::Cow::Borrowed("Expected a payload"))?;
                let ::ohkami::ContentType::FormData { boundary } = content_type
                    else {return ::std::result::Result::Err(::std::borrow::Cow::Borrowed("Expected a `multipart/form-data` payload"))};
            
                #declaring_exprs
                #parsing_expr
                #building_expr
            }
        }
    })
}
