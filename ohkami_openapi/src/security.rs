use super::_util::Map;
use serde::Serialize;

#[derive(Serialize, Clone, PartialEq)]
pub struct SecurityScheme {
    #[serde(skip)]
    pub(crate) __name__: &'static str,

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
    #[serde(rename = "bearerFormat")]
    bearer_format: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "openIdConnectUrl")]
    openidconnect_url: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    flows: Option<oauth2::OAuthFlow>,
}

#[derive(Clone)]
pub struct APIKey {
    apikey_in: &'static str,
    name:      &'static str,
}
impl APIKey {
    pub fn header(name: &'static str) -> Self {
        Self { apikey_in:"header", name }
    }
    pub fn query(name: &'static str) -> Self {
        Self { apikey_in:"query", name }
    }
    pub fn cookie(name: &'static str) -> Self {
        Self { apikey_in:"cookie", name }
    }
}

#[derive(Serialize, Clone, PartialEq)]
pub enum OAuthFlow {
    #[serde(rename = "authorizationCode")]
    AuthorizationCode {
        #[serde(rename = "authorizationUrl")]
        authorization_url: &'static str,
        #[serde(rename = "tokenUrl")]
        token_url:         &'static str,
    },
    #[serde(rename = "implicit")]
    Implicit {
        authorization_url: &'static str,
    },
    #[serde(rename = "password")]
    Password {
        token_url: &'static str,
    },
    #[serde(rename = "clientCredentials")]
    ClientCredentials {
        token_url: &'static str,
    },
}
mod oauth2 {
    use super::{Serialize, Map};

    #[derive(Serialize, Clone, PartialEq)]
    #[allow(private_interfaces)]
    pub enum OAuthFlow {
        #[serde(rename = "authorizationCode")]
        AuthorizationCode {
            #[serde(rename = "authorizationUrl")]
            authorization_url: &'static str,
            #[serde(rename = "tokenUrl")]
            token_url: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "refreshUrl")]
            refresh_url: Option<&'static str>,
            scopes: Map<&'static str, &'static str>
        },
        #[serde(rename = "implicit")]
        Implicit {
            #[serde(rename = "authorizationUrl")]
            authorization_url: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "refreshUrl")]
            refresh_url: Option<&'static str>,
            scopes: Map<&'static str, &'static str>
        },
        #[serde(rename = "password")]
        Password {
            #[serde(rename = "tokenUrl")]
            token_url: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "refreshUrl")]
            refresh_url: Option<&'static str>,
            scopes: Map<&'static str, &'static str>
        },
        #[serde(rename = "clientCredentials")]
        ClientCredentials {
            #[serde(rename = "tokenUrl")]
            token_url: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "refreshUrl")]
            refresh_url: Option<&'static str>,
            scopes: Map<&'static str, &'static str>
        },
    }

    impl super::OAuthFlow {
        pub fn refresh_url(self, refresh_url: &'static str) -> OAuthFlow {
            OAuthFlow::from(self).refresh_url(refresh_url)
        }

        pub fn scope(self, name: &'static str, description: &'static str) -> OAuthFlow {
            OAuthFlow::from(self).scope(name, description)
        }
    }
    impl OAuthFlow {
        pub fn refresh_url(mut self, url: &'static str) -> Self {
            match &mut self {
                | OAuthFlow::AuthorizationCode { refresh_url, .. }
                | OAuthFlow::Implicit { refresh_url, .. }
                | OAuthFlow::Password { refresh_url, .. }
                | OAuthFlow::ClientCredentials { refresh_url, .. }
                => *refresh_url = Some(url)
            }
            self
        }
        pub fn scope(mut self, name: &'static str, description: &'static str) -> Self {
            match &mut self {
                | OAuthFlow::AuthorizationCode { scopes, .. }
                | OAuthFlow::Implicit { scopes, .. }
                | OAuthFlow::Password { scopes, .. }
                | OAuthFlow::ClientCredentials { scopes, .. }
                => scopes.insert(name, description)
            }
            self
        }
    }
    impl From<super::OAuthFlow> for OAuthFlow {
        fn from(it: super::OAuthFlow) -> OAuthFlow {
            match it {
                super::OAuthFlow::AuthorizationCode {
                    authorization_url,
                    token_url,
                } => Self::AuthorizationCode {
                    authorization_url,
                    token_url,
                    refresh_url:None, scopes:Map::new()
                },
                super::OAuthFlow::Implicit {
                    authorization_url,
                } => Self::Implicit {
                    authorization_url,
                    refresh_url:None, scopes:Map::new()
                },
                super::OAuthFlow::Password {
                    token_url,
                } => Self::Password {
                    token_url,
                    refresh_url:None, scopes:Map::new()
                },
                super::OAuthFlow::ClientCredentials {
                    token_url,
                } => Self::ClientCredentials {
                    token_url,
                    refresh_url:None, scopes:Map::new()
                },
            }
        }
    }
}

impl SecurityScheme {
    pub fn basic(scheme_name: &'static str) -> Self {
        Self {
            __name__:  scheme_name,
            auth_type: "http",
            scheme:    Some("basic"),
            name:None, apikey_in:None, bearer_format:None, openidconnect_url:None, flows:None, description:None
        }
    }
    pub fn bearer(scheme_name: &'static str, token_format: Option<&'static str>) -> Self {
        Self {
            __name__:     scheme_name,
            auth_type:    "http",
            scheme:       Some("bearer"),
            bearer_format: token_format.into(),
            name:None, apikey_in:None, openidconnect_url:None, flows:None, description:None
        }
    }
    pub fn openidconnect(scheme_name: &'static str, url: &'static str) -> Self {
        Self {
            __name__:         scheme_name,
            auth_type:        "openIdConnect",
            openidconnect_url: Some(url),
            scheme:None, name:None, apikey_in:None, bearer_format:None, flows:None, description:None
        }
    }
    pub fn apikey(scheme_name: &'static str, APIKey { apikey_in, name }: APIKey) -> Self {
        Self {
            __name__:  scheme_name,
            auth_type: "apiKey",
            name:      Some(name),
            apikey_in: Some(apikey_in),
            scheme:None, bearer_format:None, openidconnect_url:None, flows:None, description:None
        }
    }
    pub fn oauth2(scheme_name: &'static str, flow: impl Into<oauth2::OAuthFlow>) -> Self {
        Self {
            __name__:  scheme_name,
            auth_type: "oauth2",
            flows:     Some(flow.into()),
            openidconnect_url: None, scheme:None, name:None, apikey_in:None, bearer_format:None, description:None
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
}
