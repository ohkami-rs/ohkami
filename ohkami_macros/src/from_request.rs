use proc_macro2::{Span, TokenStream};
use syn::{Field, ItemStruct, GenericParam, LifetimeParam, Lifetime};
use quote::quote;


pub(super) fn derive_from_request(target: TokenStream) -> syn::Result<TokenStream> {
    let s: ItemStruct = syn::parse2(target)?;

    let name = &s.ident;

    let generics_params_r = &s.generics.params;
    let generics_params_l = &mut generics_params_r.clone();
    let generics_where    = &s.generics.where_clause;

    let impl_lifetime = match s.generics.lifetimes().count() {
        0 => {
            let il = GenericParam::Lifetime(LifetimeParam::new(
                Lifetime::new("'__impl_from_request_lifetime", Span::call_site())
            ));
            generics_params_l.push(il.clone());
            il
        }
        1 => s.generics.params.first().unwrap().clone(),
        _ => return Err(syn::Error::new(Span::call_site(), "#[derive(FromRequest)] doesn't support multiple lifetime params")),
    };

    let build = if s.semi_token.is_none() {/* struct S { 〜 } */
        let fields = s.fields.into_iter()
            .map(|Field { ident, ty, .. }| quote! {
                #ident: {
                    match <#ty as ::ohkami::FromRequest>::from_request(req)? {
                        ::std::result::Result::Ok(field) => field,
                        ::std::result::Result::Err(err)  => return Some(::std::result::Result::Err(
                            ::ohkami::IntoResponse::into_response(err)
                        )),
                    }
                }
            });
        quote![ Self { #( #fields ),* } ]

    } else {/* struct T(); */
        let fields = s.fields.into_iter()
            .map(|Field { ty, .. }| quote! {
                {
                    match <#ty as ::ohkami::FromRequest>::from_request(req)? {
                        ::std::result::Result::Ok(field) => field,
                        ::std::result::Result::Err(err)  => return Some(::std::result::Result::Err(
                            ::ohkami::IntoResponse::into_response(err)
                        )),
                    }
                }
            });
        quote![ Self(#( #fields ),*) ]
    };

    Ok(quote! {
        impl<#generics_params_l> ::ohkami::FromRequest<#impl_lifetime> for #name<#generics_params_r>
            #generics_where
        {
            type Error = ::ohkami::Response;
            fn from_request(req: &#impl_lifetime ::ohkami::Request) -> ::std::option::Option<::std::result::Result<Self, Self::Error>> {
                ::std::option::Option::Some(::std::result::Result::Ok(#build))
            }
        }
    })
}
