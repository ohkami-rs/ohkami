use super::_util::{Map, is_false};
use serde::Serialize;
use serde_json::Value;
use std::marker::PhantomData;

pub struct Schema<T: SchemaType> {
    datatype: PhantomData<fn() -> T>,
    raw: RawSchema,
}

use Type::SchemaType;
pub mod Type {
    pub struct string;
    pub struct number;
    pub struct integer;
    pub struct bool;
    pub struct array;
    pub struct object;
    pub struct any;

    #[allow(private_bounds)]
    pub trait SchemaType: Sealed {
        const NAME: &'static str;
    }
    trait Sealed {
        const NAME: &'static str;
    }
    impl<S: Sealed> SchemaType for S {
        const NAME: &'static str = <S as Sealed>::NAME;
    }

    impl Sealed for string {
        const NAME: &'static str = "string";
    }
    impl Sealed for number {
        const NAME: &'static str = "number";
    }
    impl Sealed for integer {
        const NAME: &'static str = "integer";
    }
    impl Sealed for bool {
        const NAME: &'static str = "bool";
    }
    impl Sealed for array {
        const NAME: &'static str = "array";
    }
    impl Sealed for object {
        const NAME: &'static str = "object";
    }
    impl Sealed for any {
        const NAME: &'static str = "";
    }
}

#[derive(Serialize, PartialEq, Clone)]
pub struct RawSchema {
    #[serde(skip)]
    pub(crate) __name__: Option<&'static str>,

    #[serde(rename = "type", skip_serializing_if = "str::is_empty")]
    datatype: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    #[serde(rename = "anyOf")]
    any_of: Vec<SchemaRef>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    #[serde(rename = "allOf")]
    all_of: Vec<SchemaRef>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    #[serde(rename = "oneOf")]
    one_of: Vec<SchemaRef>,

    /* metadata and flags */
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<Value>,
    #[serde(rename = "enum", skip_serializing_if = "<[_]>::is_empty")]
    enumerates: Vec<Value>,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,
    #[serde(skip_serializing_if = "is_false")]
    nullable: bool,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(rename = "readOnly")]
    read_only: bool,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(rename = "writeOnly")]
    write_only: bool,

    /* string definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<&'static str>,

    /* object definition */
    #[serde(skip_serializing_if = "Map::is_empty")]
    properties: Map<&'static str, SchemaRef>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    required: Vec<&'static str>,

    /* array definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<SchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxItems")]
    max_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minItems")]
    min_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxProperties")]
    max_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxProperties")]
    min_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxLength")]
    max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxLength")]
    min_length: Option<usize>,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(rename = "uniqueItems")]
    unique_items: bool,

    /* number,integer definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "multipleOf")]
    multiple_of: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum: Option<f64>,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(rename = "exclusiveMaximum")]
    exclusive_maximum: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    minimum: Option<f64>,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(rename = "exclusiveMinimum")]
    exclusive_minimum: bool,
}
impl<T: Type::SchemaType> From<Schema<T>> for RawSchema {
    fn from(schema: Schema<T>) -> Self {
        schema.raw
    }
}
impl<T: Type::SchemaType> From<RawSchema> for Schema<T> {
    fn from(raw: RawSchema) -> Self {
        Self {
            raw,
            datatype: PhantomData,
        }
    }
}
impl From<RawSchema> for SchemaRef {
    fn from(this: RawSchema) -> SchemaRef {
        SchemaRef::Inline(Box::new(this))
    }
}
impl RawSchema {
    #[doc(hidden)]
    pub fn into_properties(self) -> Vec<(&'static str, SchemaRef, bool)> {
        self.properties
            .into_iter()
            .map(|(name, schema)| (name, schema, self.required.contains(&name)))
            .collect()
    }
}

#[derive(PartialEq, Clone)]
#[allow(private_interfaces/* construct only via `From` */)]
pub enum SchemaRef {
    Inline(Box<RawSchema>),
    Reference(&'static str),
}
impl Serialize for SchemaRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SchemaRef::Inline(schema) => schema.serialize(serializer),
            SchemaRef::Reference(name) => {
                use serde::ser::SerializeMap;
                let mut s = serializer.serialize_map(None)?;
                s.serialize_entry("$ref", &format!("#/components/schemas/{name}"))?;
                s.end()
            }
        }
    }
}
impl<T: SchemaType> From<Schema<T>> for SchemaRef {
    fn from(schema: Schema<T>) -> Self {
        SchemaRef::Inline(Box::new(schema.raw))
    }
}
impl From<&'static str> for SchemaRef {
    fn from(name: &'static str) -> Self {
        SchemaRef::Reference(name)
    }
}
impl SchemaRef {
    pub fn into_inline(self) -> Option<RawSchema> {
        match self {
            SchemaRef::Inline(raw) => Some(*raw),
            SchemaRef::Reference(_) => None,
        }
    }

    pub(crate) fn refize(&mut self) -> impl Iterator<Item = RawSchema> {
        let mut component_schemas = vec![];
        if let SchemaRef::Inline(raw) = self {
            #[allow(clippy::option_map_unit_fn)]
            raw.items
                .as_mut()
                .map(|s| component_schemas.extend(s.refize()));
            raw.properties
                .values_mut()
                .for_each(|s| component_schemas.extend(s.refize()));
            raw.any_of
                .iter_mut()
                .for_each(|s| component_schemas.extend(s.refize()));
            raw.all_of
                .iter_mut()
                .for_each(|s| component_schemas.extend(s.refize()));
            raw.one_of
                .iter_mut()
                .for_each(|s| component_schemas.extend(s.refize()));
            if let Some(name) = raw.__name__ {
                let raw = std::mem::replace(self, SchemaRef::Reference(name));
                component_schemas.push(raw.into_inline().unwrap());
            }
        }
        component_schemas.into_iter()
    }
}

impl<T: Type::SchemaType> Schema<T> {
    pub fn component(name: &'static str, mut schema: Self) -> Self {
        schema.raw.__name__ = Some(name);
        schema
    }
}

const _: (/* constructors */) = {
    const ANY: RawSchema = RawSchema {
        __name__: None,

        datatype: Type::any::NAME,
        format: None,
        any_of: Vec::new(),
        all_of: Vec::new(),
        one_of: Vec::new(),

        /* metadata and flags */
        description: None,
        default:     None,
        example:     None,
        enumerates:  Vec::new(),
        deprecated:  false,
        nullable:    false,
        read_only:    false,
        write_only:   false,

        /* string definition */
        pattern: None,

        /* object definition */
        properties: Map::new(),
        required:   Vec::new(),

        /* array definition */
        items:         None,
        max_items:      None,
        min_items:      None,
        max_properties: None,
        min_properties: None,
        max_length:     None,
        min_length:     None,
        unique_items:   false,

        /* number,integer definition */
        multiple_of:       None,
        maximum:          None,
        exclusive_maximum: false,
        minimum:          None,
        exclusive_minimum: false,
    };

    impl Schema<Type::string> {
        pub fn string() -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema { datatype: Type::string::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::number> {
        pub fn number() -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema { datatype: Type::number::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::integer> {
        pub fn integer() -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema { datatype: Type::integer::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::bool> {
        pub fn bool() -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema { datatype: Type::bool::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::array> {
        pub fn array(items: impl Into<SchemaRef>) -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema {
                    datatype: Type::array::NAME,
                    items:    Some(items.into()),
                    ..ANY
                }
            }
        }
    }
    impl Schema<Type::object> {
        pub fn object() -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema { datatype: Type::object::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::any> {
        pub fn any_of(schemas: impl SchemaList) -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema {
                    any_of: SchemaList::collect(schemas),
                    ..ANY
                }
            }
        }
        pub fn all_of(schemas: impl SchemaList) -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema {
                    all_of: SchemaList::collect(schemas),
                    ..ANY
                }
            }
        }
        pub fn one_of(schemas: impl SchemaList) -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema {
                    one_of: SchemaList::collect(schemas),
                    ..ANY
                }
            }
        }
    }
};

pub trait SchemaList {
    fn collect(self) -> Vec<SchemaRef>;
}
impl<S: Into<SchemaRef>> SchemaList for S {
    fn collect(self) -> Vec<SchemaRef> {
        vec![self.into()]
    }
}
macro_rules! tuple_schemalist {
    ($($S:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($S: Into<SchemaRef>),*> SchemaList for ($($S,)*) {
            fn collect(self) -> Vec<SchemaRef> {
                let ($($S,)*) = self;
                vec![$($S.into()),*]
            }
        }
    }
}
tuple_schemalist!(S1);
tuple_schemalist!(S1, S2);
tuple_schemalist!(S1, S2, S3);
tuple_schemalist!(S1, S2, S3, S4);

/* metadata and flags */
impl<T: Type::SchemaType> Schema<T> {
    pub fn description(mut self, description: &'static str) -> Self {
        self.raw.description = Some(description);
        self
    }
    pub fn default(mut self, default: impl Serialize) -> Self {
        self.raw.default =
            Some(serde_json::to_value(default).expect("can't serialize given `default` value"));
        self
    }
    pub fn example(mut self, example: impl Serialize) -> Self {
        self.raw.example =
            Some(serde_json::to_value(example).expect("can't serialize given `example` value"));
        self
    }
    pub fn enumerates<const N: usize, V: Serialize>(mut self, enumerates: [V; N]) -> Self {
        self.raw.enumerates = enumerates
            .map(|v| serde_json::to_value(v).expect("can't serialize given `enum` values"))
            .into();
        self
    }
    pub fn deprecated(mut self) -> Self {
        self.raw.deprecated = true;
        self
    }
    pub fn nullable(mut self) -> Self {
        self.raw.nullable = true;
        self
    }
    pub fn read_only(mut self) -> Self {
        self.raw.read_only = true;
        self
    }
    pub fn write_only(mut self) -> Self {
        self.raw.write_only = true;
        self
    }
}

/* string definition */
impl Schema<Type::string> {
    pub fn format(mut self, format: &'static str) -> Self {
        self.raw.format = Some(format);
        self
    }
    pub fn pattern(mut self, pattern: &'static str) -> Self {
        self.raw.pattern = Some(pattern);
        self
    }
}

/* object definition */
impl Schema<Type::object> {
    pub fn property(mut self, name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        self.raw.properties.insert(name, schema.into());
        self.raw.required.push(name);
        self
    }
    pub fn optional(mut self, name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        self.raw.properties.insert(name, schema.into());
        self
    }
}

/* array definition */
impl Schema<Type::array> {
    pub fn max_items(mut self, max_items: usize) -> Self {
        self.raw.max_items = Some(max_items);
        self
    }
    pub fn min_items(mut self, min_items: usize) -> Self {
        self.raw.min_items = Some(min_items);
        self
    }
    pub fn max_properties(mut self, max_properties: usize) -> Self {
        self.raw.max_properties = Some(max_properties);
        self
    }
    pub fn min_properties(mut self, min_properties: usize) -> Self {
        self.raw.min_properties = Some(min_properties);
        self
    }
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.raw.max_length = Some(max_length);
        self
    }
    pub fn min_length(mut self, min_length: usize) -> Self {
        self.raw.min_length = Some(min_length);
        self
    }
    pub fn unique_items(mut self) -> Self {
        self.raw.unique_items = true;
        self
    }
}

/* number,integer definition */
impl Schema<Type::number> {
    pub fn format(mut self, format: &'static str) -> Self {
        self.raw.format = Some(format);
        self
    }
    pub fn multiple_of(mut self, n: impl Into<f64>) -> Self {
        self.raw.multiple_of = Some(n.into());
        self
    }
    pub fn maximum(mut self, maximum: impl Into<f64>) -> Self {
        self.raw.maximum = Some(maximum.into());
        self
    }
    pub fn exclusive_maximum(mut self, maximum: impl Into<f64>) -> Self {
        self.raw.maximum = Some(maximum.into());
        self.raw.exclusive_maximum = true;
        self
    }
    pub fn minimum(mut self, minimum: impl Into<f64>) -> Self {
        self.raw.minimum = Some(minimum.into());
        self
    }
    pub fn exclusive_minimum(mut self, minimum: impl Into<f64>) -> Self {
        self.raw.minimum = Some(minimum.into());
        self.raw.exclusive_minimum = true;
        self
    }
}
impl Schema<Type::integer> {
    pub fn format(mut self, format: &'static str) -> Self {
        self.raw.format = Some(format);
        self
    }
    pub fn multiple_of(mut self, n: i32) -> Self {
        self.raw.multiple_of = Some(n.into());
        self
    }
    pub fn maximum(mut self, maximum: i32) -> Self {
        self.raw.maximum = Some(maximum.into());
        self
    }
    pub fn exclusive_maximum(mut self, maximum: i32) -> Self {
        self.raw.maximum = Some(maximum.into());
        self.raw.exclusive_maximum = true;
        self
    }
    pub fn minimum(mut self, minimum: i32) -> Self {
        self.raw.minimum = Some(minimum.into());
        self
    }
    pub fn exclusive_minimum(mut self, minimum: i32) -> Self {
        self.raw.minimum = Some(minimum.into());
        self.raw.exclusive_minimum = true;
        self
    }
}

#[cfg(test)]
fn __usability__() {
    let _user_schema = Schema::object()
        .property("id", Schema::integer().read_only())
        .property("name", Schema::string())
        .optional("age", Schema::integer().minimum(18).maximum(120));
}
