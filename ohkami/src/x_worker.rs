#![cfg(feature="rt_worker")]

pub use ::ohkami_macros::{worker, bindings, DurableObject};
    
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
