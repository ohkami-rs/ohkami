use super::{Parameter, RequestBody, Responses};
use super::schema::{Schema, RawSchema, SchemaRef};
use super::_util::{is_false, Map};
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Serialize, Clone, PartialEq)]
pub struct SecurityScheme {
    #[serde(skip)]
    __name__: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    #[serde(rename = "type")]
    auth_type: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'static str>,

    #[serde(rename = "in")]
    #[serde(skip_serializing_if = "Option::is_none")]
    apikey_in: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    scheme: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    openIdConnectUrl: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    flows: Option<OAuthFlows>,
}

impl SecurityScheme {
    pub fn Basic() -> Self {
        Self {
            __type__: PhantomData,
            raw: RawSecurityScheme {__name__: None,
                auth_type: "http",
                scheme:    Some("basic"),
                name:None, apikey_in:None, openIdConnectUrl:None, flows:None, description:None
            }
        }
    }
    pub fn Bearer() -> Self {
        Self {
            __type__: PhantomData,
            raw: RawSecurityScheme {__name__: None,
                auth_type: "http",
                scheme:    Some("bearer"),
                name:None, apikey_in:None, openIdConnectUrl:None, flows:None, description:None
            }
        }
    }
    pub fn OpenIDConnect(url: &'static str) -> Self {
        Self {
            __type__: PhantomData,
            raw: RawSecurityScheme {__name__: None,
                auth_type:        "openIdConnect",
                openIdConnectUrl: Some(url),
                scheme:None, name:None, apikey_in:None, flows:None, description:None
            }
        }
    }
    pub fn OAuth2(flows: OAuthFlows) -> Self {
        Self {
            __type__: PhantomData,
            raw: RawSecurityScheme {__name__: None,
                auth_type: "oauth2",
                flows,
                openIdConnectUrl: None, scheme:None, name:None, apikey_in:None, description:None
            }
        }
    }
    pub fn APIKeyHeader(name: &'static str) -> Self {
        Self {
            __type__: PhantomData,
            raw: RawSecurityScheme {__name__: None,
                auth_type: "apiKey",
                name:      Some(name),
                apikey_in: Some("header"),
                scheme:None, openIdConnectUrl:None, flows:None, description:None
            }
        }
    }
    pub fn APIKeyQuery(name: &'static str) -> Self {
        Self {
            __type__: PhantomData,
            raw: RawSecurityScheme {__name__: None,
                auth_type: "apiKey",
                name:      Some(name),
                apikey_in: Some("query"),
                scheme:None, openIdConnectUrl:None, flows:None, description:None
            }
        }
    }
    pub fn APIKeyCookie(name: &'static str) -> Self {
        Self {
            __type__: PhantomData,
            raw: RawSecurityScheme {__name__: None,
                auth_type: "apiKey",
                name:      Some(name),
                apikey_in: Some("cookie"),
                scheme:None, openIdConnectUrl:None, flows:None, description:None
            }
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
    pub(crate) fn __name__(mut self, __name__: &'static str) -> Self {
        self.__name__ = Some(description);
        self
    }
}

#[derive(Serialize, Clone, PartialEq)]
pub struct OAuthFlows {
    pub authorizationCode: Option<OAuthFlow>,
    pub implicit:          Option<OAuthFlow>,
    pub password:          Option<OAuthFlow>,
    pub clientCredentials: Option<OAuthFlow>,
}

#[derive(Serialize, Clone, PartialEq)]
struct OAuthFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    authorizationUrl: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tokenUrl: Option<&'static str>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    refreshUrl: Option<&'static str>,
    
    scopes: Map<&'static str, &'static str>,
}

impl OAuthFlow {
    pub fn authorizationCode(
        authorizationUrl: &'static str,
        tokenUrl: &'static str
    ) -> Self {
        Self {
            authorizationUrl,
            tokenUrl,
            refreshUrl: None,
            scopes:     Map::new()
        }
    }

    pub fn implicit(authorizationUrl: &'static str) -> Self {
        Self {
            authorizationUrl,
            tokenUrl:   None,
            refreshUrl: None,
            scopes:     Map::new()
        }
    }

    pub fn password(tokenUrl: &'static str) -> Self {
        Self {
            tokenUrl,
            authorizationUrl: None,
            refreshUrl:       None,
            scopes:           Map::new()
        }
    }

    pub fn clientCredentials(tokenUrl: &'static str) -> Self {
        Self {
            tokenUrl,
            authorizationUrl: None,
            refreshUrl:       None,
            scopes:           Map::new()
        }
    }

    pub fn refreshUrl(mut self, refreshUrl: &'static str) -> Self {
        self.refreshUrl = Some(refreshUrl);
        self
    }
    pub fn scope(mut self, name: &'static str, description: &'static str) -> Self {
        self.scopes.insert(name, description);
        self
    }
}

