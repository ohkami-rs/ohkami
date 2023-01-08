use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::Type;

pub(crate) fn serialize_process(items: &Vec<(Ident, Type)>) -> TokenStream {
    let mut items = items.clone();
    let mut add_serialized_fields = TokenStream::new();

    let (ident_1, ty_1) = &items.pop().unwrap();
    let key_1 = format!(r#""{ident_1}":"#);
    let value_1 = serializing_expr(ident_1, ty_1);
    quote!{s += #key_1;}.to_tokens(&mut add_serialized_fields);
    quote!{s += &#value_1;}.to_tokens(&mut add_serialized_fields);

    for (ident, ty) in &items {
        let key = format!(r#","{ident}":"#);
        let value = serializing_expr(ident, ty);
        quote!{s += #key;}.to_tokens(&mut add_serialized_fields);
        quote!{s += &#value;}.to_tokens(&mut add_serialized_fields);
    }

    quote!{
        let mut s = String::from("{");
        #add_serialized_fields
        s + "}"
    }
}

pub(crate) fn serializing_expr(ident: &Ident, ty: &Type) -> TokenStream {
    let ty_str = ty.into_token_stream().to_string();
    match ty_str.as_str() {
        "u8"|"u16"|"u32"|"u64"|"u128"|"usize"|
        "i8"|"i16"|"i32"|"i64"|"i128"|"isize"|
        "Vec<u8>"|"Vec<u16>"|"Vec<u32>"|"Vec<u64>"|"Vec<u128>"|"Vec<usize>"|
        "Vec<i8>"|"Vec<i16>"|"Vec<i32>"|"Vec<i64>"|"Vec<i128>"|"Vec<isize>"|
        "bool"|"Vec<bool>"|
        "String"|"<&str>"|"Vec<String>"|"Vec<&str>" => quote!{
            <#ty as ohkami_json::Serialize>::serialize(&self.#ident)
        },
        _ => quote!{
            <#ty as JSON>::serialize(&self.#ident)
        }
    }
}


#[cfg(test)]
mod test {
    use quote::{format_ident, quote};
    use syn::Type;

    use super::serializing_expr;

    #[test]
    fn test_serializing_expr() {
        let case = serializing_expr(
            &format_ident!("id"),
            &Type::Verbatim(quote!{ u64 })
        );
        assert_eq!(case.to_string(), "self . id . to_string () ,")
    }
}
