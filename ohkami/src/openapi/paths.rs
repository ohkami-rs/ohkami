use super::{Parameter, RequestBody, Responses, document::Components};
use super::_util::{is_false, Map};
use serde::Serialize;

#[derive(Serialize)]
pub struct Paths(
    Map<&'static str, Operations>
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

#[derive(Serialize, Clone)]
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
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<Parameter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    requestBody: Option<RequestBody>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    security: Vec<Map<&'static str, Vec<&'static str>>>,
    responses: Responses,
}

#[derive(Serialize, Clone)]
pub struct ExternalDoc {
    pub url: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>
}

impl Paths {
    pub fn new() -> Self {
        Self(Map::new())
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
            deprecated:   false,
            parameters:   Vec::new(),
            requestBody:  None,
            security:     Vec::new(),
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
        self.security.push(Map::from_iter([(schema, scopes.into())]));
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

    pub(crate) fn input(mut self, input: Option<super::Input>) -> Self {
        match input {
            None => self,
            Some(super::Input::Body(body)) => self.requestBody(body),
            Some(super::Input::Param(param)) => self.param(param),
            Some(super::Input::Params(params)) => {
                for param in params {self = self.param(param)}
                self
            }
        }
    }
}
