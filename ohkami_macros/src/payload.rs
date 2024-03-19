use proc_macro2::TokenStream;
use syn::{parse::Parse, token, Error, Ident, ItemStruct, Path, Result};
use quote::quote;

struct PayloadFormat {
    payload_type: Path,
    serde_derive: Option<SerdeDerive>,
} impl Parse for PayloadFormat {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            payload_type: input.parse()?,
            serde_derive: input.parse::<token::Div>().is_ok().then_some(input.parse()?)
        })
    }
}

#[allow(non_snake_case)]
struct SerdeDerive {
    S: bool,
    D: bool,
} impl Parse for SerdeDerive {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut this = Self { S:false, D:false };
        {
            let derives = input.parse::<Ident>()?;

            let derives_str = derives.to_string();
            for c in derives_str.chars() {
                match c {
                    'S' => this.S = true,
                    'D' => this.D = true,
                     _  => return Err(Error::new(
                        derives.span(), format!(
                            "Unexpected derive specified `{c}`: \
                            only `S` (for Serialize) or `D` (for Deserialize) \
                            are available here"
                        )
                    )),
                }
            }
        }
        Ok(this)
    }
} impl SerdeDerive {
    fn into_derive(self) -> Option<TokenStream> {
        let mut derives = Vec::new();
        if self.S {
            derives.extend(quote!{
                ::ohkami::__internal__::serde::Serialize
            });
        }
        if self.D {
            derives.extend(quote!{
                ::ohkami::__internal__::serde::Deserialize
            });
        }

        (!derives.is_empty()).then_some(quote!{
            #[derive( #( #derives ),* )]
        })
    }
}

#[allow(non_snake_case)]
pub(super) fn Payload(format: TokenStream, target: TokenStream) -> Result<TokenStream> {
    let format: PayloadFormat = syn::parse2(format)?;
    let target: ItemStruct    = syn::parse2(target)?;

    let name            = &target.ident;
    let generics_params = &target.generics.params;
    let generics_where  = &target.generics.where_clause;

    let PayloadFormat { payload_type, serde_derive } = format;

    let serde_derive = match serde_derive {
        None     => None,
        Some(sd) => sd.into_derive(),
    };

    Ok(quote!{
        #serde_derive
        #target

        impl<#generics_params> ::ohkami::typed::Payload for #name<#generics_params>
            #generics_where
        {
            type Type = #payload_type;
        }
    })
}
