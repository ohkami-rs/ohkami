use quote::{format_ident, quote};
use proc_macro2::{TokenStream, Span};
use syn::{Error, Field, GenericParam, ItemStruct, Lifetime, LifetimeDef, Result};


#[allow(non_snake_case)]
pub(super) fn Query(target: TokenStream) -> Result<TokenStream> {
    let mut target: ItemStruct = syn::parse2(target)?;
    if target.semi_token.is_some() {
        return Err(Error::new(Span::call_site(), "#[Query] doesn't support unit / tuple struct !"))
    }
    if target.generics.lifetimes().count() >= 2 {
        return Err(Error::new(Span::call_site(), "#[Query] doesn't suport more than one lifetimes !"))
    }

    let name = &target.ident;
    let generics_params = &target.generics.params;
    let generics_where  = &target.generics.where_clause;

    let (
        from_request_lifetime,
        from_request_impl_additional_lifetime
    ) = match &target.generics.lifetimes().count() {
        0 => (
            GenericParam::Lifetime(LifetimeDef::new(
                Lifetime::new("'__from_request", Span::call_site())
            )),
            Some(quote!{
                '__from_request ,
            })
        ),
        1 => (
            target.generics.params.first().unwrap().clone(),
            None
        ),
        _ => return Err(syn::Error::new(Span::call_site(), "#[Query] doesn't support multiple lifetime params"))
    };

    let target_cloned = {
        let mut just_cloned = ItemStruct {
            ident: format_ident!("{}__cloned", target.ident),
            ..target.clone()
        };

        for attr in &mut just_cloned.attrs {
            if attr.path.get_ident().is_some_and(|i| i.to_string() == "query") {
                attr.path = syn::parse2(quote!{ serde }).unwrap();
            }
        }
        for field in &mut just_cloned.fields {
            for attr in &mut field.attrs {
                if attr.path.get_ident().is_some_and(|i| i.to_string() == "query") {
                    attr.path = syn::parse2(quote!{ serde }).unwrap();
                }
            }
        }

        target.attrs = target.attrs.iter()
            .filter(|a| !{a.path.get_ident().is_some_and(|i| i.to_string() == "query")})
            .cloned()
            .collect();
        for field in &mut target.fields {
            field.attrs = field.attrs.iter()
                .filter(|a| !{a.path.get_ident().is_some_and(|i| i.to_string() == "query")})
                .cloned()
                .collect()
        }

        just_cloned
    };
    let cloned_name = &target_cloned.ident;

    let set_fields = target.fields.iter()
        .map(|Field { ident, .. }| quote!{
            #ident: self.#ident,
        });

    Ok(quote!{
        #target

        const _: () = {
            #[allow(non_camel_case_types)]
            #[derive(::ohkami::serde::Deserialize)]
            #target_cloned

            impl<#generics_params> Into<#name<#generics_params>> for #cloned_name<#generics_params>
            where
                #generics_where
            {
                #[inline(always)]
                fn into(self) -> #name<#generics_params> {
                    #name {
                        #( #set_fields )*
                    }
                }
            }

            impl<
                #from_request_impl_additional_lifetime
                #generics_params
            > ::ohkami::FromRequest<#from_request_lifetime> for #name<#generics_params>
            where
                #generics_where
            {
                type Error = ::ohkami::Response;

                #[inline]
                fn from_request(req: &#from_request_lifetime ::ohkami::Request) -> Result<Self, Self::Error> {
                    let deserialized = req.query::<#cloned_name<#generics_params>>()
                        .map_err(|e| ::ohkami::Response::BadRequest().text(::std::format!("Unexpected query parameters: {e}")))?;
                    Ok(deserialized.into())
                }
            }
        };
    })
}
