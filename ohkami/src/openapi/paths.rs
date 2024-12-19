use super::{Parameter, RequestBody, Responses};
use super::schema::{SchemaRef, Schema, Type::SchemaType};
use super::_util::{Content, is_false};
use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Paths(
    HashMap<&'static str, Operations>
);

#[derive(Serialize)]
pub struct Operations {
    #[serde(skip_serializing_if = "Option::is_none")]
    get:     Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post:    Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put:     Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch:   Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete:  Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    head:    Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Operation>,
}

#[derive(Serialize)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    operationId: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    summary:      Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description:  Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    externalDocs: Option<ExternalDoc>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<Parameter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    requestBody: Option<RequestBody>,

    responses: Responses,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    security: Vec<HashMap<&'static str, Vec<&'static str>>>,

    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,
}

#[derive(Serialize)]
pub struct ExternalDoc {
    pub url: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>
}

impl Paths {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn at(mut self, path: &'static str, operations: Operations) -> Self {
        self.0.insert(path, operations);
        self
    }
}

impl Operations {
    pub fn new() -> Self {
        Self { get:None, post:None, put:None, patch:None, delete:None, head:None, options:None }
    }

    pub fn get(mut self, operation: Operation) -> Self {
        self.get = Some(operation);
        self
    }
    pub fn post(mut self, operation: Operation) -> Self {
        self.post = Some(operation);
        self
    }
    pub fn put(mut self, operation: Operation) -> Self {
        self.put = Some(operation);
        self
    }
    pub fn patch(mut self, operation: Operation) -> Self {
        self.patch = Some(operation);
        self
    }
    pub fn delete(mut self, operation: Operation) -> Self {
        self.delete = Some(operation);
        self
    }
    pub fn options(mut self, operation: Operation) -> Self {
        self.options = Some(operation);
        self
    }
    pub fn head(mut self, operation: Operation) -> Self {
        self.head = Some(operation);
        self
    }
}

impl Operation {
    pub fn with(responses: Responses) -> Self {
        Self {
            responses,
            operationId:  None,
            tags:         Vec::new(),
            summary:      None,
            description:  None,
            externalDocs: None,
            parameters:   Vec::new(),
            requestBody:  None,
            security:     Vec::new(),
            deprecated:   false
        }
    }

    pub fn param(mut self, param: Parameter) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn requestBody(mut self, requestBody: RequestBody) -> Self {
        self.requestBody = Some(requestBody);
        self
    }

    pub fn security<const N: usize>(mut self, schema: &'static str, scopes: [&'static str; N]) -> Self {
        self.security.push(HashMap::from_iter([(schema, scopes.into())]));
        self
    }

    pub fn operationId(mut self, operationId: &'static str) -> Self {
        self.operationId = Some(operationId);
        self
    }
    pub fn tags<const N: usize>(mut self, tags: [&'static str; N]) -> Self {
        self.tags = tags.into();
        self
    }
    pub fn summary(mut self, summary: &'static str) -> Self {
        self.summary = Some(summary);
        self
    }
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
    pub fn externalDocs(mut self, externalDocs: ExternalDoc) -> Self {
        self.externalDocs = Some(externalDocs);
        self
    }
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }
}
