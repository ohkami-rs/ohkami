#![cfg(feature="rt_worker")]

pub use ::ohkami_macros::{worker, bindings, DurableObject};

pub trait FromEnv: Sized {
    fn from_env(env: &worker::Env) -> Result<Self, worker::Error>;

    #[doc(hidden)]
    /// list of bindings: `(BindingName, Option<BindingType>)`
    /// 
    /// `Option<BindingType>` is `None` for KV binding and `Some` holding :
    /// 
    /// - `"String"` for Variable binding (ref: https://github.com/cloudflare/workers-rs/blob/38af58acc4e54b29c73336c1720188f3c3e86cc4/worker/src/env.rs#L138-L140)
    /// - `"Ai"` for AI binding (ref: https://github.com/cloudflare/workers-rs/blob/38af58acc4e54b29c73336c1720188f3c3e86cc4/worker/src/ai.rs#L63-L79)
    /// - `"R2Bucket"` for R2 binding (ref: https://github.com/cloudflare/workers-rs/blob/38af58acc4e54b29c73336c1720188f3c3e86cc4/worker/src/r2/mod.rs#L131-L133)
    /// - `"Fetcher"` for Service binding (ref: https://github.com/cloudflare/workers-rs/blob/38af58acc4e54b29c73336c1720188f3c3e86cc4/worker/src/fetcher.rs#L94-L96)
    /// - `"DurableObjectNamespace"` for DurableObject binding (ref: https://github.com/cloudflare/workers-rs/blob/38af58acc4e54b29c73336c1720188f3c3e86cc4/worker/src/durable.rs#L742-L744)
    /// - `"D1Database"` for D1 binding (ref: https://github.com/cloudflare/workers-rs/blob/38af58acc4e54b29c73336c1720188f3c3e86cc4/worker/src/d1/mod.rs#L83-L101)
    fn bindings_meta() -> &'static [(&'static str, Option<&'static str>)] {
        &[]
    }
    #[doc(hidden)]
    fn dummy_env() -> worker::Env {
        use worker::wasm_bindgen::{JsCast, closure::Closure};
        use worker::js_sys::{Object, Reflect, Function};

        let env = Object::new();
        for (binding_name, binding_type) in Self::bindings_meta() {
            let binding = Object::new();
            if let Some(binding_type) = binding_type {
                let constructor = Function::unchecked_from_js(Closure::<dyn Fn()>::new(|| {}).into_js_value());
                {
                    let attributes = Object::new();
                    Reflect::set(&attributes, &"value".into(), &(*binding_type).into()).unwrap();
                    Reflect::define_property(&constructor, &"name".into(), &attributes).unwrap();
                }
                Reflect::set(&binding, &"constructor".into(), &constructor).unwrap();
            }
            Reflect::set(&env, &(*binding_name).into(), &binding).unwrap();
        }
        worker::Env::unchecked_from_js(env.unchecked_into())
    }
}

pub mod bindings {
    /// `Var` binding can also be accessed via associated const
    /// of the same name.
    pub type Var           = &'static str;
    pub type AI            = ::worker::Ai;
    pub type KV            = ::worker::kv::KvStore;
    pub type R2            = ::worker::Bucket;
    pub type Service       = ::worker::Fetcher;
    pub type DurableObject = ::worker::ObjectNamespace;
    pub type D1            = ::worker::d1::D1Database;
    /// `Queue` may cause a lot of *WARNING*s on `npm run dev`, but
    /// it's not an actual problem and `Queue` binding does work.
    pub type Queue         = ::worker::Queue;
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub trait has_DurableObject_attribute {}

/// **Note:** Implement this trait with a standard `impl DurableObject for YourType` block, but in order to
/// integrate them with the Workers Runtime, you must also add the **`#[DurableObject]`** attribute
/// to the struct.
/// 
/// ### Example
/// 
/// ```no_run
/// use ohkami::DurableObject;
/// 
/// # struct User;
/// # struct Message;
/// 
/// #[DurableObject]
/// pub struct Chatroom {
///     users: Vec<User>,
///     messages: Vec<Message>,
///     state: worker::State,
///     env: worker::Env, // access `Env` across requests, use inside `fetch`
/// }
/// 
/// impl DurableObject for Chatroom {
///     fn new(state: worker::State, env: worker::Env) -> Self {
///         Self {
///             users: vec![],
///             messages: vec![],
///             state,
///             env,
///         }
///     }
/// 
///     async fn fetch(&mut self, req: worker::Request) -> worker::Result<worker::Response> {
///         // do something when a worker makes a request to this DO
///         worker::Response::ok(&format!("{} active users.", self.users.len()))
///     }
/// }
/// ```
#[allow(async_fn_in_trait/* `Send` is not needed */)] 
pub trait DurableObject: has_DurableObject_attribute {
    fn new(state: worker::State, env: worker::Env) -> Self;
    
    async fn fetch(&mut self, req: worker::Request) -> worker::Result<worker::Response>;
    
    async fn alarm(&mut self) -> worker::Result<worker::Response> {
        worker::console_error!("alarm() handler is not implemented");
        Err(worker::Error::RustError("alarm() handler is not implemented".into()))
    }
    
    #[allow(unused_variables)]
    async fn websocket_message(
        &mut self,
        ws: worker::WebSocket,
        message: worker::WebSocketIncomingMessage,
    ) -> worker::Result<()> {
        Ok(())
    }
    
    #[allow(unused_variables)]
    async fn websocket_close(
        &mut self,
        ws: worker::WebSocket,
        code: usize,
        reason: String,
        was_clean: bool,
    ) -> worker::Result<()> {
        Ok(())
    }
    
    #[allow(unused_variables)]
    async fn websocket_error(
        &mut self,
        ws: worker::WebSocket,
        error: worker::Error,
    ) -> worker::Result<()> {
        Ok(())
    }
}
