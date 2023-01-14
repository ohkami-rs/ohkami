use quote::quote;
use crate::internals::Build;
use super::{JsonStr, number::Number, serialize_fmt::SerializeFmt};

impl Build for JsonStr {
    fn build(self) -> proc_macro2::TokenStream {
        match self {
            Self::Bool(boolean) => {
                if boolean {quote!{
                    std::borrow::Cow::Borrowed("true")
                }} else {quote!{
                    std::borrow::Cow::Borrowed("false")
                }}
            },
            Self::Num(number) => {
                let num_str = match number {
                    Number::Positive(p) => p.to_string(),
                    Number::Negative(n) => n.to_string(),
                    Number::Float(f) => f.to_string(),
                };
                quote!{
                    std::borrow::Cow::Borrowed(#num_str)
                }
            },
            Self::Str(string) => {
                let quoted_str = format!(r#""{string}""#);
                quote!{
                    std::borrow::Cow::Borrowed(#quoted_str)
                }
            },
            Self::Var(var) => quote!{
                std::borrow::Cow::Owned(ohkami::components::json::ser(#var)?)
            },
            Self::Array(array) => {
                let (fmt, args) = array.serialize_fmt();
                if args.is_empty() {
                    quote!{
                        std::borrow::Cow::Borrowed(#fmt)
                    }
                } else {
                    quote!{
                        std::borrow::Cow::Owned(format!(#fmt, #args))
                    }
                }
            },
            Self::Object(object) => {
                let (fmt, args) = object.serialize_fmt();
                if args.is_empty() {
                    let fmt = &fmt[1..fmt.len()-1];
                    quote!{
                        std::borrow::Cow::Borrowed(#fmt)
                    }
                } else {
                    quote!{
                        std::borrow::Cow::Owned(format!(#fmt, #args))
                    }
                }
            },
        }
    }
}
