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
    openIdConnectUrl: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    flows: Option<oauth2::OAuthFlow>,
}

#[derive(Clone)]
pub enum APIKey {
    header { name: &'static str },
    query  { name: &'static str },
    cookie { name: &'static str },
}

#[derive(Serialize, Clone, PartialEq)]
pub enum OAuthFlow {
    authorizationCode {
        authorizationUrl: &'static str,
        tokenUrl:         &'static str,
    },
    implicit {
        authorizationUrl: &'static str,
    },
    password {
        tokenUrl: &'static str,
    },
    clientCredentials {
        tokenUrl: &'static str,
    },
}
mod oauth2 {
    use super::{Serialize, Map};

    #[derive(Serialize, Clone, PartialEq)]
    #[allow(private_interfaces)]
    pub enum OAuthFlow {
        authorizationCode {
            authorizationUrl: &'static str,
            tokenUrl:         &'static str,
            refreshUrl: Option<&'static str>, scopes: Map<&'static str, &'static str>
        },
        implicit {
            authorizationUrl: &'static str,
            refreshUrl: Option<&'static str>, scopes: Map<&'static str, &'static str>
        },
        password {
            tokenUrl: &'static str,
            refreshUrl: Option<&'static str>, scopes: Map<&'static str, &'static str>
        },
        clientCredentials {
            tokenUrl: &'static str,
            refreshUrl: Option<&'static str>, scopes: Map<&'static str, &'static str>
        },
    }

    impl super::OAuthFlow {
        pub fn refreshUrl(self, refreshUrl: &'static str) -> OAuthFlow {
            OAuthFlow::from(self).refreshUrl(refreshUrl)
        }

        pub fn scope(self, name: &'static str, description: &'static str) -> OAuthFlow {
            OAuthFlow::from(self).scope(name, description)
        }
    }
    impl OAuthFlow {
        pub fn refreshUrl(mut self, url: &'static str) -> Self {
            match &mut self {
                | OAuthFlow::authorizationCode { refreshUrl, .. }
                | OAuthFlow::implicit { refreshUrl, .. }
                | OAuthFlow::password { refreshUrl, .. }
                | OAuthFlow::clientCredentials { refreshUrl, .. }
                => *refreshUrl = Some(url)
            }
            self
        }
        pub fn scope(mut self, name: &'static str, description: &'static str) -> Self {
            match &mut self {
                | OAuthFlow::authorizationCode { scopes, .. }
                | OAuthFlow::implicit { scopes, .. }
                | OAuthFlow::password { scopes, .. }
                | OAuthFlow::clientCredentials { scopes, .. }
                => scopes.insert(name, description)
            }
            self
        }
    }
    impl From<super::OAuthFlow> for OAuthFlow {
        fn from(it: super::OAuthFlow) -> OAuthFlow {
            match it {
                super::OAuthFlow::authorizationCode {
                    authorizationUrl,
                    tokenUrl,
                } => Self::authorizationCode {
                    authorizationUrl,
                    tokenUrl,
                    refreshUrl:None, scopes:Map::new()
                },
                super::OAuthFlow::implicit {
                    authorizationUrl,
                } => Self::implicit {
                    authorizationUrl,
                    refreshUrl:None, scopes:Map::new()
                },
                super::OAuthFlow::password {
                    tokenUrl,
                } => Self::password {
                    tokenUrl,
                    refreshUrl:None, scopes:Map::new()
                },
                super::OAuthFlow::clientCredentials {
                    tokenUrl,
                } => Self::clientCredentials {
                    tokenUrl,
                    refreshUrl:None, scopes:Map::new()
                },
            }
        }
    }
}

impl SecurityScheme {
    pub fn Basic(scheme_name: &'static str) -> Self {
        Self {
            __name__:  scheme_name,
            auth_type: "http",
            scheme:    Some("basic"),
            name:None, apikey_in:None, openIdConnectUrl:None, flows:None, description:None
        }
    }
    pub fn Bearer(scheme_name: &'static str) -> Self {
        Self {
            __name__:  scheme_name,
            auth_type: "http",
            scheme:    Some("bearer"),
            name:None, apikey_in:None, openIdConnectUrl:None, flows:None, description:None
        }
    }
    pub fn OpenIDConnect(scheme_name: &'static str, url: &'static str) -> Self {
        Self {
            __name__:         scheme_name,
            auth_type:        "openIdConnect",
            openIdConnectUrl: Some(url),
            scheme:None, name:None, apikey_in:None, flows:None, description:None
        }
    }
    pub fn APIKey(scheme_name: &'static str, apiKey: APIKey) -> Self {
        let (name, apikey_in) = match apiKey {
            APIKey::header { name } => (Some(name), Some("header")),
            APIKey::query  { name } => (Some(name), Some("query")),
            APIKey::cookie { name } => (Some(name), Some("cookie")),
        };
        Self {
            __name__:  scheme_name,
            auth_type: "apiKey",
            name,
            apikey_in,
            scheme:None, openIdConnectUrl:None, flows:None, description:None
        }
    }
    pub fn OAuth2(scheme_name: &'static str, flow: impl Into<oauth2::OAuthFlow>) -> Self {
        Self {
            __name__:  scheme_name,
            auth_type: "oauth2",
            flows:     Some(flow.into()),
            openIdConnectUrl: None, scheme:None, name:None, apikey_in:None, description:None
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
}
