#![cfg(feature="rt_lambda")]

#![allow(non_snake_cases, non_camel_case_types)]

//! Based on
//! 
//! * <https://github.com/DefinitelyTyped/DefinitelyTyped/blob/master/types/aws-lambda/trigger/api-gateway-proxy.d.ts>
//! 
//! * <https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html>
//! * <https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-integration-requests.html>

#[cfg(all(feature="ws", not(feature="apigateway")))]
compile_error!("On `rt_lambda`, `ws` can't be activated without `apigateway` !");

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
pub enum LambdaRequest {
    HTTP {
        /* @skip version: "2.0", */
        /* @unused routeKey: "$default", */
        /* @skip rawPath: String, // using requestContext.http.path */
        rawQueryString: String,
        cookies: Vec<String>,
        headers: RequestHeaders,
        #[cfg(feature="apigateway"/* useless in Function URLs because Ohkami howls at the single entry point and handle all */)]
        pathParameters: TupleMap<String, String>,
        /* @skip queryStringParameters, // parsing rawQueryString */
        requestContext: LambdaHTTPRequestContext,
        body: Option<String>,
        isBase64Encoded: bool,
        #[cfg(feature="apigateway")]
        stageVariables: TupleMap<String, String>,
    },
    #[cfg(feature="ws")]
    WebSocket {
        requestContext: LambdaWebSocketRequestContext,
        body: Option<String>,
        isBase64Encoded: bool,
        stageVariables: TupleMap<String, String>
    },
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
    /* @skip domainPrefix: String, */
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
