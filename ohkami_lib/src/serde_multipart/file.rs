/// File in `multipart/form-data` requests
/// 
/// *expected_usage.rs*
/// 
/// ---
/// ```ignore
/// #[derive(Deserialize)]
/// struct SignUpForm<'req> {
///     #[serde(rename = "user-name")]
///     user_name:  &'req str,
/// 
///     password:   &'req str,
///     
///     #[serde(rename = "user-icon")]
///     user_icon:  Option<File<'req>>,
/// 
///     #[serde(rename = "pet-photos")]
///     pet_photos: Vec<File<'req>>,
/// }
/// 
/// async fn sign_up(
///     Multipart(form): Multipart<SignUpForm<'_>>,
///     pool: Context<'_, PgPool>,
/// ) -> Result<String, APIError> {
///     //...
/// }
/// ```
/// ---
#[derive(serde::Deserialize)]
#[derive(Debug, PartialEq)]
pub struct File<'req> {
    pub filename: &'req str,
    pub mimetype: &'req str,
    pub content:  &'req [u8],
}




const _: () = {
    use super::Error;
    use serde::de::IntoDeserializer;


    impl<'de> IntoDeserializer<'de, Error> for File<'de> {
        type Deserializer = FileDeserializer<'de>;

        fn into_deserializer(self) -> Self::Deserializer {
            FileDeserializer {
                file:  self,
                state: FileField::init(),
            }
        }
    }

    pub struct FileDeserializer<'de> {
        file:  File<'de>,
        state: FileField,
    }
    #[allow(non_camel_case_types)]
    enum FileField {
        filename,
        mimetype,
        content,
        __finished
    } impl FileField {
        const fn init() -> Self {
            Self::filename
        }
        fn step(&mut self) {
            match &self {
                Self::filename => *self = Self::mimetype,
                Self::mimetype => *self = Self::content,
                Self::content  => *self = Self::__finished,
                Self::__finished => ()
            }
        }
    }

    impl<'de> serde::de::Deserializer<'de> for FileDeserializer<'de> {
        type Error = Error;

        fn deserialize_struct<V>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            self.deserialize_map(visitor)
        }
        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            visitor.visit_map(self)
        }

        fn deserialize_newtype_struct<V>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            visitor.visit_newtype_struct(self)
        }

        serde::forward_to_deserialize_any! {
            i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
            str string char bytes byte_buf bool
            seq option enum identifier
            unit unit_struct tuple tuple_struct
            ignored_any
        }
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            visitor.visit_map(self)
        }
    }

    impl<'de> serde::de::MapAccess<'de> for FileDeserializer<'de> {
        type Error = Error;

        fn next_entry_seed<K, V>(
            &mut self,
            kseed: K,
            vseed: V,
        ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where
            K: serde::de::DeserializeSeed<'de>,
            V: serde::de::DeserializeSeed<'de>,
        {
            let (k, v) = match &self.state {
                FileField::filename => (
                    kseed.deserialize("filename".into_deserializer())?,
                    vseed.deserialize(serde::de::value::BorrowedStrDeserializer::new(self.file.filename))?,
                ),
                FileField::mimetype => (
                    kseed.deserialize("mimetype".into_deserializer())?,
                    vseed.deserialize(serde::de::value::BorrowedStrDeserializer::new(self.file.mimetype))?,
                ),
                FileField::content => (
                    kseed.deserialize("content".into_deserializer())?,
                    vseed.deserialize(serde::de::value::BorrowedBytesDeserializer::new(self.file.content))?,
                ),
                FileField::__finished => return Ok(None)
            };

            self.state.step();

            Ok(Some((k, v)))
        }

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: serde::de::DeserializeSeed<'de> {
            let k = match &self.state {
                FileField::filename => seed.deserialize("filename".into_deserializer())?,
                FileField::mimetype => seed.deserialize("mimetype".into_deserializer())?,
                FileField::content  => seed.deserialize("content".into_deserializer())?,
                FileField::__finished => return Ok(None)
            };

            Ok(Some(k))
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: serde::de::DeserializeSeed<'de> {
            let v = match &self.state {
                FileField::filename => seed.deserialize(serde::de::value::BorrowedStrDeserializer::new(self.file.filename))?,
                FileField::mimetype => seed.deserialize(serde::de::value::BorrowedStrDeserializer::new(self.file.mimetype))?,
                FileField::content  => seed.deserialize(serde::de::value::BorrowedBytesDeserializer::new(self.file.content))?,
                FileField::__finished => unsafe {std::hint::unreachable_unchecked(/* already checked in `next_key_seed` */)}
            };

            self.state.step();

            Ok(v)
        }
    }
};
