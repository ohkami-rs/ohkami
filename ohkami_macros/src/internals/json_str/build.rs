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
                let quoted_str = format!(r#""{string}""#);
                quote!(#quoted_str)
            },
            Self::Var(var) => quote!{
                #var.ser()?
            },
            Self::Array(array) => {
                let (fmt, args) = array.serialize_fmt();
                if args.is_empty() {
                    quote!{
                        #fmt
                    }
                } else {
                    quote!{
                        format!(#fmt, #args)
                    }
                }
            },
            Self::Object(object) => {
                let (fmt, args) = object.serialize_fmt();
                if args.is_empty() {
                    let fmt = &fmt[1..fmt.len()-1];
                    quote!{
                        #fmt
                    }
                } else {
                    quote!{
                        format!(#fmt, #args)
                    }
                }
            },
        }
    }
}