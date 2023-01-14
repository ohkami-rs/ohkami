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
                        map_str += &format!(r#""{key}":{{}},"#);
                        args.extend(quote!{
                            #name.ser()?,
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

            format!("{{{{{}}}}}", map_str.trim_end_matches(","))
        };

        (fmt, args)
    }
}

impl SerializeFmt for Vec<JsonStr> {
    fn serialize_fmt(self) -> (String, TokenStream) {
        let mut args = TokenStream::new();
        let fmt = {
            let mut elems_str = String::new();

            for elem in self {
                match elem {
                    JsonStr::Var(name) => {
                        elems_str += "{},";
                        args.extend(quote!{
                            #name,
                        })
                    },
                    JsonStr::Object(obj) => {
                        let (fmt, obj_args) = obj.serialize_fmt();
                        let fmt = &fmt[1..fmt.len()-1];
                        elems_str += &format!("{fmt},");
                        args.extend(obj_args)
                    },
                    other => elems_str += &format!("{other:?},"),
                }
            }

            format!("[{}]", elems_str.trim_end_matches(","))
        };

        (fmt, args)
    }
}
