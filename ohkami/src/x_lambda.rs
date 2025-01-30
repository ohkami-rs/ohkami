#![cfg(feature="rt_lambda")]

#![allow(non_snake_cases, non_camel_case_types)]

#[cfg(all(feature="ws", not(feature="apigateway")))]
compile_error!("On `rt_lambda`, `ws` can't be activated without `apigateway` !");

pub(crate) use internal::*;
/// Internal interfances between Lambda Events.
/// 
/// Based on :
/// 
/// * <https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/aws-lambda/trigger/api-gateway-proxy.d.ts>
/// * <https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html>
/// * <https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-integration-requests.html>
pub(crate) mod internal {
    use crate::{Method, request::RequestHeaders, response::ResponseHeaders};
    use ohkami_lib::TupleMap;
    use serde_json::Map as JsonMap;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize)]
    pub struct LambdaResponse {
        pub statusCode: u16,
        pub headers: ResponseHeaders,
        #[serde(skip_serialize_if = "Option::is_none")]
        pub cookies: Option<Vec<String>>,
        #[serde(skip_serialize_if = "Option::is_none")]
        pub body: Option<String>,
        #[serde(skip_serialize_if = "Option::is_none")]
        pub isBase64Encoded: Option<bool>,
    }

    #[derive(Deserialize)]
    pub struct LambdaHTTPRequest {
        /* @skip version: "2.0", */
        /* @unused routeKey: "$default", */
        /* @skip rawPath: String, // using requestContext.http.path */
        pub rawQueryString: String,
        pub cookies: Vec<String>,
        pub headers: RequestHeaders,
        /* @skip pathParameters: TupleMap<String, String>, */
        /* @skip queryStringParameters, // parsing rawQueryString */
        pub requestContext: LambdaHTTPRequestContext,
        pub body: Option<String>,
        pub isBase64Encoded: bool,
        pub stageVariables: TupleMap<String, String>,
    }

    #[cfg(feature="ws")]
    #[derive(Deserialize)]
    pub struct LambdaWebSocketRequest {
        pub requestContext: LambdaWebSocketRequestContext,
        pub body: Option<String>,
        pub isBase64Encoded: bool,
        pub stageVariables: TupleMap<String, String>,
    }

    #[derive(Deserialize)]
    pub struct LambdaHTTPRequestContext {
        /* @skip accountId: String, */
        pub apiId: String,
        pub authentication: Option<LambdaRequestAuthentication>,
        pub authorizer: Option<LambdaRequestAuthorizer>,
        pub domainName: String,
        /* @skip domainPrefix: String, // domainName is enough */
        pub http: LambdaHTTPRequestDetails,
        pub requestId: String,
        /* @unused routeKey: "$default", */
        /* @unused stage: "$default", */
        /* @skip time: String, // timeEpoch is enough */
        pub timeEpoch: u64,
    }

    #[cfg(feature="ws")]
    #[derive(Deserialize)]
    pub struct LambdaWebSocketRequestContext {
        pub apiId: String,
        /* @skip connectedAt: u64, */
        pub connectionId: String,
        pub domainName: String,
        pub eventType: LambdaWebSocketEventType,
        /* @skip extendedRequestId: String, */
        pub routeKey: String,
        /* @skip messageDirection: "IN", */
        pub messageId: String,
        pub requestId: String,
        /* @skip requestTime: String, // requestTimeEpoch is enough */
        pub requestTimeEpoch: u64,
        pub stage: String,
    }

    #[derive(Deserialize)]
    pub struct LambdaHTTPRequestDetails {
        pub method: Method,
        pub path: String,
        /* @skip protocol: String, */
        pub sourceIp: std::net::IpAddr,
        /* @skip userAgent: String, */
    }

    #[cfg(feature="ws")]
    #[derive(Deserialize)]
    pub enum LambdaWebSocketEventType {
        CONNECT,
        DISCONNECT,
        MESSAGE,
    }

    #[derive(Deserialize)]
    pub struct LambdaRequestAuthentication {
        pub clientCertPem: String,
        pub issuerDN: String,
        pub subjectDN: String,
        pub serialNumber: String,
        pub validity: LambdaRequestAuthenticationValidity,
    }
    #[derive(Deserialize)]
    pub struct LambdaRequestAuthenticationValidity {
        pub notAfter: String,
        pub notBefore: String,
    }

    #[derive(Deserialize)]
    pub enum LambdaRequestAuthorizer {
        iam {
            accessKey: String,
            accountId: String,
            callerId: String,
            /* @unused cognitoIdentity */
            principalOrgId: String,
            userArn: String,
            userId: String,
        },
        jwt {
            claims: JsonMap,
            scopes: Vec<String>,
        },
    }
}

#[cfg(feature="ws")]
pub use ws::*;
#[cfg(feature="ws")]
mod ws {
    use super::internal;
    use crate::util::ErrorMessage;
    use std::{future::Future, marker::PhantomData};

    struct Client {
        host: String,
        path: String,
        conn: tokio::net::TcpStream,
    }
    struct ClientInit {
        domain_name: &str,
        stage: &str,
        connection_id: &str,
    }
    impl Client {
        /// Create backend client based on
        /// <https://docs.aws.amazon.com/en_us/apigateway/latest/developerguide/apigateway-how-to-call-websocket-api-connections.html>
        async fn new(init: ClientInit) -> Result<Self, impl std::error::Error> {
            use ::ohkami_lib::percent_encode;

            let conn = tokio::net::TcpStream::connect(init.domain_name).await?;
            let host = init.domain_name.to_owned();
            let path = format!(
                "/{stage}/%40connections/{connection_id}",
                stage = percent_encode(init.stage),
                connection_id = percent_encode(init.connection_id)
            );
            Ok(Self { host, conn })
        }

        async fn fetch(
            &mut self,
            method: &'static str,
            body: Option<&[u8]>,
        ) -> Result<(), impl std::error::Error> {
            use ohkami_lib::num::itoa;
            use tokio::io::AsyncWriteExt;

            let mut request = Vec::with_capacity(
                method.len() + " ".len() + self.path.len() + " HTTP/1.1\r\n".len() +
                "host: ".len() + self.host.len() + "\r\n".len() +
                "\r\n".len() +
                body.as_ref().map(|b|
                    "content-length: ".len() + (1 + b.len().ilog10()) + "\r\n".len() +
                    b.len()
                ).unwrap_or(0)
            );
            {
                request.push(method.as_bytes());
                request.push(b" ");
                request.push(self.path.as_bytes());
                request.push(b" HTTP/1.1\r\n");
                {
                    request.push(b"host: ");
                    request.push(self.host.as_bytes());
                    request.push(b"\r\n");
                }
                if let Some(ref body) = body {
                    request.push(b"content-length: ");
                    request.push(itoa(body.len()).as_bytes());
                    request.push(b"\r\n");
                }
                request.push(b"\r\n");
                if let Some(body) = body {
                    request.push(body);
                }
            }

            self.conn.write_all(request).await?;

            Ok(())
        }
    }

    /// ```no_run
    /// use ohkami::{LambdaWebSocket, LambdaWebSocketMESSAGE};
    /// use lambda_runtime::Error;
    /// 
    /// #[ohkami::lambda]
    /// async fn main() -> Result<(), Error> {
    ///     lambda_runtime::run(LambdaWebSocket::handle(echo)).await
    /// }
    /// 
    /// async fn echo(
    ///     ws: LambdaWebSocket<LamdaWebSocketMESSAGE>
    /// ) -> Result<(), Error> {
    ///     ws.send(ws.event).await?;
    ///     Ok(())
    /// }
    /// ```
    pub struct LambdaWebSocket<E: TryFrom<LambdaWebSocketEvent, std::error::Error> = LambdaWebSocketEvent> {
        pub context: internal::LambdaWebSocketRequestContext,
        pub event: E,
        client: Client,
    }

    impl<E: TryFrom<LambdaWebSocketEvent, std::error::Error>> LambdaWebSocket<E> {
        async fn new(
            context: internal::LambdaWebSocketRequestContext,
            event: E,
        ) -> Result<Self, impl std::error::Error> {
            let client = Client::new(ClientInit {
                domain_name: &context.domainName,
                stage: &context.stage,
                connection_id: &context.connectionId,
            }).await?;

            Ok(Self {
                context,
                event,
                client
            })
        }

        pub async fn send(&mut self, data: impl Into<LambdaWebSocketMESSAGE>) -> Result<(), impl std::error::Error> {
            self.client().await?.fetch("POST", Some(match data.into() {
                LambdaWebSocketMESSAGE::Text(t) => t.as_bytes(),
                LambdaWebSocketMESSAGE::Binary(b) => b.as_bytes()
            })).await
        }
        pub async fn close(mut self) -> Result<(), impl std::error::Error> {
            self.client().await?.fetch("DELETE", None).await
        }

        pub async fn handle<F, Fut>(handler: F) ->
            impl lambda_runtime::Service<
                lambda_runtime::LambdaEvent<internal::LambdaWebSocketRequest>,
                Response = lambda_runtime::FunctionResponse<
                    internal::LambdaResponse,
                    std::pin::Pin<Box<dyn ohkami_lib::Stream<Item = Result<String, std::convert::Infallible>> + Send>>
                >
            >
        where
            F:   Fn(Self) -> Fut,
            Fut: Future<Output = Result<(), lambda_runtime::Error>>,
        {
            return LambdaWebSocketService {
                handler,
                __fut__: PhantomData
            };

            ///////////////////////////////////////////////////////

            use lambda_runtime::{Service, LambdaEvent};
            use internal::{LambdaWebSocketRequest, LambdaResponse};

            struct LambdaWebSocketService<F, Fut> {
                handler: F,
                __fut__: PhantomData<Fut>,
            }

            impl Service<LambdaEvent<LambdaWebSocketRequest>, E, F, Fut> for LambdaWebSocketService<F, Fut>
            where
                F:   Fn(LambdaWebSocket<E>) -> Fut,
                E:   TryFrom<LambdaWebSocketEvent, std::error::Error>,
                Fut: Future<Output = Result<(), lambda_runtime::Error>>,
            {
                type Response = lambda_runtime::FunctionResponse<
                    internal::LambdaResponse,
                    std::pin::Pin<Box<dyn ohkami_lib::Stream<Item = Result<String, std::convert::Infallible>> + Send>>
                >;
                type Error = lambda_runtime::Error;
                type Future = impl Future<Output = Result<(), lambda_runtime::Error>>;

                fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
                    std::task::Poll::Ready(Ok(()))
                }

                fn call(&mut self, req: LambdaEvent<LambdaWebSocketRequest>) -> Self::Future {
                    let payload: internal::LambdaWebSocketRequest = req.payload;

                    let event = match &payload.requestContext.eventType {
                        internal::LambdaWebSocketEventType::CONNECT => {
                            LambdaWebSocketEvent::CONNECT(LambdaWebSocketCONNECT)
                        }
                        internal::LambdaWebSocketEventType::DISCONNECT => {
                            LambdaWebSocketEvent::DISCONNECT(LambdaWebSocketDISCONNECT)
                        }
                        internal::LambdaWebSocketEventType::MESSAGE => {
                            let body = payload.body
                                .ok_or_else(|| Box::new(ErrorMessage("Got MESSAGE event, but not `body` found".into())))?;
                            let body = if payload.isBase64Encoded {
                                use ::base64::engine::{Engine as _, general_purpose::STANDARD as BASE64};
                                LambdaWebSocketMESSAGE::Binary(BASE64.decode(body)?)
                            } else {
                                LambdaWebSocketMESSAGE::Text(body)
                            };
                            LambdaWebSocketEvent::MESSAGE(body)
                        }
                    };

                    async move {
                        let ws = LambdaWebSocket::new(
                            payload.requestContext,
                            E::try_from(event)?
                        ).await?;

                        (self.handler)(ws).await?;

                        Ok(lambda_runtime::FunctionResponse::BufferedResponse(internal::LambdaResponse {
                            statusCode: 200,
                            headers: ResponseHeaders::new(),
                            cookies: None,
                            body: None,
                            isBase64Encoded: None,
                        }))
                    }
                }
            }
        }
    }

    pub enum LambdaWebSocketEvent {
        CONNECT(LambdaWebSocketCONNECT),
        DISCONNECT(LambdaWebSocketDISCONNECT),
        MESSAGE(LambdaWebSocketMESSAGE),
    }
    impl TryFrom<LambdaWebSocketEvent> for LambdaWebSocketEvent {
        type Error = std::convert::Infallible;
        fn try_from(e: LambdaWebSocketEvent) -> Result<Self, Self::Error> {
            Ok(e)
        }
    }

    pub struct LambdaWebSocketCONNECT;
    impl TryFrom<LambdaWebSocketEvent> for LambdaWebSocketCONNECT {
        type Error = ErrorMessage;
        fn try_from(e: LambdaWebSocketEvent) -> Result<Self, Self::Error> {
            match e {
                LambdaWebSocketEvent::CONNECT(it) => Ok(it),
                LambdaWebSocketEvent::DISCONNECT(_) => Err(ErrorMessage(
                    "Expected CONNECT event, but got DISCONNECT".into()
                )),
                LambdaWebSocketEvent::MESSAGE(_) => Err(ErrorMessage(
                    "Expected CONNECT event, but got MESSAGE".into()
                )),
            }
        }
    }

    pub struct LambdaWebSocketDISCONNECT;
    impl TryFrom<LambdaWebSocketEvent> for LambdaWebSocketDISCONNECT {
        type Error = ErrorMessage;
        fn try_from(e: LambdaWebSocketEvent) -> Result<Self, Self::Error> {
            match e {
                LambdaWebSocketEvent::DISCONNECT(it) => Ok(it),
                LambdaWebSocketEvent::MESSAGE(_) => Err(ErrorMessage(
                    "Expected DISCONNECT event, but got MESSAGE".into()
                )),
                LambdaWebSocketEvent::CONNECT(_) => Err(ErrorMessage(
                    "Expected DISCONNECT event, but got CONNECT".into()
                )),
            }
        }
    }

    pub enum LambdaWebSocketMESSAGE {
        Text(String),
        Binary(Vec<u8>),
    }
    impl TryFrom<LambdaWebSocketEvent> for LambdaWebSocketMESSAGE {
        type Error = ErrorMessage;
        fn try_from(e: LambdaWebSocketEvent) -> Result<Self, Self::Error> {
            match e {
                LambdaWebSocketEvent::MESSAGE(it) => Ok(it),
                LambdaWebSocketEvent::CONNECT(_) => Err(ErrorMessage(
                    "Expected MESSAGE event, but got CONNECT".into()
                )),
                LambdaWebSocketEvent::DISCONNECT(_) => Err(ErrorMessage(
                    "Expected MESSAGE event, but got DISCONNECT".into()
                )),
            }
        }
    }
    const _: () = {
        impl From<String> for LambdaWebSocketMESSAGE {
            fn from(text: String) -> Self {Self::Text(text)}
        }
        impl From<&str> for LambdaWebSocketMESSAGE {
            fn from(text: &str) -> Self {Self::Text(text.to_owned())}
        }
        impl From<std::borrow::Cow<str>> for LambdaWebSocketMESSAGE {
            fn from(text: String) -> Self {Self::Text(text.into())}
        }

        impl From<Vec<u8>> for LambdaWebSocketMESSAGE {
            fn from(binary: Vec<u8>) -> Self {Self::binary(binary)}
        }
        impl From<&[u8]> for LambdaWebSocketMESSAGE {
            fn from(binary: &[u8]) -> Self {Self::binary(binary.to_owned())}
        }
        impl From<std::borrow::Cow<[u8]>> for LambdaWebSocketMESSAGE {
            fn from(binary: std::borrow::<[u8]>) -> Self {Self::binary(binary.into())}
        }
    };
}
