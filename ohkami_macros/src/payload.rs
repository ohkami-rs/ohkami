use proc_macro2::TokenStream;
use syn::{parse::Parse, token, Error, Ident, ItemStruct, Path, Result};
use quote::quote;

struct PayloadFormat {
    payload_type: Path,
    serde_derive: Option<SerdeDerive>,
    validation:   Option<Validation>,
} impl Parse for PayloadFormat {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            payload_type: input.parse()?,
            serde_derive: input.peek(token::Div).then(|| {
                input.parse::<token::Div>().unwrap();
                input.parse()
            }).transpose()?,
            validation: input.peek(token::Where).then(|| {
                input.parse()
            }).transpose()?,
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
    fn into_derive(self) -> TokenStream {
        let mut derives = Vec::new();
        if self.S {
            derives.extend(quote!{
                ::ohkami::serde::Serialize,
            });
        }
        if self.D {
            derives.extend(quote!{
                ::ohkami::serde::Deserialize,
            });
        }

        quote!{
            #[derive( #( #derives )* )]
        }
    }
}

struct Validation {
    _where:   token::Where,
    validate: TokenStream,
} impl Parse for Validation {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            _where:   input.parse()?,
            validate: input.parse()?,
        })
    }
} impl Validation {
    fn into_fn(self) -> TokenStream {
        let Self { validate, .. } = self;
        quote! {
            #[inline]
            fn validate(&self) -> std::result::Result<(), impl std::fmt::Display> {
                #validate(self)
            }
        }
    }
}

#[allow(non_snake_case)]
pub(super) fn Payload(format: TokenStream, target: TokenStream) -> Result<TokenStream> {
    let format: PayloadFormat = syn::parse2(format)?;
    let target: ItemStruct    = syn::parse2(target)?;

    let name            = &target.ident;
    let generics_params = &target.generics.params;
    let generics_where  = &target.generics.where_clause;

    let PayloadFormat { payload_type, serde_derive, validation } = format;

    let serde_derive = serde_derive.map(SerdeDerive::into_derive);
    let validation   = validation.map(Validation::into_fn);

    Ok(quote!{
        #serde_derive
        #target

        impl<#generics_params> ::ohkami::typed::Payload for #name<#generics_params>
            #generics_where
        {
            type Type = #payload_type;

            #validation
        }
    })
}
