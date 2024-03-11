use proc_macro2::{TokenStream, Span};
use syn::{parse::Parse, Ident, ItemStruct, Result, Error, token};
use quote::{quote, ToTokens};

use crate::components::*;


struct PayloadFormat {
    pt: PayloadType,
    ps: Option<PayloadSerde>,
} impl Parse for PayloadFormat {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut this = Self {
            pt: input.parse()?,
            ps: input.parse::<token::Div>()
                .is_ok().then_some(input.parse()?),
        };

        /* TODO: refactor serde-support combination management */
        match this.pt {
            PayloadType::JSON => {
                if this.ps.is_none() {/* set default derives */
                    this.ps = Some(PayloadSerde { S: true, D: true })
                }
            }
            PayloadType::Form => {
                if this.ps.is_some() {
                    return Err(Error::new(Span::call_site(), "#[Payload(Form)] doesn't support serde derive filtering syntax"))
                }
            }
            PayloadType::URLEncoded => {
                if this.ps.is_some() {
                    return Err(Error::new(Span::call_site(), "#[Payload(URLEncoded)] doesn't support serde derive filtering syntax"))
                }
            }
        }

        Ok(this)
    }
}
enum PayloadType {
    JSON,
    Form,
    URLEncoded,
} impl Parse for PayloadType {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        match &*input.parse::<Ident>()?.to_string() {
            "JSON" => Ok(Self::JSON),
            "Form" => Ok(Self::Form),
            "URLEncoded" => Ok(Self::URLEncoded),
            other => Err(Error::new(Span::call_site(), format!(
                "Unexpected payload type `{other}`: expected one of \n\
                - JSON\n\
                - Form\n\
                - URLEncoded\n\
            ")))
        }
    }
}
#[allow(non_snake_case)]
struct PayloadSerde {
    S: bool,
    D: bool,
} impl Parse for PayloadSerde {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut this = Self {
            S: false,
            D: false,
        };

        let mut seq = input.parse::<Ident>()?.to_string();
        while let Some(label) = seq.pop() {
            match label {
                'S' => this.S = true,
                'D' => this.D = true,
                other => return Err(Error::new(Span::call_site(), format!(
                    "Unexpected serde label `{other}`: expected a sequence of \n\
                    - S\n\
                    - D\n\
                ")))
            }
        }

        Ok(this)
    }
}



#[allow(non_snake_case)]
pub(super) fn Payload(format: TokenStream, target: TokenStream) -> Result<TokenStream> {
    let format: PayloadFormat = syn::parse2(format)?;
    let mut target = parse_request_struct("Payload", target)?;

    let impl_payload = match format.pt {
        PayloadType::JSON       => impl_payload_json(&target, &format.ps),
        PayloadType::Form       => impl_payload_formdata(&target),
        PayloadType::URLEncoded => impl_payload_urlencoded(&target),
    }?;

    if format.ps.as_ref().is_some_and(|sd| sd.S || sd.D) {
        target.attrs.retain(|a| a.path.to_token_stream().to_string() != "serde");
        for f in &mut target.fields {
            f.attrs.retain(|a| a.path.to_token_stream().to_string() != "serde")
        }
    }

    Ok(quote!{
        #target
        #impl_payload
    })
}

fn impl_payload_json(target: &ItemStruct, serde: &Option<PayloadSerde>) -> Result<TokenStream> {
    let struct_name = &target.ident;

    let (impl_lifetime, struct_lifetime) = match target.generics.lifetimes().count() {
        0 => (
            from_request_lifetime(),
            None,
        ),
        1 => (
            target.generics.params.first().unwrap().clone(),
            Some(target.generics.params.first().unwrap().clone()),
        ),
        _ => return Err(syn::Error::new(Span::call_site(), "#[Payload] doesn't support multiple lifetime params")),
    };

    let derives = serde.as_ref().map(|sd| {
        let mut d = TokenStream::new();
        if sd.S {
            d.extend(quote!{
                #[derive(::ohkami::serde::Serialize)]
                #[::ohkami::__internal__::consume_struct]
                #target
            })
        }
        if sd.D {
            d.extend(quote!{
                #[derive(::ohkami::serde::Deserialize)]
                #[::ohkami::__internal__::consume_struct]
                #target
            })
        }
        d
    });
    
    Ok(quote!{
        #derives

        impl<#impl_lifetime> ::ohkami::typed::ResponseBody for #struct_name<#struct_lifetime> {
            type Type = ::ohkami::typed::bodytype::JSON;
            #[inline(always)] fn into_response_with(self, status: ::ohkami::Status) -> ::ohkami::Response {
                ::ohkami::Response::with(status).json(self)
            }
        }

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

fn impl_payload_urlencoded(target: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &target.ident;
    let fields_data = FieldData::collect_from_struct_fields(&target.fields)?;

    let (impl_lifetime, struct_lifetime) = match target.generics.lifetimes().count() {
        0 => (
            from_request_lifetime(),
            None,
        ),
        1 => (
            target.generics.params.first().unwrap().clone(),
            Some(target.generics.params.first().unwrap().clone()),
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
fn impl_payload_formdata(target: &ItemStruct) -> Result<TokenStream> {
    let struct_name = &target.ident;
    let fields_data = FieldData::collect_from_struct_fields(&target.fields)?;

    let (impl_lifetime, struct_lifetime) = match target.generics.lifetimes().count() {
        0 => (
            from_request_lifetime(),
            None,
        ),
        1 => (
            target.generics.params.first().unwrap().clone(),
            Some(target.generics.params.first().unwrap().clone()),
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
                    unexpected => return ::std::result::Result::Err(::ohkami::Response::BadRequest().text(::std::format!("unexpected part in form-target: `{unexpected}`")))
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
                quote!{ #ident: #ident.ok_or_else(|| ::ohkami::Response::BadRequest().text(::std::concat!("Field `", #ident_str, "` is not found in the form-target")))?, }
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
                    ::ohkami::Response::BadRequest().text("Expected `multipart/form-target` and a boundary")
                }
                let ("multipart/form-target", boundary) = req.headers.ContentType().unwrap()
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
