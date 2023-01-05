use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::Type;

pub(crate) fn serialize_fmt_and_args(items: &Vec<(Ident, Type)>) -> (TokenStream, TokenStream) {
    let fmt_raw = {
        let mut fmt_raw = String::new();
        for (ident, _) in items {
            fmt_raw += &format!(r#""{ident}":{{}},"#)
        }
        fmt_raw.trim_end_matches(',').to_owned()
    };
    let fmt = {
        let mut fmt = TokenStream::new();
        quote!(r).to_tokens(&mut fmt);
        quote!(#).to_tokens(&mut fmt);
        quote!(#fmt_raw).to_tokens(&mut fmt);
        quote!(#).to_tokens(&mut fmt);
        fmt
    };

    let args = {
        let mut args = TokenStream::new();
        for (ident, ty) in items {
            serializing_expr(ident, ty).to_tokens(&mut args);
        }
        args
    };

    (fmt, args)
}

pub(crate) fn serializing_expr(ident: &Ident, ty: &Type) -> TokenStream {
    let ty_str = ty.into_token_stream().to_string();
    match ty_str.as_str() {
        "u8"|"u16"|"u32"|"u64"|"u128"|"usize"|
        "i8"|"i16"|"i32"|"i64"|"i128"|"isize"|
        "bool" => quote!{
            self.#ident.to_string(),
        },
        "String"|"&str" => quote!{
            format!(r#""{}""#, self.#ident),
        },
        "Vec<u8>"|"Vec<u16>"|"Vec<u32>"|"Vec<u64>"|"Vec<u128>"|"Vec<usize>"|
        "Vec<i8>"|"Vec<i16>"|"Vec<i32>"|"Vec<i64>"|"Vec<i128>"|"Vec<isize>"|
        "Vec<bool>" => quote!{
            {
                let mut s = self.#ident.into_iter().fold(
                    String::from("["),
                    |it, next| it + &next.to_string() + ","
                );
                s.pop(); s + "]"
            },
        },
        "Vec<String>"|"Vec<&str>" => quote!{
            {
                let mut s = self.#ident.into_iter().fold(
                    String::from("["),
                    |it, next| it + &format!(r#""{}""#, next) + ","
                );
                s.pop(); s + "]"
            },
        },
        _ => quote!{
            <ty as JSON>::serialize(&self.#ident),
        }
    }
}
