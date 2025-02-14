use super::_util::{is_false, Map};
use std::marker::PhantomData;
use serde::Serialize;
use serde_json::Value;

pub struct Schema<T: SchemaType> {
    datatype: PhantomData<fn()->T>,
    raw: RawSchema
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
    pub trait SchemaType: Sealed {const NAME: &'static str;}
    trait Sealed {const NAME: &'static str;}
    impl<S: Sealed> SchemaType for S {const NAME: &'static str = <S as Sealed>::NAME;}

    impl Sealed for string {const NAME: &'static str = "string";}
    impl Sealed for number {const NAME: &'static str = "number";}
    impl Sealed for integer {const NAME: &'static str = "integer";}
    impl Sealed for bool {const NAME: &'static str = "bool";}
    impl Sealed for array {const NAME: &'static str = "array";}
    impl Sealed for object {const NAME: &'static str = "object";}
    impl Sealed for any {const NAME: &'static str = "";}
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
    #[serde(rename = "enum", skip_serializing_if = "<[_]>::is_empty")]
    enumerates: Vec<Value>,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,
    #[serde(skip_serializing_if = "is_false")]
    nullable: bool,
    #[serde(skip_serializing_if = "is_false")]
    readOnly: bool,
    #[serde(skip_serializing_if = "is_false")]
    writeOnly: bool,

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
    #[serde(skip_serializing_if = "is_false")]
    uniqueItems: bool,

    /* number,integer definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    multipleOf: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum: Option<f64>,
    #[serde(skip_serializing_if = "is_false")]
    exclusiveMaximum: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    minimum: Option<f64>,
    #[serde(skip_serializing_if = "is_false")]
    exclusiveMinimum: bool,
}
impl<T: Type::SchemaType> From<Schema<T>> for RawSchema {
    fn from(schema: Schema<T>) -> Self {
        schema.raw
    }
}
impl<T: Type::SchemaType> From<RawSchema> for Schema<T> {
    fn from(raw: RawSchema) -> Self {
        Self { raw, datatype:PhantomData }
    }
}
impl Into<SchemaRef> for RawSchema {
    fn into(self) -> SchemaRef {
        SchemaRef::Inline(Box::new(self))
    }
}
impl RawSchema {
    #[doc(hidden)]
    pub fn into_properties(self) -> Vec<(&'static str, SchemaRef, bool)> {
        self.properties.into_iter()
            .map(|(name, schema)| (name, schema, self.required.contains(&name)))
            .collect()
    }
}

#[derive(PartialEq, Clone)]
#[allow(private_interfaces/* construct only via `From` */)]
pub enum SchemaRef {
    Inline(Box<RawSchema>),
    Reference(&'static str)
}
impl Serialize for SchemaRef {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S
    ) -> Result<S::Ok, S::Error> {
        match self {
            SchemaRef::Inline(schema)  => schema.serialize(serializer),
            SchemaRef::Reference(name) => {use serde::ser::SerializeMap;
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
            SchemaRef::Inline(raw)  => Some(*raw),
            SchemaRef::Reference(_) => None
        }
    }

    pub(crate) fn refize(&mut self) -> impl Iterator<Item = RawSchema> {
        let mut component_schemas = vec![];
        match self {
            SchemaRef::Inline(raw) => {
                raw.properties.values_mut().for_each(|s| component_schemas.extend(s.refize()));
                raw.anyOf.iter_mut().for_each(|s| component_schemas.extend(s.refize()));
                raw.allOf.iter_mut().for_each(|s| component_schemas.extend(s.refize()));
                raw.oneOf.iter_mut().for_each(|s| component_schemas.extend(s.refize()));
                raw.items.as_mut().map(|s| component_schemas.extend(s.refize()));
                if let Some(name) = raw.__name__ {
                    let raw = std::mem::replace(self, SchemaRef::Reference(name));
                    component_schemas.push(raw.into_inline().unwrap());
                }
            }
            _ => ()
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
        anyOf: Vec::new(),
        allOf: Vec::new(),
        oneOf: Vec::new(),

        /* metadata and flags */
        description: None,
        default:     None,
        example:     None,
        enumerates:  Vec::new(),
        deprecated:  false,
        nullable:    false,
        readOnly:    false,
        writeOnly:   false,

        /* string definition */
        pattern: None,

        /* object definition */
        properties: Map::new(),
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
        pub fn anyOf(schemas: impl SchemaList) -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema {
                    anyOf: SchemaList::collect(schemas),
                    ..ANY
                }
            }
        }
        pub fn allOf(schemas: impl SchemaList) -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema {
                    allOf: SchemaList::collect(schemas),
                    ..ANY
                }
            }
        }
        pub fn oneOf(schemas: impl SchemaList) -> Self {
            Self {
                datatype: PhantomData,
                raw: RawSchema {
                    oneOf: SchemaList::collect(schemas),
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
    fn collect(self) -> Vec<SchemaRef> {vec![self.into()]}
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
        self.raw.default = Some(serde_json::to_value(default).expect("can't serialize given `default` value"));
        self
    }
    pub fn example(mut self, example: impl Serialize) -> Self {
        self.raw.example = Some(serde_json::to_value(example).expect("can't serialize given `example` value"));
        self
    }
    pub fn enumerates<const N: usize, V: Serialize>(mut self, enumerates: [V; N]) -> Self {
        self.raw.enumerates = enumerates.map(
            |v| serde_json::to_value(v).expect("can't serialize given `enum` values")
        ).into();
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
    pub fn readOnly(mut self) -> Self {
        self.raw.readOnly = true;
        self
    }
    pub fn writeOnly(mut self) -> Self {
        self.raw.writeOnly = true;
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
    pub fn maxItems(mut self, maxItems: usize) -> Self {
        self.raw.maxItems = Some(maxItems);
        self
    }
    pub fn minItems(mut self, minItems: usize) -> Self {
        self.raw.minItems = Some(minItems);
        self
    }
    pub fn maxProperties(mut self, maxProperties: usize) -> Self {
        self.raw.maxProperties = Some(maxProperties);
        self
    }
    pub fn minProperties(mut self, minProperties: usize) -> Self {
        self.raw.minProperties = Some(minProperties);
        self
    }
    pub fn maxLength(mut self, maxLength: usize) -> Self {
        self.raw.maxLength = Some(maxLength);
        self
    }
    pub fn minLength(mut self, minLength: usize) -> Self {
        self.raw.minLength = Some(minLength);
        self
    }
    pub fn uniqueItems(mut self) -> Self {
        self.raw.uniqueItems = true;
        self
    }
}

/* number,integer definition */
impl Schema<Type::number> {
    pub fn format(mut self, format: &'static str) -> Self {
        self.raw.format = Some(format);
        self
    }
    pub fn multipleOf(mut self, n: impl Into<f64>) -> Self {
        self.raw.multipleOf = Some(n.into());
        self
    }
    pub fn maximum(mut self, maximum: impl Into<f64>) -> Self {
        self.raw.maximum = Some(maximum.into());
        self
    }
    pub fn exclusiveMaximum(mut self, maximum: impl Into<f64>) -> Self {
        self.raw.maximum = Some(maximum.into());
        self.raw.exclusiveMaximum = true;
        self
    }
    pub fn minimum(mut self, minimum: impl Into<f64>) -> Self {
        self.raw.minimum = Some(minimum.into());
        self
    }
    pub fn exclusiveMinimum(mut self, minimum: impl Into<f64>) -> Self {
        self.raw.minimum = Some(minimum.into());
        self.raw.exclusiveMinimum = true;
        self
    }
}
impl Schema<Type::integer> {
    pub fn format(mut self, format: &'static str) -> Self {
        self.raw.format = Some(format);
        self
    }
    pub fn multipleOf(mut self, n: i32) -> Self {
        self.raw.multipleOf = Some(n.into());
        self
    }
    pub fn maximum(mut self, maximum: i32) -> Self {
        self.raw.maximum = Some(maximum.into());
        self
    }
    pub fn exclusiveMaximum(mut self, maximum: i32) -> Self {
        self.raw.maximum = Some(maximum.into());
        self.raw.exclusiveMaximum = true;
        self
    }
    pub fn minimum(mut self, minimum: i32) -> Self {
        self.raw.minimum = Some(minimum.into());
        self
    }
    pub fn exclusiveMinimum(mut self, minimum: i32) -> Self {
        self.raw.minimum = Some(minimum.into());
        self.raw.exclusiveMinimum = true;
        self
    }
}

#[cfg(test)]
fn __usability__() {
    let _user_schema = Schema::object()
        .property("id", Schema::integer().readOnly())
        .property("name", Schema::string())
        .optional("age", Schema::integer().minimum(18).maximum(120));
}
