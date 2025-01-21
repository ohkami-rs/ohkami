use crate::util;
use proc_macro2::Span;
use syn::{token, Ident, Lit, LitStr};


pub(super) struct WorkerMeta {
    pub(super) title:   LitStr,
    pub(super) version: LitStr,
    pub(super) servers: Vec<Server>,
}

pub(super) struct Server {
    pub(super) url:         LitStr,
    pub(super) description: Option<LitStr>,
    pub(super) variables:   Option<Vec<(LitStr, ServerVariable)>>,
}

pub(super) struct ServerVariable {
    pub(super) r#default:   LitStr,
    pub(super) r#enum:      Option<Vec<LitStr>>,
    pub(super) description: Option<LitStr>,
}

trait TryDefault: Sized {
    fn try_default() -> syn::Result<Self>;
}
const _: (/* TryDefault */) = {
    impl TryDefault for WorkerMeta {
        fn try_default() -> syn::Result<Self> {
            let package_json = {use std::io::Read;
                let mut file = util::find_a_file_in_maybe_workspace("package.json")
                    .map_err(|e| syn::Error::new(Span::call_site(), e.to_string()))?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)
                    .map_err(|e| syn::Error::new(Span::call_site(), e.to_string()))?;
                serde_json::from_str(&buf).ok()
                    .and_then(|j| match j {serde_json::Value::Object(obj) => Some(obj), _ => None})
                    .expect("invalid package.json")
            };

            let wrangler_toml = {use std::io::Read;
                let mut file = util::find_a_file_in_maybe_workspace("wrangler.toml")
                    .map_err(|e| syn::Error::new(Span::call_site(), e.to_string()))?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)
                    .map_err(|e| syn::Error::new(Span::call_site(), e.to_string()))?;
                toml::from_str(&buf).ok()
                    .and_then(|t| match t {toml::Value::Table(t) => Some(t), _ => None})
                    .expect("invalid wrangler.toml")
            };

            let title = LitStr::new(package_json["name"].as_str().unwrap(), Span::call_site());

            let version = LitStr::new(package_json["version"].as_str().unwrap(), Span::call_site());

            let mut servers = vec![
                Server {
                    url:         LitStr::new("http://localhost:8787", Span::call_site()),
                    description: Some(LitStr::new("local dev", Span::call_site())),
                    variables:   None,
                }
            ];
            if let Some(routes) = wrangler_toml.get("routes").and_then(|r| r.as_array()) {
                for route in routes {
                    let pattern = route
                        .as_table()
                        .and_then(|r| r.get("pattern"))
                        .and_then(|p| p.as_str())
                        .expect("invalid `routes` of wrangler.toml")
                        .trim_end_matches(&['/', '*']);
                    servers.push(Server {
                        url:         to_url(pattern),
                        description: None,
                        variables:   None,
                    });
                }
            } else if let Some(route) = wrangler_toml.get("route").and_then(|r| r.as_str()) {
                let route = route.trim_end_matches(&['/', '*']);
                servers.push(Server {
                    url:         to_url(route),
                    description: None,
                    variables:   None,
                });
            };
            fn to_url(route_pattern: &str) -> LitStr {
                if route_pattern.contains("://") {
                    LitStr::new(route_pattern, Span::call_site())
                } else {
                    LitStr::new(&format!("https://{route_pattern}"), Span::call_site())
                }
            }
            if servers.len() == 1 + 1 {
                servers[1].description = Some(LitStr::new("production", Span::call_site()));
            }

            Ok(Self { title, version, servers })
        }
    }

    impl TryDefault for Server {
        fn try_default() -> syn::Result<Self> {
            Ok(Self {
                url:         LitStr::new("/", Span::call_site()),
                description: None,
                variables:   None,
            })
        }
    }

    impl TryDefault for ServerVariable {
        fn try_default() -> syn::Result<Self> {
            Ok(Self {
                r#default:   LitStr::new("", Span::call_site()),
                r#enum:      None,
                description: None,
            })
        }
    }
};

const _: (/* Parse */) = {
    impl syn::parse::Parse for WorkerMeta {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut this = WorkerMeta::try_default()?;

            if !input.is_empty() {
                let meta; syn::braced!(meta in input);
                while let Ok(i) = meta.parse::<Ident>() {
                    match &*i.to_string() {
                        "title" => {
                            let _ = meta.parse::<token::Colon>()?;
                            this.title = meta.parse::<LitStr>()?;
                        }
                        "version" => {
                            let _ = meta.parse::<token::Colon>()?;
                            this.version = match meta.parse::<Lit>()? {
                                Lit::Str(s)   => s,
                                Lit::Int(i)   => LitStr::new(i.base10_digits(), i.span()),
                                Lit::Float(f) => LitStr::new(f.base10_digits(), f.span()),
                                unexpected => return Err(syn::Error::new(
                                    unexpected.span(),
                                    "unexpected `version` value"
                                ))
                            };
                        }
                        "servers" => {
                            let _ = meta.parse::<token::Colon>()?;
                            this.servers = {
                                let servers; syn::bracketed!(servers in meta);
                                servers.parse_terminated(Server::parse, token::Comma)?.into_iter().collect()
                            };
                        }
                        _ => {/* accept any other fields for documentation purpose */
                            let _ = meta.parse::<token::Colon>()?;
                            if meta.peek(token::Brace) {
                                let _object; syn::braced!(_object in meta);
                            } else if meta.peek(Lit) {
                                let _ = meta.parse::<Lit>()?;
                            }
                        }
                    }
                    if meta.peek(token::Comma) {
                        let _ = meta.parse::<token::Comma>()?;
                    }
                }
            }

            Ok(this)
        }
    }

    impl syn::parse::Parse for Server {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut this = Server::try_default()?;

            let tokens; syn::braced!(tokens in input);
            // panic!("tokens = `{}`", tokens.to_string());
            while let Ok(i) = tokens.parse::<Ident>() {
                match &*i.to_string() {
                    "url" => {
                        let _ = tokens.parse::<token::Colon>()?;
                        this.url = tokens.parse::<LitStr>()?;
                    }
                    "description" => {
                        let _ = tokens.parse::<token::Colon>()?;
                        this.description = Some(tokens.parse::<LitStr>()?);
                    }
                    "variables" => {
                        let _ = tokens.parse::<token::Colon>()?;
                        this.variables = Some({
                            let mut variables = Vec::<(LitStr, ServerVariable)>::new();
                            {
                                let vars; syn::braced!(vars in tokens);
                                while let Ok(name) = vars.parse::<Ident>() {
                                    variables.push({
                                        let name = LitStr::new(&name.to_string(), name.span());
                                        let _ = vars.parse::<token::Colon>()?;
                                        let var = vars.parse::<ServerVariable>()?;
                                        (name, var)
                                    });
                                }
                            }
                            variables
                        });
                    }
                    _ => {/* accept any other fields for documentation purpose */
                        let _ = tokens.parse::<token::Colon>()?;
                        if tokens.peek(token::Brace) {
                            let _object; syn::braced!(_object in tokens);
                        } else if tokens.peek(Lit) {
                            let _ = tokens.parse::<Lit>()?;
                        }
                    }
                }
                if tokens.peek(token::Comma) {
                    let _ = tokens.parse::<token::Comma>()?;
                }
            }

            Ok(this)
        }
    }

    impl syn::parse::Parse for ServerVariable {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut this = ServerVariable::try_default()?;

            let tokens; syn::braced!(tokens in input);
            while let Ok(i) = tokens.parse::<Ident>() {
                match &*i.to_string() {
                    "default" => {
                        let _ = tokens.parse::<token::Colon>()?;
                        this.r#default = tokens.parse::<LitStr>()?;
                    }
                    "enum" => {
                        let _ = tokens.parse::<token::Colon>()?;
                        this.r#enum = Some({
                            let variants; syn::bracketed!(variants in tokens);
                            variants.parse_terminated(
                                <LitStr as syn::parse::Parse>::parse,
                                token::Comma
                            )?.into_iter().collect()
                        });
                    }
                    "description" => {
                        let _ = tokens.parse::<token::Colon>()?;
                        this.description = Some(tokens.parse::<LitStr>()?);
                    }
                    _ => {/* accept any other fields for documentation purpose */
                        let _ = tokens.parse::<token::Colon>()?;
                        if tokens.peek(token::Brace) {
                            let _object; syn::braced!(_object in tokens);
                        } else if tokens.peek(Lit) {
                            let _ = tokens.parse::<Lit>()?;
                        }
                    }
                }
                if tokens.peek(token::Comma) {
                    let _ = tokens.parse::<token::Comma>()?;
                }
            }

            Ok(this)
        }
    }
};
