use proc_macro2::{TokenStream, Ident};
use quote::{quote, ToTokens};
use syn::Type;

pub(crate) fn deserialize_struct(struct_name: &Ident, items: &Vec<(Ident, Type)>) -> TokenStream {
    let fields = {
        let mut fields = TokenStream::new();
        for (ident, _) in items {
            quote!{
                let mut #ident = None;
            }.to_tokens(&mut fields)
        }
        fields
    };

    let field_deserializers = {
        let mut field_deserializers = TokenStream::new();
        for (ident, ty) in items {
            let ident_str = ident.to_string();
            let ty_deserializer = deserialize_field(ty);
            quote!{
                #ident_str => {
                    string.next_if_eq(&':')?;
                    string.next_if_eq(&' ');
                    if #ident.replace(#ty_deserializer).is_some() {return None}
                    string.next_if_eq(&',');
                    string.next_if_eq(&' ');
                },
            }.to_tokens(&mut field_deserializers)
        }
        field_deserializers
    };

    let builder = {
        let mut validator = quote!{ string.next().is_none() };
        let mut builder = TokenStream::new();

        for (ident, _) in items {
            quote!{
                && #ident.is_some()
            }.to_tokens(&mut validator);

            quote!{
                #ident: #ident.unwrap(),
            }.to_tokens(&mut builder);
        }

        quote!{
            return (
                #validator
            ).then(|| #struct_name {
                #builder
            })
        }
    };

    quote!{
        #fields

        string.next_if_eq(&'{')?;
        loop {
            match string.peek()? {
                '}' => {
                    string.next();
                    #builder
                },
                _ => match 'string: {
                    string.next_if_eq(&'"')?;
                    let mut ret = String::new();
                    while let Some(ch) = string.next() {
                        match ch {
                            '"' => break 'string Some(ret),
                            _ => ret.push(ch),
                        }
                    }
                    None
                }?.as_str() {
                    #field_deserializers
                    _ => return None
                },
            }
        }
    }
}

fn deserialize_field(ty: &Type) -> TokenStream {
    let ty_str = ty.to_token_stream().to_string();
    match ty_str.as_str() {
        "String"|"Vec<String>"|"bool"|"Vec<bool>"|
        "u8"|"u16"|"u32"|"u64"|"u128"|"usize"|
        "i8"|"i16"|"i32"|"i64"|"i128"|"isize"|
        "Vec<u8>"|"Vec<u16>"|"Vec<u32>"|"Vec<u64>"|"Vec<u128>"|"Vec<usize>"|
        "Vec<i8>"|"Vec<i16>"|"Vec<i32>"|"Vec<i64>"|"Vec<i128>"|"Vec<isize>" => quote!{
            <#ty as ohkami_json::Deserialize>::_deserialize(string)?,
        },

        _ => quote!{
            <#ty as ohkami_json::JSON>::_deserialize(string)?,
        }
    }
}