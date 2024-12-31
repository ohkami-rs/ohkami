#![cfg(feature="openapi")]

use std::collections::HashMap;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Visibility, Token};

pub(super) fn derive_schema(input: TokenStream) -> syn::Result<TokenStream> {
    todo!()
}

pub(super) fn operation(meta: TokenStream, handler: TokenStream) -> syn::Result<TokenStream> {
    #[allow(non_snake_case)]
    struct OperationMeta {
        summary:               Option<String>,
        description:           Option<String>,
        operationId:           Option<String>,
        description_overrides: HashMap<DescriptionTarget, String>,
    }
    #[derive(PartialEq, PartialOrd)]
    enum DescriptionTarget {
        RequestBody,
        Param { name: String },
        Response { status: String },
    }

    impl syn::Parse for OperationMeta {
        fn parse(input: syn::ParseBuf) -> syn::Result<Self> {

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
