use proc_macro2::{TokenStream, Span};
use syn::{Ident, LitStr};
use quote::quote;

pub enum Binding {
    Variable(String),
    AI,
    D1,
    KV,
    R2,
    Service,
    Queue,
    DurableObject,
}

impl Binding {
    pub fn binding_type(&self) -> Option<&'static str> {
        match self {
            Self::KV => None,
            Self::Variable(_)   => Some("String"),
            Self::AI            => Some("Ai"),
            Self::D1            => Some("D1Database"),
            Self::R2            => Some("R2Bucket"),
            Self::Service       => Some("Fetcher"),
            Self::Queue         => Some("WorkerQueue"),
            Self::DurableObject => Some("DurableObjectNamespace"),
        }
    }

    pub fn tokens_ty(&self) -> TokenStream {
        match self {
            Self::Variable(_)   => quote!(&'static str),
            Self::AI            => quote!(::worker::Ai),
            Self::D1            => quote!(::worker::d1::D1Database),
            Self::KV            => quote!(::worker::kv::KvStore),
            Self::R2            => quote!(::worker::Bucket),
            Self::Queue         => quote!(::worker::Queue),
            Self::Service       => quote!(::worker::Fetcher),
            Self::DurableObject => quote!(::worker::ObjectNamespace),
        }
    }

    pub fn tokens_extract_from_env(&self, name: &Ident) -> TokenStream {
        let name_str = LitStr::new(&name.to_string(), name.span());

        let from_env = |getter: TokenStream| quote! {
            #name: env.#getter?
        };

        match self {
            Self::Variable(value) => quote! { #name: #value },
            Self::AI              => from_env(quote! { ai(#name_str) }),
            Self::D1              => from_env(quote! { d1(#name_str) }),
            Self::KV              => from_env(quote! { kv(#name_str) }),
            Self::R2              => from_env(quote! { bucket(#name_str) }),
            Self::Queue           => from_env(quote! { queue(#name_str) }),
            Self::Service         => from_env(quote! { service(#name_str) }),
            Self::DurableObject   => from_env(quote! { durable_object(#name_str) }),
        }
    }
}

#[derive(serde::Deserialize, Default)]
struct EnvBindingCollection {
    vars:            Option<std::collections::BTreeMap<String, String>>,
    ai:              Option<BindingDeclare>,
    d1_databases:    Option<Vec<BindingDeclare>>,
    kv_namespaces:   Option<Vec<BindingDeclare>>,
    r2_buckets:      Option<Vec<BindingDeclare>>,
    services:        Option<Vec<BindingDeclare>>,
    queues:          Option<QueueProducers>,
    durable_objects: Option<BindingsArray>,

    #[serde(default)]
    env: std::collections::BTreeMap<String, EnvBindingCollection>,
}

#[derive(serde::Deserialize)]
struct BindingDeclare {
    binding: String,
}

#[derive(serde::Deserialize)]
struct QueueProducers {
    producers: Vec<BindingDeclare>,
}

#[derive(serde::Deserialize)]
struct BindingsArray {
    bindings: Vec<BindingName>,
}

#[derive(serde::Deserialize)]
struct BindingName {
    name: String,
}

impl Binding {
    pub fn collect_from_env(env_name: Option<Ident>) -> Result<Vec<(Ident, Self)>, syn::Error> {
        let mut config = super::wrangler::parse_wrangler::<EnvBindingCollection>()
            .map_err(|e| syn::Error::new(Span::call_site(), e))?;
        let config = match env_name.as_ref() {
            None => config,
            Some(name) => {
                let config = config.env.get_mut(&name.to_string())
                    .ok_or_else(|| syn::Error::new(name.span(), format!("env `{name}` is not found in wrangler config")))?;
                std::mem::take(config)
            }
        };

        let mut collection = Vec::new();
        {
            if let Some(vars) = config.vars {
                for (name, value) in vars {
                    collection.push((name, Self::Variable(value)));
                }
            }
            if let Some(BindingDeclare { binding }) = config.ai {
                collection.push((binding, Self::AI));
            }
            if let Some(d1_databases) = config.d1_databases {
                for BindingDeclare { binding } in d1_databases {
                    collection.push((binding, Self::D1));
                }
            }
            if let Some(kv_namespaces) = config.kv_namespaces {
                for BindingDeclare { binding } in kv_namespaces {
                    collection.push((binding, Self::KV));
                }
            }
            if let Some(r2_buckets) = config.r2_buckets {
                for BindingDeclare { binding } in r2_buckets {
                    collection.push((binding, Self::R2));
                }
            }
            if let Some(services) = config.services {
                for BindingDeclare { binding } in services {
                    collection.push((binding, Self::Service));
                }
            }
            if let Some(QueueProducers { producers }) = config.queues {
                for BindingDeclare { binding } in producers {
                    collection.push((binding, Self::Queue));
                }
            }
            if let Some(BindingsArray { bindings }) = config.durable_objects {
                for BindingName { name } in bindings {
                    collection.push((name, Self::DurableObject));
                }
            }   
        }

        collection
            .into_iter()
            .map(|(name, binding)| {
                let name = syn::parse_str(&name).map_err(|_| syn::Error::new(
                    Span::call_site(),
                    format!("can't handle binding name `{name}` as a Rust identifier")
                ))?;
                Ok((name, binding))
            })
            .collect()
    }
}
