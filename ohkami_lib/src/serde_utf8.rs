mod ser;
mod de;

#[cfg(test)]
mod _test;


#[inline(always)]
pub fn to_string(value: &impl serde::Serialize) -> Result<String, Error> {
    let mut s = ser::UTF8Serializer { output: String::new() };
    value.serialize(&mut s)?;
    Ok(s.output)
}

#[inline(always)]
pub fn from_str<'de, D: serde::Deserialize<'de>>(input: &'de str) -> Result<D, Error> {
    let mut d = de::UTF8Deserializer { input };
    let t = D::deserialize(&mut d)?;
    if d.input.is_empty() {
        Ok(t)
    } else {
        Err((||serde::de::Error::custom(format!("Unexpected trailing charactors: {}", d.input)))())
    }
}


#[derive(Debug)]
pub struct Error(String);
const _: () = {
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for Error {}

    impl serde::ser::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
    impl serde::de::Error for Error {
        fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
            Self(msg.to_string())
        }
    }
};

pub(crate) enum Infallible {}
const _: () = {
    impl serde::ser::SerializeMap for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_key<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn serialize_value<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeSeq for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_element<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeStruct for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeStructVariant for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, _: &'static str, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeTuple for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_element<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }

    impl serde::ser::SerializeTupleStruct for Infallible {
        type Ok    = ();
        type Error = Error;

        fn serialize_field<T: ?Sized>(&mut self, _: &T) -> Result<(), Self::Error>
        where T: serde::Serialize {match *self {}}

        fn end(self) -> Result<Self::Ok, Self::Error> {match self {}}
    }
};
