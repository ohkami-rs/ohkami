use std::collections::BTreeMap;

use syn::{parse::Parse, token, Ident, Lit, braced, punctuated::Punctuated };
use super::{JsonStr, number::Number, object::Object};

impl Parse for JsonStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(token::Brace) {
            let map;
            braced!(map in input);
            let key_values: Punctuated<KeyValue, token::Comma> = map.parse_terminated(KeyValue::parse)?;

            let mut object = Object(BTreeMap::new());
            for KeyValue { key, _colon, value } in key_values {
                object.0.insert(key, value);
            }

            Ok(Self::Object(object))
        } else if input.peek(Ident) {
            let ident = input.parse::<Ident>()?;
            Ok(Self::Var(ident))
        } else if input.peek(token::Sub) {
            input.parse::<token::Sub>()?;
            match input.parse::<Lit>()? {
                Lit::Float(float) => Ok(Self::Num(Number::Float(-float.base10_parse::<f64>().unwrap()))),
                Lit::Int(integer) => Ok(Self::Num(Number::Negative(-integer.base10_parse::<isize>().unwrap()))),
                _ => Err(input.error("Expected float or integer literal"))
            }
        } else if input.peek(Lit) {
            match input.parse::<Lit>()? {
                Lit::Bool(boolean) => Ok(Self::Bool(boolean.value)),
                Lit::Float(float) => Ok(Self::Num(Number::Float(float.base10_parse().unwrap()))),
                Lit::Int(integer) => Ok(Self::Num(Number::Positive(integer.base10_parse().unwrap()))),
                Lit::Str(string) => Ok(Self::Str(string.value())),
                _ => Err(input.error("Expected one of bool, float, int or str lieral"))
            }
        } else {
            Err(input.error("Expected one of literal, varialble, object (map)"))
        }
    }
}

struct KeyValue {
    key:    String,
    _colon: token::Colon,
    value:  JsonStr,
}
impl Parse for KeyValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse::<Lit>()?;
        let key = match key {
            Lit::Str(s) => s.value(),
            _ => return Err(input.error("expected string literal as key"))
        };

        Ok(Self {
            key,
            _colon: input.parse()?,
            value:  input.parse()?,
        })
    }
}