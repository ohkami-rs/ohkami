use proc_macro2::TokenStream;
use syn::Error;
use quote::quote;
use self::json::serialize::serialize_fmt_and_args;

mod json;

pub(super) fn derive_json(struct_stream: TokenStream) -> Result<TokenStream, Error> {
    let struct_stream = syn::parse2::<syn::ItemStruct>(struct_stream)?;
    let ident = struct_stream.ident;

    if struct_stream.semi_token.is_some() {
        
        Ok(quote!(
            impl JSON for #ident {
                fn serialize(&self) -> String {

                }

                fn _deserialize(&str) -> Self {

                }
            }
        ))
    } else {
        /*
            struct User {
                id:   u64,
                name: String,
            }

            impl JSON for User {
                fn serialize(&self) -> String {
                    format!(r#"{{"id":{},"name":{}}}"#,
                        <u64 as Serialize>::serialize(&self.id),
                        <String as Serialize>::serialize(&self.name),
                    )
                }
        */
        let fields = struct_stream.fields;
        let items = {
            let mut items = Vec::with_capacity(fields.len());
            for field in fields {items.push((field.ident.unwrap(), field.ty));}
            items
        };

        let (serialize_fmt, serialize_args) = serialize_fmt_and_args(&items);

        Ok(quote!{
            impl JSON for #ident {
                fn serialize(&self) -> String {
                    format!(#serialize_fmt, #serialize_args)
                }
                fn de(string: &str) -> Self {
                    
                }
            }
        })
    }
}
