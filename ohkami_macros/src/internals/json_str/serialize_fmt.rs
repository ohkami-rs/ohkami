use proc_macro2::TokenStream;
use quote::quote;

use super::{object::Object, JsonStr};

trait SerializeFmt {
    fn fmt(self) -> (String, TokenStream);
}

impl SerializeFmt for Object {
    fn fmt(self) -> (String, TokenStream) {
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
                        let (fmt, args) = obj.fmt();
                        map_str += &format!(r#""{key}":{fmt}"#);
                        args.extend(args)
                    },
                    other => map_str += &format!(r#""{key}":{other:?},"#),
                }
            }

            format!("{{{}}}", map_str.trim_end_matches(","))
        };

        (fmt, args)
    }
}
