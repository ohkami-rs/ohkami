use serde::{Serialize, Deserialize};
use crate::typed::{Payload, PayloadType};


pub struct JSON;
impl PayloadType for JSON {
    const CONTENT_TYPE: &'static str = "application/json";
    
    #[inline(always)]
    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        ::serde_json::from_slice(bytes)
    }

    #[inline(always)]
    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        ::serde_json::to_vec(&value)
    }
}

const _: (/* JSON payload utitlity impls */) = {
    impl<P: Payload<Type = JSON>> Payload for Option<P> {
        type Type = JSON;
    }

    impl<P: Payload<Type = JSON>> Payload for Vec<P> {
        type Type = JSON;
    }

    impl<P: Payload<Type = JSON>> Payload for &[P] {
        type Type = JSON;
    }

    impl<P: Payload<Type = JSON>, const N: usize> Payload for [P; N] {
        type Type = JSON;
    }
};

const _: (/* builtin impls */) = {
    impl Payload for ::serde_json::Value {
        type Type = JSON;
    }
};
