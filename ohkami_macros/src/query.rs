use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{parse_str, Error, GenericParam, ItemStruct, Lifetime, LifetimeDef, Result, Type};


#[allow(non_snake_case)]
pub(super) fn Query(target: TokenStream) -> Result<TokenStream> {
    let target: ItemStruct = syn::parse2(target)?;

    if target.semi_token.is_some() {
        return Err(Error::new(Span::call_site(), "#[Query] doesn't support unit / tuple struct !"))
    }

    let impl_from_request = {
        let struct_name = &target.ident;

        let (impl_lifetime, struct_lifetime) = match &target.generics.lifetimes().count() {
            0 => (
                GenericParam::Lifetime(LifetimeDef::new(
                    Lifetime::new("'__impl_from_request_lifetime", Span::call_site())
                )),
                None,
            ),
            1 => (
                target.generics.params.first().unwrap().clone(),
                Some(target.generics.params.first().unwrap().clone()),
            ),
            _ => return Err(syn::Error::new(Span::call_site(), "#[Query] doesn't support multiple lifetime params"))
        };

        let fields = target.fields.iter().map(|f| {
            let field_name = f.ident.as_ref().unwrap(/* already checked in `parse_request_struct` */);
            let field_name_str = field_name.to_string();
            let field_type = &f.ty;
            let field_type_str = field_type.to_token_stream().to_string();

            if field_type_str.starts_with("Option") {
                let inner_type = parse_str::<Type>(field_type_str.strip_prefix("Option <").unwrap().strip_suffix(">").unwrap()).unwrap();
                quote!{
                    #field_name: req.query::<#inner_type>(#field_name_str) // Option<Result<_>>
                        .transpose()
                        .map_err(|e| ::ohkami::Response::InternalServerError().text(e.to_string()))?,
                }
            } else {
                quote!{
                    #field_name: req.query::<#field_type>(#field_name_str) // Option<Result<_>>
                        .ok_or_else(|| ::ohkami::Response::BadRequest().text(
                            concat!("Expected query parameter `", #field_name_str, "`")
                        ))?
                        .map_err(|e| ::ohkami::Response::InternalServerError().text(e.to_string()))?,
                }
            } 
        });
        
        quote!{
            impl<#impl_lifetime> ::ohkami::FromRequest<#impl_lifetime> for #struct_name<#struct_lifetime> {
                type Error = ::ohkami::Response;
                #[inline] fn from_request(req: &#impl_lifetime ::ohkami::Request) -> ::std::result::Result<Self, Self::Error> {
                    ::std::result::Result::Ok(Self {
                        #( #fields )*
                    })
                }
            }
        }
    };

    Ok(quote!{
        #target
        #impl_from_request
    })
}
