use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{Ident, LitStr};

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

    pub fn collect_from_env(env: &toml::Table) -> Result<Vec<(Ident, Self)>, syn::Error> {
        fn invalid_wrangler_toml() -> syn::Error {
            syn::Error::new(
                Span::call_site(),
                "Invalid wrangler.toml: a binding doesn't have `binding = \"...\"`, or some unexpected structure"
            )
        }

        fn invalid_name(name: &str) -> syn::Error {
            syn::Error::new(
                Span::call_site(),
                format!("Can't bind binding `{name}` into Rust struct field")
            )
        }

        ///////////////////////////////////////////////////////////////////////////////////////////

        fn get_field_as_ident(t: &toml::Table, field: &str) -> Result<Ident, syn::Error> {
            t.get(field)
                .and_then(|b| b.as_str())
                .ok_or_else(invalid_wrangler_toml)
                .and_then(|name| syn::parse_str::<Ident>(name)
                    .map_err(|_| invalid_name(name))
                )
        }

        fn binding_of(t: &toml::Table) -> Result<Ident, syn::Error> {
            get_field_as_ident(t, "binding")
        }
        fn name_of(t: &toml::Table) -> Result<Ident, syn::Error> {
            get_field_as_ident(t, "name")
        }

        fn table_array(a: &toml::value::Array) -> Result<impl IntoIterator<Item = &toml::Table>, syn::Error> {
            a.iter()
                .map(|v| v.as_table().ok_or_else(invalid_wrangler_toml))
                .collect::<Result<Vec<_>, _>>()
        }

        ///////////////////////////////////////////////////////////////////////////////////////////

        let mut bindings = Vec::new();

        if let Some(toml::Value::Table(vars)) = env.get("vars") {
            for (name, value) in vars {
                let name = syn::parse_str(name).map_err(|_| invalid_name(name))?;
                let value = value.as_str()
                    .ok_or_else(|| syn::Error::new(
                        Span::call_site(),
                        "`#[bindings]` doesn't support JSON values in `vars` binding"
                    ))?
                    .to_owned();
                bindings.push((name, Self::Variable(value)))
            }
        }
        if let Some(toml::Value::Table(ai)) = env.get("ai") {
            bindings.push((binding_of(ai)?, Self::AI))
        }
        if let Some(toml::Value::Array(d1_databases)) = env.get("d1_databases") {
            for d1 in table_array(d1_databases)? {
                bindings.push((binding_of(d1)?, Self::D1))
            }
        }
        if let Some(toml::Value::Array(kv_namespaces)) = env.get("kv_namespaces") {
            for kv in table_array(kv_namespaces)? {
                bindings.push((binding_of(kv)?, Self::KV))
            }
        }
        if let Some(toml::Value::Array(r2_buckets)) = env.get("r2_buckets") {
            for r2 in table_array(r2_buckets)? {
                bindings.push((binding_of(r2)?, Self::R2))
            }
        }
        if let Some(toml::Value::Array(services)) = env.get("services") {
            for service in table_array(services)? {
                bindings.push((binding_of(service)?, Self::Service))
            }
        }
        if let Some(toml::Value::Table(queues)) = env.get("queues") {
            if let Some(toml::Value::Array(producers)) = queues.get("producers") {
                for producer in table_array(producers)? {
                    bindings.push((binding_of(producer)?, Self::Queue))
                }
            }
        }
        if let Some(toml::Value::Table(durable_objects)) = env.get("durable_objects") {
            if let Some(toml::Value::Array(durable_object_bindings)) = durable_objects.get("bindings") {
                for durable_object in table_array(durable_object_bindings)? {
                    bindings.push((name_of(durable_object)?, Self::DurableObject))
                }
            }
        }

        Ok(bindings)
    }
}
