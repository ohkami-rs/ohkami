use super::_util;
use std::marker::PhantomData;
use serde::{Serialize, ser::SerializeMap};
use serde_json::Value;

#[derive(Serialize)]
pub struct Schema<T: SchemaType> {
    inner: SchemaInner,
    __type__: PhantomData<fn()->T>
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

    pub trait SchemaType {const NAME: &'static str;}
    impl SchemaType for string {const NAME: &'static str = "string";}
    impl SchemaType for number {const NAME: &'static str = "number";}
    impl SchemaType for integer {const NAME: &'static str = "integer";}
    impl SchemaType for bool {const NAME: &'static str = "bool";}
    impl SchemaType for array {const NAME: &'static str = "array";}
    impl SchemaType for object {const NAME: &'static str = "object";}
    impl SchemaType for any {const NAME: &'static str = "";}
}

#[derive(Serialize, PartialEq)]
struct SchemaInner {
    #[serde(rename = "type", skip_serializing_if = "str::is_empty")]
    __type__: &'static str,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    anyOf: Vec<SchemaRef>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    allOf: Vec<SchemaRef>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    oneOf: Vec<SchemaRef>,

    /* metadata and flags */
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<Value>,
    #[serde(skip_serializing_if = "_util::is_false")]
    deprecated: bool,
    #[serde(skip_serializing_if = "_util::is_false")]
    nullable: bool,
    #[serde(skip_serializing_if = "_util::is_false")]
    readOnly: bool,
    #[serde(skip_serializing_if = "_util::is_false")]
    writeOnly: bool,

    /* string definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<&'static str>,
    #[serde(rename = "enum", skip_serializing_if = "<[_]>::is_empty")]
    enumerates: Vec<&'static str>,

    /* object definition */
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    #[serde(serialize_with = "serialize_properties")]
    properties: Vec<(&'static str, SchemaRef)>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    required: Vec<&'static str>,

    /* array definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<SchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maxItems: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minItems: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maxProperties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minProperties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maxLength: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minLength: Option<usize>,
    #[serde(skip_serializing_if = "_util::is_false")]
    uniqueItems: bool,

    /* number,integer definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    multipleOf: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum: Option<f64>,
    #[serde(skip_serializing_if = "_util::is_false")]
    exclusiveMaximum: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    minimum: Option<f64>,
    #[serde(skip_serializing_if = "_util::is_false")]
    exclusiveMinimum: bool,
}
fn serialize_properties<S: serde::Serializer>(
    properties: &[(&'static str, SchemaRef)],
    serializer: S
) -> Result<S::Ok, S::Error> {
    let mut s = serializer.serialize_map(None)?;
    for (k, v) in properties {
        s.serialize_entry(k, v)?;
    }
    s.end()
}

#[derive(PartialEq)]
#[allow(private_interfaces/* construct only via `From` */)]
pub enum SchemaRef {
    Inline(Box<SchemaInner>),
    Reference(&'static str)
}
impl Serialize for SchemaRef {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S
    ) -> Result<S::Ok, S::Error> {
        match self {
            SchemaRef::Inline(schema)  => schema.serialize(serializer),
            SchemaRef::Reference(name) => {
                let mut s = serializer.serialize_map(None)?;
                s.serialize_entry("$ref", &format!("#/components/schemas/{name}"))?;
                s.end()
            }
        }    
    }
}
impl<T: SchemaType> From<Schema<T>> for SchemaRef {
    fn from(schema: Schema<T>) -> Self {
        SchemaRef::Inline(Box::new(schema.inner))
    }
}
impl From<&'static str> for SchemaRef {
    fn from(name: &'static str) -> Self {
        SchemaRef::Reference(name)
    }
}

const _: (/* constructors */) = {
    const ANY: SchemaInner = SchemaInner {
        __type__: Type::any::NAME,
        anyOf: Vec::new(),
        allOf: Vec::new(),
        oneOf: Vec::new(),

        /* metadata and flags */
        description: None,
        default:     None,
        example:     None,
        deprecated:  false,
        nullable:    false,
        readOnly:    false,
        writeOnly:   false,

        /* string definition */
        format:  None,
        pattern: None,
        enumerates: Vec::new(),

        /* object definition */
        properties: Vec::new(),
        required:   Vec::new(),

        /* array definition */
        items:         None,
        maxItems:      None,
        minItems:      None,
        maxProperties: None,
        minProperties: None,
        maxLength:     None,
        minLength:     None,
        uniqueItems:   false,

        /* number,integer definition */
        multipleOf:       None,
        maximum:          None,
        exclusiveMaximum: false,
        minimum:          None,
        exclusiveMinimum: false,
    };

    impl Schema<Type::string> {
        pub fn string() -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner { __type__: Type::string::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::number> {
        pub fn number() -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner { __type__: Type::number::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::integer> {
        pub fn integer() -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner { __type__: Type::integer::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::bool> {
        pub fn bool() -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner { __type__: Type::bool::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::array> {
        pub fn array() -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner { __type__: Type::array::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::object> {
        pub fn object() -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner { __type__: Type::object::NAME, ..ANY }
            }
        }
    }
    impl Schema<Type::any> {
        pub fn anyOf<const N: usize>(schema_refs: [&'static str; N]) -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner {
                    anyOf: schema_refs.map(SchemaRef::Reference).into(),
                    ..ANY
                }
            }
        }
        pub fn allOf<const N: usize>(schema_refs: [&'static str; N]) -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner {
                    allOf: schema_refs.map(SchemaRef::Reference).into(),
                    ..ANY
                }
            }
        }
        pub fn oneOf<const N: usize>(schema_refs: [&'static str; N]) -> Self {
            Self {
                __type__: PhantomData,
                inner: SchemaInner {
                    oneOf: schema_refs.map(SchemaRef::Reference).into(),
                    ..ANY
                }
            }
        }
    }
};

/* metadata and flags */
impl<T: Type::SchemaType> Schema<T> {
    pub fn description(mut self, description: &'static str) -> Self {
        self.inner.description = Some(description);
        self
    }
    pub fn default(mut self, default: impl Serialize) -> Self {
        self.inner.default = Some(serde_json::to_value(default).expect("can't serialize given `default` value"));
        self
    }
    pub fn example(mut self, example: impl Serialize) -> Self {
        self.inner.example = Some(serde_json::to_value(example).expect("can't serialize given `example` value"));
        self
    }
    pub fn deprecated(mut self) -> Self {
        self.inner.deprecated = true;
        self
    }
    pub fn nullable(mut self) -> Self {
        self.inner.nullable = true;
        self
    }
    pub fn readOnly(mut self) -> Self {
        self.inner.readOnly = true;
        self
    }
    pub fn writeOnly(mut self) -> Self {
        self.inner.writeOnly = true;
        self
    }
}

/* string definition */
impl Schema<Type::string> {
    pub fn format(mut self, format: &'static str) -> Self {
        self.inner.format = Some(format);
        self
    }
    pub fn pattern(mut self, pattern: &'static str) -> Self {
        self.inner.pattern = Some(pattern);
        self
    }
    pub fn enumerates<const N: usize>(mut self, enumerates: [&'static str; N]) -> Self {
        self.inner.enumerates = enumerates.into();
        self
    }
}

/* object definition */
impl Schema<Type::object> {
    pub fn property(mut self, name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        self.inner.properties.push((name, schema.into()));
        self.inner.required.push(name);
        self
    }
    pub fn optional(mut self, name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        self.inner.properties.push((name, schema.into()));
        self
    }
}

/* array definition */
impl Schema<Type::array> {
    pub fn items(mut self, schema: impl Into<SchemaRef>) -> Self {
        self.inner.items = Some(schema.into());
        self
    }
    pub fn maxItems(mut self, maxItems: usize) -> Self {
        self.inner.maxItems = Some(maxItems);
        self
    }
    pub fn minItems(mut self, minItems: usize) -> Self {
        self.inner.minItems = Some(minItems);
        self
    }
    pub fn maxProperties(mut self, maxProperties: usize) -> Self {
        self.inner.maxProperties = Some(maxProperties);
        self
    }
    pub fn minProperties(mut self, minProperties: usize) -> Self {
        self.inner.minProperties = Some(minProperties);
        self
    }
    pub fn maxLength(mut self, maxLength: usize) -> Self {
        self.inner.maxLength = Some(maxLength);
        self
    }
    pub fn minLength(mut self, minLength: usize) -> Self {
        self.inner.minLength = Some(minLength);
        self
    }
    pub fn uniqueItems(mut self) -> Self {
        self.inner.uniqueItems = true;
        self
    }
}

/* number,integer definition */
impl Schema<Type::number> {
    pub fn multipleOf(mut self, n: impl Into<f64>) -> Self {
        self.inner.multipleOf = Some(n.into());
        self
    }
    pub fn maximum(mut self, maximum: impl Into<f64>) -> Self {
        self.inner.maximum = Some(maximum.into());
        self
    }
    pub fn exclusiveMaximum(mut self, maximum: impl Into<f64>) -> Self {
        self.inner.maximum = Some(maximum.into());
        self.inner.exclusiveMaximum = true;
        self
    }
    pub fn minimum(mut self, minimum: impl Into<f64>) -> Self {
        self.inner.minimum = Some(minimum.into());
        self
    }
    pub fn exclusiveMinimum(mut self, minimum: impl Into<f64>) -> Self {
        self.inner.minimum = Some(minimum.into());
        self.inner.exclusiveMinimum = true;
        self
    }
}
impl Schema<Type::integer> {
    pub fn multipleOf(mut self, n: i32) -> Self {
        self.inner.multipleOf = Some(n.into());
        self
    }
    pub fn maximum(mut self, maximum: i32) -> Self {
        self.inner.maximum = Some(maximum.into());
        self
    }
    pub fn exclusiveMaximum(mut self, maximum: i32) -> Self {
        self.inner.maximum = Some(maximum.into());
        self.inner.exclusiveMaximum = true;
        self
    }
    pub fn minimum(mut self, minimum: i32) -> Self {
        self.inner.minimum = Some(minimum.into());
        self
    }
    pub fn exclusiveMinimum(mut self, minimum: i32) -> Self {
        self.inner.minimum = Some(minimum.into());
        self.inner.exclusiveMinimum = true;
        self
    }
}

#[cfg(test)]
fn __usability__() {
    let _user_schema = Schema::object()
        .property("id", Schema::integer())
        .property("name", Schema::string())
        .optional("age", Schema::integer());
}
