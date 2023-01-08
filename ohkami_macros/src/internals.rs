use proc_macro2::TokenStream;
use quote::quote;

#[allow(non_snake_case)]
pub(super) fn JSON(struct_stream: TokenStream) -> TokenStream {
    quote!{
        #[derive(serde::Serialize, serde::Deserialize, ohkami::components::json::Json)]
        #struct_stream
    }
}

/*

use syn::Error;
mod ohkami_json; use self::ohkami_json::{
    serialize::serialize_process,
    deserialize::deserialize_struct,
};

pub(super) fn derive_ohkami_json(struct_stream: TokenStream) -> Result<TokenStream, Error> {
    let struct_stream = syn::parse2::<syn::ItemStruct>(struct_stream)?;
    let ident = struct_stream.ident;

    if struct_stream.semi_token.is_some() {
        unimplemented!()
    } else {
        let fields = struct_stream.fields;
        let items = {
            let mut items = Vec::with_capacity(fields.len());
            for field in fields {items.push((field.ident.unwrap(), field.ty));}
            items
        };

        let serialize_process = serialize_process(&items);
        let deserialized_struct = deserialize_struct(&ident, &items);

        Ok(quote!{
            impl JSON for #ident {
                fn serialize(&self) -> String {
                    #serialize_process
                }
                fn _deserialize(string: &mut std::iter::Peekable<std::str::Chars>) -> Option<Self> {
                    #deserialized_struct
                }
            }
        })
    }
}

*/