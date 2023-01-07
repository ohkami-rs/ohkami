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
                && #ident.is_none()
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
                            '"' => break 'string ret,
                            _ => ret.push(ch),
                        }
                    }
                }.as_str() {
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
        "String" => quote!{
            'string: {
                string.next_if_eq(&'"')?;
                let mut ret = String::new();
                while let Some(ch) = string.next() {
                    match ch {
                        '"' => break 'string ret,
                        _ => ret.push(ch),
                    }
                }
            },
        },
        "Vec<String>" => quote!{
            'vec_string: {
                string.next_if_eq(&'[')?;
                let mut ret = Vec::new();
                loop {
                    match string.peek() {
                        Some(']') => {string.next(); break 'vec_string ret},
                        Some(' ') => {string.next();},
                        _ => {
                            ret.push('string: {
                                string.next_if_eq(&'"')?;
                                let mut ret = String::new();
                                while let Some(ch) = string.next() {
                                    match ch {
                                        '"' => break 'string ret,
                                        _ => ret.push(ch),
                                    }
                                }
                            });
                            string.next_if_eq(&',');
                        }
                    }
                }
            },
        },
        "bool" => quote!{
            match string.next() {
                Some('t') => (
                    string.next() == Some('r') &&
                    string.next() == Some('u') &&
                    string.next() == Some('e')
                ).then_some(true)?,
                Some('f') => (
                    string.next() == Some('a') &&
                    string.next() == Some('l') &&
                    string.next() == Some('s') &&
                    string.next() == Some('e')
                ).then_some(false)?,
                _ => return None
            }?,
        },
        "Vec<bool>" => quote!{
            'vec_bool: {
                string.next_if_eq(&'[')?;
                let mut ret = Vec::new();
                loop {
                    match string.peek() {
                        Some(']') => {string.next(); break 'vec_bool ret},
                        Some(' ') => {string.next();},
                        _ => {
                            ret.push(match string.next() {
                                Some('t') => (
                                    string.next() == Some('r') &&
                                    string.next() == Some('u') &&
                                    string.next() == Some('e')
                                ).then_some(true)?,
                                Some('f') => (
                                    string.next() == Some('a') &&
                                    string.next() == Some('l') &&
                                    string.next() == Some('s') &&
                                    string.next() == Some('e')
                                ).then_some(false)?,
                                _ => return None
                            });
                            string.next_if_eq(&',');
                        }
                    }
                }
            },
        },
        "u8"|"u16"|"u32"|"u64"|"u128"|"usize"|
        "i8"|"i16"|"i32"|"i64"|"i128"|"isize" => quote!{
            {
                let mut int_str = String::new();
                while let Some(ch) = string.peek() {
                    match ch {
                        '0'..='9' => int_str.push(string.next().unwrap()),
                        _ => return None,
                    }
                }
                int_str.parse::<#ty>().ok()?
            },
        },
        "Vec<u8>"|"Vec<u16>"|"Vec<u32>"|"Vec<u64>"|"Vec<u128>"|"Vec<usize>"|
        "Vec<i8>"|"Vec<i16>"|"Vec<i32>"|"Vec<i64>"|"Vec<i128>"|"Vec<isize>" => quote!{
            'vec_int: {
                string.next_if_eq(&'[')?;
                let mut ret = Vec::new();
                loop {
                    match string.peek() {
                        Some(']') => {string.next(); break 'vec_int ret},
                        Some(' ') => {string.next();},
                        _ => {
                            ret.push({
                                let mut int_str = String::new();
                                while let Some(ch) = string.peek() {
                                    match ch {
                                        '0'..='9' => int_str.push(string.next().unwrap()),
                                        _ => return None,
                                    }
                                }
                                int_str.parse::<#ty>().ok()?
                            });
                            string.next_if_eq(&',');
                        }
                    }
                }
            },
        },

        _ => quote!{
            <#ty as JSON>::_deserialize(&mut string)?,
        }
    }
}