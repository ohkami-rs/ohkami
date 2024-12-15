mod util;

use serde::{ser::SerializeMap, Serialize};
use serde_json::Value;

// #[allow(non_upper_case_globals)]
// pub trait Schema {
// }

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "str::is_empty")]
    kind:  &'static str/* basically: string, number, integer, boolean, array, object */,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    anyOf: &'static [Value],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    allOf: &'static [Value],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    oneOf: &'static [Value],

    /* metadata */
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<Value>,
    #[serde(skip_serializing_if = "util::is_false")]
    deprecated: bool,
    #[serde(skip_serializing_if = "util::is_false")]
    nullable: bool,
    #[serde(skip_serializing_if = "util::is_false")]
    readOnly: bool,
    #[serde(skip_serializing_if = "util::is_false")]
    writeOnly: bool,

    /* string definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<&'static str>,
    #[serde(rename = "enum", skip_serializing_if = "<[_]>::is_empty")]
    enumerates: &'static [&'static str],

    /* object definition */
    #[serde(skip_serializing_if = "<[_]>::is_empty", serialize_with = "serialize_properties")]
    properties: &'static [(&'static str, SchemaRef)],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    required: &'static [&'static str],

    /* array definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<SchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maxItems: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minItems: Option<usize>,
    #[serde(skip_serializing_if = "util::is_false")]
    uniqueItems: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    maxProperties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minProperties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maxLength: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minLength: Option<usize>,

    /* number,integer definition */
    #[serde(skip_serializing_if = "Option::is_none")]
    multipleOf: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum: Option<f64>,
    #[serde(skip_serializing_if = "util::is_false")]
    exclusiveMaximum: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    minimum: Option<f64>,
    #[serde(skip_serializing_if = "util::is_false")]
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
impl Default for Schema {
    fn default() -> Self {
        Schema {
            kind:  "",
            anyOf: &[],
            allOf: &[],
            oneOf: &[],

            /* metadata */
            description: None,
            default:    None,
            example:    None,
            deprecated:  false,
            nullable:    false,
            readOnly:    false,
            writeOnly:   false,

            /* string definition */
            format:  None,
            pattern: None,
            enumerates: &[],

            /* object definition */
            properties: &[],
            required:   &[],

            /* array definition */
            items:         None,
            maxItems:      None,
            minItems:      None,
            uniqueItems:   false,
            maxProperties: None,
            minProperties: None,
            maxLength:     None,
            minLength:     None,

            /* number,integer definition */
            multipleOf:       None,
            maximum:          None,
            exclusiveMaximum: false,
            minimum:          None,
            exclusiveMinimum: false,
        }
    }
}

pub enum SchemaRef {
    Inline(&'static Schema),
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
