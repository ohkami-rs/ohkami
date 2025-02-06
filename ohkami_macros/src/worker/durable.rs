pub(super) enum DurableObjectType {
    Fetch,
    Alarm,
    WebSocket,
}
impl syn::parse::Parse for DurableObjectType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        match &*ident.to_string() {
            "fetch" => Ok(Self::Fetch),
            "alarm" => Ok(Self::Alarm),
            "websocket" => Ok(Self::WebSocket),
            _ => Err(syn::Error::new(ident.span(), "must have either 'fetch', 'alarm' or 'websocket' attribute, e.g. #[durable_object(websocket)]"))
        }
    }
}

pub(super) mod bindgen_methods {
    use proc_macro2::TokenStream;
    use quote::quote;

    pub fn core() -> TokenStream {
        quote! {
            #[wasm_bindgen(constructor)]
            pub fn new(
                state: ::worker::worker_sys::DurableObjectState,
                env:   ::worker::Env
            ) -> Self {
                <Self as ::ohkami::DurableObject>::new(
                    ::worker::durable::State::from(state),
                    env
                )
            }

            #[wasm_bindgen(js_name = fetch)]
            pub fn fetch(
                &mut self,
                req: ::worker::worker_sys::web_sys::Request
            ) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::ohkami::DurableObject>::fetch(static_self, req.into()).await
                        .map(worker::worker_sys::web_sys::Response::from)
                        .map(::worker::wasm_bindgen::JsValue::from)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }
        }
    }

    pub fn alarm() -> TokenStream {
        quote! {
            #[wasm_bindgen(js_name = alarm)]
            pub fn alarm(&mut self) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::ohkami::DurableObject>::alarm(static_self).await
                        .map(::worker::worker_sys::web_sys::Response::from)
                        .map(::worker::wasm_bindgen::JsValue::from)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }
        }
    }

    pub fn websocket() -> TokenStream {
        quote! {
            #[wasm_bindgen(js_name = webSocketMessage)]
            pub fn websocket_message(
                &mut self,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                message: ::worker::wasm_bindgen::JsValue
            ) -> ::worker::js_sys::Promise {
                let message = match message.as_string() {
                    Some(message) => ::worker::WebSocketIncomingMessage::String(message),
                    None => ::worker::WebSocketIncomingMessage::Binary(
                        ::worker::js_sys::Uint8Array::new(&message).to_vec()
                    )
                };

                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::ohkami::DurableObject>::websocket_message(static_self, ws.into(), message).await
                        .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }

            #[wasm_bindgen(js_name = webSocketClose)]
            pub fn websocket_close(
                &mut self,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                code: usize,
                reason: String,
                was_clean: bool
            ) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::ohkami::DurableObject>::websocket_close(static_self, ws.into(), code, reason, was_clean).await
                        .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }

            #[wasm_bindgen(js_name = webSocketError)]
            pub fn websocket_error(
                &mut self,
                ws: ::worker::worker_sys::web_sys::WebSocket,
                error: ::worker::wasm_bindgen::JsValue
            ) -> ::worker::js_sys::Promise {
                // SAFETY:
                // Durable Object will never be destroyed while there is still
                // a running promise inside of it, therefore we can let a reference
                // to the durable object escape into a static-lifetime future.
                let static_self: &'static mut Self = unsafe {&mut *(self as *mut _)};

                ::worker::wasm_bindgen_futures::future_to_promise(async move {
                    <Self as ::ohkami::DurableObject>::websocket_error(static_self, ws.into(), error.into()).await
                        .map(|_| ::worker::wasm_bindgen::JsValue::NULL)
                        .map_err(::worker::wasm_bindgen::JsValue::from)
                })
            }
        }
    }
}
