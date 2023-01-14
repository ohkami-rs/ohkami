use quote::quote;
use crate::internals::Build;
use super::{JsonStr, number::Number, serialize_fmt::SerializeFmt};

impl Build for JsonStr {
    fn build(self) -> proc_macro2::TokenStream {
        match self {
            Self::Bool(boolean) => {
                if boolean {quote!("true")} else {quote!("false")}
            },
            Self::Num(number) => {
                let num_str = match number {
                    Number::Positive(p) => p.to_string(),
                    Number::Negative(n) => n.to_string(),
                    Number::Float(f) => f.to_string(),
                };
                quote!(#num_str)
            },
            Self::Str(string) => {
                let quoted_str = format!(r#"{string}"#);
                quote!(#quoted_str)
            },
            Self::Var(var) => quote!{
                format!("{:?}", #var)
            },
            Self::Array(array) => {
                let array_str = format!("{array:?}");
                quote!(#array_str)
            },
            Self::Object(object) => {
                let (fmt, args) = object.serialize_fmt();
                quote!{
                    format!(#fmt, #args)
                }
            },
        }
    }
}