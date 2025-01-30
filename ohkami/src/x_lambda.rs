#![cfg(feature="rt_lambda")]

#![allow(non_snake_cases, non_camel_case_types)]

#[cfg(all(feature="ws", not(feature="apigateway")))]
compile_error!("On `rt_lambda`, `ws` can't be activated without `apigateway` !");

/*

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

*/
use crate::util::ErrorMessage;
use std::{future::Future, marker::PhantomData};

pub struct LambdaWebSocket<E: TryFrom<LambdaWebSocketEvent, std::error::Error> = LambdaWebSocketEvent> {
    pub context: internal::LambdaWebSocketRequestContext,
    pub event: E,
    client: Option<tokio::net::TcpStream>
}

impl<E: TryFrom<LambdaWebSocketEvent, std::error::Error>> LambdaWebSocket<E> {
    pub async fn send(&mut self, data: ) -> Result<(), impl std::error::Error> {

    }

    pub async fn handle<F, Fut>(h: F) ->
        impl lambda_runtime::Service<
            lambda_runtime::LambdaEvent<internal::LambdaWebSocketRequest>,
            Response = internal::LambdaResponse
        >
    where
        F:   Fn(E) -> Fut,
        Fut: Future<Output = Result<(), lambda_runtime::Error>>,
    {
        use lambda_runtime::{Service, LambdaEvent};
        use internal::{LambdaWebSocketRequest, LambdaResponse};
        use std::{task};

        struct LambdaWebSocketService<F, Fut> {
            handler: F,
            __fut__: PhantomData<Fut>,
            // __event__: PhantomData<E>,
        }

        impl Service<LambdaEvent<LambdaWebSocketRequest>, E, F, Fut> for LambdaWebSocketService<F, Fut>
        where
            F:   Fn(E) -> Fut,
            E:   TryFrom<LambdaWebSocketEvent, std::error::Error>,
            Fut: Future<Output = Result<(), lambda_runtime::Error>>,
        {
            type Response = LambdaResponse;
            type Error = lambda_runtime::Error;
            type Future = impl Future<Output = Result<(), lambda_runtime::Error>>;

            fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
                task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: LambdaEvent<LambdaWebSocketRequest>) -> Self::Future {
                let payload: internal::LambdaWebSocketRequest = req.payload;
                
                let e = LambdaWebSocketEvent {
                    event: match &payload.requestContext.eventType {
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
                    },
                    context: payload.requestContext,
                };

                (self.handler)(E::try_from(e)?)
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
        pub cookies: Option<Vec<String>>,
        pub body: Option<String>,
        pub isBase64Encoded: Option<bool>,
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum LambdaHTTPRequest {
        /* @skip version: "2.0", */
        /* @unused routeKey: "$default", */
        /* @skip rawPath: String, // using requestContext.http.path */
        pub rawQueryString: String,
        pub cookies: Vec<String>,
        pub headers: RequestHeaders,
        #[cfg(feature="apigateway"/* useless in Function URLs because Ohkami howls at the single entry point and handle all */)]
        pub pathParameters: TupleMap<String, String>,
        /* @skip queryStringParameters, // parsing rawQueryString */
        pub requestContext: LambdaHTTPRequestContext,
        pub body: Option<String>,
        pub isBase64Encoded: bool,
        #[cfg(feature="apigateway")]
        pub stageVariables: TupleMap<String, String>,
    }

    #[cfg(feature="ws")]
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
        #[cfg(feature="apigateway")]
        pub authentication: Option<LambdaRequestAuthentication>,
        pub authorizer: Option<LambdaRequestAuthorizer>,
        #[cfg(feature="apigateway")]
        pub domainName: String,
        /* @skip domainPrefix: String, // domainName is enough */
        pub http: LambdaHTTPRequestDetails,
        pub requestId: String,
        /* @unused routeKey: "$default", */
        /* @unused stage: "$default", */
        /* @skip time: String, // timeEpoch is enough */
        pub timeEpoch: u64,
    }

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

    #[cfg(feature="apigateway")]
    #[derive(Deserialize)]
    pub struct LambdaRequestAuthentication {
        pub clientCertPem: String,
        pub issuerDN: String,
        pub subjectDN: String,
        pub serialNumber: String,
        pub validity: LambdaRequestAuthenticationValidity,
    }
    #[cfg(feature="apigateway")]
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
            /* cognitoIdentity */
            principalOrgId: String,
            userArn: String,
            userId: String,
        },
        #[cfg(feature="apigateway")]
        jwt {
            claims: JsonMap,
            scopes: Vec<String>,
        },
    }
}
