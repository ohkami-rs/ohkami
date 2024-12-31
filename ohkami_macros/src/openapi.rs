#![cfg(feature="openapi")]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Visibility, Ident, LitInt, LitStr, token, Token};

pub(super) fn derive_schema(input: TokenStream) -> syn::Result<TokenStream> {
    todo!()
}

pub(super) fn operation(meta: TokenStream, handler: TokenStream) -> syn::Result<TokenStream> {
    #[allow(non_snake_case)]
    struct OperationMeta {
        operationId: Option<String>,
        description: Option<String>,
        overrides:   Vec<Override>,
    }

    struct Override {
        key:   OverrideTarget,
        value: String,
    }
    enum OverrideTarget {
        Summary,
        RequestBody,
        DefaultResponse,
        Response { status: u16 },
        Param { name: String },
    }

    mod override_keyword {
        syn::custom_keyword!(summary);
        syn::custom_keyword!(requestBody);
    }

    impl syn::parse::Parse for Override {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let key = if false {
            } else if input.peek(override_keyword::summary) {
                input.parse::<override_keyword::summary>()?;
                OverrideTarget::Summary

            } else if input.peek(override_keyword::requestBody) {
                input.parse::<override_keyword::requestBody>()?;
                OverrideTarget::RequestBody

            } else if input.peek(Token![default]) {
                input.parse::<Token![default]>()?;
                OverrideTarget::DefaultResponse

            } else if input.peek(LitInt) {
                let status = input.parse::<LitInt>()?.base10_parse()?;
                OverrideTarget::Response { status }
                
            } else if input.peek(Ident) {
                let name = input.parse::<Ident>()?.to_string();
                OverrideTarget::Param { name }

            } else {
                return Err(syn::Error::new(input.span(), format!("\
                    Unepected description key: `{}`. Expected one of\n\
                    - summary       (.summary)\n\
                    - requestBody   (.requestBody.description)\n\
                    - default       (.responses.default.description)\n\
                    - <status:int>  (.responses.<status>.description)\n\
                    - <param:ident> (.parameters.<param>.description)\n\
                ",
                    input.parse2::<TokenStream>()?
                )))
            };

            input.parse::<Token![:]>()?;

            let value = input.parse::<LitStr>()?.value();

            Ok(Self { key, value })
        }
    }

    impl syn::parse::Parse for OperationMeta {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let description = None/* load later */;

            let operationId = input.peek(Ident)
                .then(|| input.parse())
                .transpose()?;

            let overrides = input.peek(token::Brace)
                .then(|| {
                    let overrides; syn::braced!(overrides in input);
                    overrides
                        .parse_terminated(Override::parse, Token![,])
                        .map(|iter| iter.collect::<Vec<_>>())
                })
                .transpose()?
                .unwrap_or_default();


            Ok(Self { operationId, description, overrides })
        }
    }

    //////////////////////////////////////////////////////////////

    let meta = syn::parse2::<OperationMeta>(meta)?;

    let handler = syn::parse2::<ItemFn>(handler)?;
    let handler_vis  = handler.vis;
    let handler_name = handler.ident;

    let handler = {
        let mut handler = handler.clone();
        handler.vis = Visibility::Public(Token![pub]);
        handler
    };

    let modify_op = {
        let mut modify_op = TokenStream::new();

        modify_op
    };

    Ok(quote! {
        #[allow(non_camelcase_types)]
        #handler_vis struct #handler_name;
        const _: () = {
            mod operation {
                use super::*;
                #handler
            }

            impl ::ohkami::handler::IntoHandler<#handler_name> for #handler_name {
                fn into_handler(self) -> ::ohkami::handler::Handler {
                    ::ohkami::handler::IntoHandler::into_handler(operation::#handler_name)
                        .map_openapi_operation(|op| { #modify_op })
                }
            }
        };
    })
}
