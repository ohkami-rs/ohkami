use proc_macro2::TokenStream;
use quote::quote;

use super::{object::Object, JsonStr};

pub(crate)trait SerializeFmt {
    fn serialize_fmt(self) -> (String, TokenStream);
}

impl SerializeFmt for Object {
    fn serialize_fmt(mut self) -> (String, TokenStream) {
        let mut args = TokenStream::new();
        let fmt = {
            let mut map_str = String::new();

            while let Some((key, value)) = self.0.pop_first() {
                match value {
                    JsonStr::Var(name) => {
                        map_str += &format!(r#""{key}":{{:?}},"#);
                        args.extend(quote!{
                            #name,
                        })
                    },
                    JsonStr::Object(obj) => {
                        let (fmt, new_args) = obj.serialize_fmt();
                        map_str += &format!(r#""{key}":{fmt}"#);
                        args.extend(new_args)
                    },
                    other => map_str += &format!(r#""{key}":{other:?},"#),
                }
            }

            format!("{{{}}}", map_str.trim_end_matches(","))
        };

        (fmt, args)
    }
}
