use super::{Parameter, RequestBody, Responses, security::SecurityScheme};
use super::schema::RawSchema;
use super::_util::{is_false, Map};
use serde::Serialize;

#[derive(Serialize)]
pub struct Paths(
    Map<String, Operations>
);

#[derive(Serialize)]
pub struct Operations(
    Map<&'static str, Operation>
);

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
    security: Vec<Map<SecuritySchemeName, Vec<&'static str>>>,

    responses: Responses,
}
#[derive(Clone)]
struct SecuritySchemeName(SecurityScheme);
impl PartialEq for SecuritySchemeName {
    fn eq(&self, other: &Self) -> bool {
        self.0.__name__ == other.0.__name__
    }
}
impl PartialOrd for SecuritySchemeName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self.0.__name__, other.0.__name__)
    }
}
impl Serialize for SecuritySchemeName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0.__name__)
    }
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

    pub fn at(mut self, path: impl Into<String>, operations: Operations) -> Self {
        self.0.insert(path.into(), operations);
        self
    }
}

impl Operations {
    pub fn new() -> Self {
        Self(Map::new())
    }

    pub fn get(mut self, operation: Operation) -> Self {
        self.register("get", operation);
        self
    }
    pub fn put(mut self, operation: Operation) -> Self {
        self.register("put", operation);
        self
    }
    pub fn post(mut self, operation: Operation) -> Self {
        self.register("post", operation);
        self
    }
    pub fn patch(mut self, operation: Operation) -> Self {
        self.register("patch", operation);
        self
    }
    pub fn delete(mut self, operation: Operation) -> Self {
        self.register("delete", operation);
        self
    }
    pub fn options(mut self, operation: Operation) -> Self {
        self.register("options", operation);
        self
    }

    #[doc(hidden)]
    pub fn register(&mut self, method: &'static str, operation: Operation) {
        if matches!(method, "get" | "put" | "post" | "patch" | "delete" | "options") {
            self.0.insert(method, operation);
        }
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

    pub fn security(mut self, securityScheme: SecurityScheme, scopes: &[&'static str]) -> Self {
        self.security.push(Map::from_iter([(SecuritySchemeName(securityScheme), scopes.into())]));
        self
    }

    pub fn operationId(mut self, operationId: &'static str) -> Self {
        self.operationId = Some(operationId);
        self
    }
    pub fn with_tag(mut self, tag: &'static str) -> Self {
        self.tags.push(tag);
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

    pub fn inbound(mut self, inbound: crate::Inbound) -> Self {
        match inbound {
            crate::Inbound::None => self,
            crate::Inbound::Security { scheme, scopes } => self.security(scheme, scopes),
            crate::Inbound::Body(body) => self.requestBody(body),
            crate::Inbound::Param(param) => self.param(param),
            crate::Inbound::Params(params) => {
                for param in params {self = self.param(param)}
                self
            },
        }
    }

    pub fn param_description(
        mut self,
        name: &'static str,
        new_description: &'static str
    ) -> Self {
        if let Some(param) = self.parameters.iter_mut().find(|p| p.name == name) {
            param.set_description(new_description);
        }
        self
    }
    pub fn requestBody_description(
        mut self,
        new_description: &'static str
    ) -> Self {
        if let Some(requestBody) = &mut self.requestBody {
            requestBody.set_description(new_description);
        }
        self
    }
    pub fn response_description(
        mut self,
        status: impl Into<super::response::Status>,
        new_description: &'static str
    ) -> Self {
        self.responses.override_response_description(status, new_description);
        self
    }

    #[doc(hidden)]
    pub fn assign_path_param_name(&mut self, name: impl Into<std::borrow::Cow<'static, str>>) {
        if let Some(empty_param) = self.parameters.iter_mut()
            .filter(|p| p.is_path())
            .find(|p| p.name.is_empty())
        {
            empty_param.name = name.into();
        }
    }

    #[doc(hidden)]
    pub fn iter_securitySchemes(&self) -> impl Iterator<Item = SecurityScheme> {
        self.security.clone().into_iter()
            .map(|map| {
                let [SecuritySchemeName(ss)] = map.clone()
                    .into_keys()
                    .collect::<Vec<_>>()
                    .try_into().ok()
                    .expect("[OpenAPI] Unexpected multiple keys in one element of SecurityRequirement");
                ss
            })
    }
    #[doc(hidden)]
    pub fn refize_schemas(&mut self) -> impl Iterator<Item = RawSchema> + '_ {
        [/* RawSchema */].into_iter()
            .chain(self.parameters.iter_mut().map(|p| p.schema.refize()).flatten())
            .chain(self.requestBody.as_mut().map(RequestBody::refize_schemas).into_iter().flatten())
            .chain(self.responses.refize_schemas())
    }
}

#[cfg(test)]
#[test] fn map_openapi_operation_compiles() {
    #[allow(unused)]
    fn map_openapi_operation(op: Operation) -> Operation {
        op
        .operationId("list_users")
        .description("This doc comment is used for the\n`description` field of OpenAPI document")
        .summary("...")
        .response_description(200, "List of all users")
    }
}
