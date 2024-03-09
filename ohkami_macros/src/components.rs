use proc_macro2::{TokenStream, Span, Ident};
use quote::{format_ident, ToTokens};
use syn::{Result, Error, parse2, ItemStruct, Attribute, PathSegment, Type, Fields, parse_str, GenericParam, Lifetime, LifetimeDef};


pub(crate) fn from_request_lifetime() -> GenericParam {
    GenericParam::Lifetime(LifetimeDef::new(
        Lifetime::new("'__impl_from_request_lifetime", Span::call_site())
    ))
}

pub(crate) struct FieldData {
    pub(crate) ident:       Ident,
    pub(crate) ty:          Type,
    pub(crate) is_optional: bool,
} impl FieldData {
    pub(crate) fn collect_from_struct_fields(fields: &Fields) -> Result<Vec<Self>> {
        let mut fields_data = Vec::<FieldData>::with_capacity(fields.len());
        for field in fields {
            let ident = field.ident.as_ref().unwrap(/* `parse_struct` checked fields are named */).clone();
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

                            Take note that the string contains whitespaces as above; So it start with

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
        Ok(fields_data)
    }
}

pub(crate) fn parse_request_struct(macro_name: &str, input: TokenStream) -> Result<ItemStruct> {
    let mut struct_tokens = parse2::<ItemStruct>(input)?;

    if struct_tokens.semi_token.is_some() {
        return Err(Error::new(Span::call_site(), format!(
            "`#[{macro_name}]` doesn't support tuple or unit struct"
        )))
    }

    if struct_tokens.generics.type_params().count() > 0 {
        return Err(Error::new(Span::call_site(), format!(
            "`#[{macro_name}]` doesn't support generics"
        )))
    }

    if struct_tokens.generics.const_params().count() > 0 {
        return Err(Error::new(Span::call_site(), format!(
            "`#[{macro_name}]` doesn't support const params"
        )))
    }

    if struct_tokens.generics.lifetimes().count() >= 2 {
        return Err(Error::new(Span::call_site(), format!(
            "`#[{macro_name}]` doesn't support multiple lifetime params"
        )))
    }

    struct_tokens.attrs = struct_tokens.attrs.into_iter()
        .filter(|attr| is_not(attr, macro_name))
        .collect();

    Ok(struct_tokens)
}


pub(crate) enum ResponseFormat {
    JSON,
    JSONS,
} impl ResponseFormat {
    pub(crate) fn parse(tokens: TokenStream) -> Result<Self> {
        match tokens.to_token_stream().to_string().as_str() {
            "JSON"  => Ok(Self::JSON),
            "JSONS" => Ok(Self::JSONS),
            _ => Err(Error::new(Span::mixed_site(), "\
                Valid format: \n\
                - `#[Response(JSON)]` \n\
                - `#[Response(JSONS)]` \n\
            "))
        }
    }
}




fn is_not(attr: &Attribute, name: &str) -> bool {
    let mut segments = attr.path.segments.iter().peekable();

    let is_just_ident = |s: &PathSegment, ident: &str| {
        s.arguments.is_empty() &&
        s.ident == format_ident!("{ident}")
    };

    let is_ident = |s: &PathSegment, ident: &str| {
        s.ident == format_ident!("{ident}")
    };

    match segments.next_if(|s| is_ident(s, name)) {
        Some(_) => false, /* it's me:
            #[Queries] | #[Payload(JSON | Form)]
        */
        None    => {
            match segments.next_if(|s| is_just_ident(s, "ohkami")) {
                None    => true,
                Some(_) => match segments.next_if(|s| is_just_ident(s, name)) {
                    None    => true,
                    Some(_) => false, /* it's me:
                        #[ohkami::Queries]
                        | #[::ohkami::Queries]
                        | #[ohkami::Payload(JSON | Form)]
                        | #[::ohkami::Payload(JSON | Form)]
                    */
                }
            }
        }
    }
}
