use proc_macro2::{TokenStream, Span};
use syn::{Result, ItemStruct};
use quote::{quote, ToTokens};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Payload(format: TokenStream, data: TokenStream) -> Result<TokenStream> {
    let format = PayloadFormat::parse(format)?;
    let mut data = parse_request_struct("Payload", data)?;

    let impl_payload = match format {
        PayloadFormat::JSON       => impl_payload_json(&data, false),
        PayloadFormat::JSOND      => impl_payload_json(&data, true),
        PayloadFormat::URLEncoded => impl_payload_urlencoded(&data),
        PayloadFormat::Form       => impl_payload_formdata(&data),
    }?;

    if matches!(format, PayloadFormat::JSOND) {
        data.attrs.retain(|a| a.path.to_token_stream().to_string() != "serde");
        for f in &mut data.fields {
            f.attrs.retain(|a| a.path.to_token_stream().to_string() != "serde")
        }
    }

    Ok(quote!{
        #data
        #impl_payload
    })
}

fn impl_payload_json(data: &ItemStruct, derive_deserialize: bool) -> Result<TokenStream> {
    let struct_name = &data.ident;

    let (impl_lifetime, struct_lifetime) = match data.generics.lifetimes().count() {
        0 => (
            from_request_lifetime(),
            None,
        ),
        1 => (
            data.generics.params.first().unwrap().clone(),
            Some(data.generics.params.first().unwrap().clone()),
        ),
        _ => return Err(syn::Error::new(Span::call_site(), "#[Payload] doesn't support multiple lifetime params")),
    };

    let derive_deserialize = derive_deserialize.then_some(quote! {
        #[derive(::ohkami::serde::Deserialize)]
        #[::ohkami::__internal__::consume_struct]
        #data
    });
    
    Ok(quote!{
        #derive_deserialize

        impl<#impl_lifetime> ::ohkami::FromRequest<#impl_lifetime> for #struct_name<#struct_lifetime> {
            type Error = ::ohkami::Response;
            #[inline] fn from_request(req: &#impl_lifetime ::ohkami::Request) -> ::std::result::Result<Self, Self::Error> {
                let payload = req.payload()
                    .ok_or_else(|| ::ohkami::Response::BadRequest().text("Expected payload"))?;
                if !req.headers.ContentType().unwrap().starts_with("application/json") {
                    return ::std::result::Result::Err(::ohkami::Response::BadRequest().text("Expected a payload of `Content-Type: application/json`"))
                }
                let __payload__ = ::ohkami::__internal__::parse_json(payload).map_err(|e| ::ohkami::Response::InternalServerError().text(e.to_string()))?;
                ::std::result::Result::Ok(__payload__)
            }
        }
    })
}

fn impl_payload_urlencoded(data: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &data.ident;
    let fields_data = FieldData::collect_from_struct_fields(&data.fields)?;

    let (impl_lifetime, struct_lifetime) = match data.generics.lifetimes().count() {
        0 => (
            from_request_lifetime(),
            None,
        ),
        1 => (
            data.generics.params.first().unwrap().clone(),
            Some(data.generics.params.first().unwrap().clone()),
        ),
        _ => return Err(syn::Error::new(Span::call_site(), "#[Payload] doesn't support multiple lifetime params")),
    };

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
                #ident_str => #ident.replace(<#ty as ::ohkami::FromParam>::parse(v.as_bytes())?)
                    .map_or(::std::result::Result::Ok(()), |_|
                        ::std::result::Result::Err(::ohkami::Response::BadRequest().text(concat!("Duplicated key: `", #ident_str,"`")))
                    )?,
            }
        });

        quote!{
            for (k, v) in ::ohkami::__internal__::parse_urlencoded(payload) {
                match &*k {
                    #( #arms )*
                    unexpected => return ::std::result::Result::Err(::ohkami::Response::BadRequest().text(::std::format!("Unexpected key: `{unexpected}`")))
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
                quote!{ #ident: #ident.ok_or_else(|| ::ohkami::Response::BadRequest().text(::std::concat!("`", #ident_str, "` is not found")))?, }
            }
        });

        quote!{
            ::std::result::Result::Ok(#struct_name {
                #( #fields )*
            })
        }
    };

    Ok(quote!{
        impl<#impl_lifetime> ::ohkami::FromRequest<#impl_lifetime> for #struct_name<#struct_lifetime> {
            type Error = ::ohkami::Response;
            fn from_request(req: &#impl_lifetime ::ohkami::Request) -> ::std::result::Result<Self, Self::Error> {
                let payload = req.payload()
                    .ok_or_else(|| ::ohkami::Response::BadRequest().text("Expected a payload"))?;
                if !req.headers.ContentType().unwrap().starts_with("application/x-www-form-urlencoded") {
                    return ::std::result::Result::Err(::ohkami::Response::BadRequest().text("Expected an `application/x-www-form-urlencoded` payload"))
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

    let (impl_lifetime, struct_lifetime) = match data.generics.lifetimes().count() {
        0 => (
            from_request_lifetime(),
            None,
        ),
        1 => (
            data.generics.params.first().unwrap().clone(),
            Some(data.generics.params.first().unwrap().clone()),
        ),
        _ => return Err(syn::Error::new(Span::call_site(), "#[Payload] doesn't support multiple lifetime params")),
    };

    // `#[Payload(Form)]` doesn't accept optional fields
    if fields_data.iter().any(|FieldData { is_optional, .. }| *is_optional) {
        return Err(syn::Error::new(Span::mixed_site(), "`Option<_>` is not available in `#[Payload(Form)]`"))
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
                    Self::Field => quote!{ form_part.into_field().map_err(|e| ::ohkami::Response::InternalServerError().text(e))?.text().map_err(|e| ::ohkami::Response::InternalServerError().text(::std::format!("Invalid form text: {e}")))? },
                    Self::Files => quote!{ form_part.into_files().map_err(|e| ::ohkami::Response::InternalServerError().text(e))? },
                    Self::File  => quote!{ form_part.into_file().map_err(|e| ::ohkami::Response::InternalServerError().text(e))? },
                }
            }
        }

        let arms = fields_data.iter().map(|FieldData { ident, ty, .. }| {
            let part_name = ident.to_string().replace("_", "-");

            let into_the_field = match &*ty.to_token_stream().to_string().split_ascii_whitespace().collect::<String>() {
                "String" => PartType::Field,
                "File" | "utils::File" | "ohkami::File" | "::ohkami::File" => PartType::File,
                "Vec<File>" | "Vec<utils::File>" | "Vec<ohkami::utils::File>" | "Vec<::ohkami::utils::File>" => PartType::Files,
                unexpected  => return Err(syn::Error::new(Span::call_site(), &format!("Unexpected field type `{unexpected}` : `#[Payload(Form)]` supports only `String`, `File` or `Vec<File>` as field type")))
            }.into_method_call();

            Ok(quote!{
                #part_name => #ident = ::std::option::Option::Some(#into_the_field),
            })
        }).collect::<Result<Vec<_>>>()?;

        quote!{
            for form_part in ::ohkami::__internal__::parse_formparts(payload, &boundary).map_err(|e| ::ohkami::Response::InternalServerError().text(e))? {
                match form_part.name() {
                    #( #arms )*
                    unexpected => return ::std::result::Result::Err(::ohkami::Response::BadRequest().text(::std::format!("unexpected part in form-data: `{unexpected}`")))
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
                quote!{ #ident: #ident.ok_or_else(|| ::ohkami::Response::BadRequest().text(::std::concat!("Field `", #ident_str, "` is not found in the form-data")))?, }
            }
        });

        quote!{
            ::std::result::Result::Ok(#struct_name {
                #( #fields )*
            })
        }
    };

    Ok(quote!{
        impl<#impl_lifetime> ::ohkami::FromRequest<#impl_lifetime> for #struct_name<#struct_lifetime> {
            type Error = ::ohkami::Response;

            fn from_request(req: &#impl_lifetime ::ohkami::Request) -> ::std::result::Result<Self, Self::Error> {
                let payload = req.payload()
                    .ok_or_else(|| ::ohkami::Response::BadRequest().text("Expected a payload"))?;

                #[cold] fn __expected_multipart_formdata_and_boundary() -> ::ohkami::Response {
                    ::ohkami::Response::BadRequest().text("Expected `multipart/form-data` and a boundary")
                }
                let ("multipart/form-data", boundary) = req.headers.ContentType().unwrap()
                    .split_once("; boundary=")
                    .ok_or_else(__expected_multipart_formdata_and_boundary)?
                else {
                    return ::std::result::Result::Err(__expected_multipart_formdata_and_boundary())
                };
                
                #declaring_exprs
                #parsing_expr
                #building_expr
            }
        }
    })
}
