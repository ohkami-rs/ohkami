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
    statusCode: u16,
    headers: ResponseHeaders,
    cookies: Vec<String>,
    body: Option<String>,
    isBase64Encoded: bool,
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
struct LambdaHTTPRequestContext {
    /* @skip accountId: String, */
    apiId: String,
    #[cfg(feature="apigateway")]
    authentication: Option<LambdaRequestAuthentication>,
    authorizer: Option<LambdaRequestAuthorizer>,
    domainName: String,
    /* @skip domainPrefix: String, */
    http: LambdaHTTPRequestDetails,
    requestId: String,
    /* @unused routeKey: "$default", */
    /* @unused stage: "$default", */
    /* @skip time: String, // timeEpoch is enough */
    timeEpoch: u64,
}

#[derive(Deserialize)]
struct LambdaWebSocketRequestContext {
    apiId: String,
    /* @skip connectedAt: u64, */
    connectionId: String,
    /* @skip domainName: String, */
    eventType: LambdaWebSocketEventType,
    /* @skip extendedRequestId: String, */
    routeKey: String,
    /* @skip messageDirection: "IN", */
    messageId: String,
    requestId: String,
    /* @skip requestTime: String, // requestTimeEpoch is enough */
    requestTimeEpoch: u64,
    stage: String,
}

#[derive(Deserialize)]
struct LambdaHTTPRequestDetails {
    method: Method,
    path: String,
    /* @skip protocol: String, */
    sourceIp: std::net::IpAddr,
    /* @skip userAgent: String, */
}

#[cfg(feature="ws")]
#[derive(Deserialize)]
enum LambdaWebSocketEventType {
    CONNECT,
    DISCONNECT,
    MESSAGE,
}

#[cfg(feature="apigateway")]
#[derive(Deserialize)]
struct LambdaRequestAuthentication {
    clientCertPem: String,
    issuerDN: String,
    subjectDN: String,
    serialNumber: String,
    validity: LambdaRequestAuthenticationValidity,
}
#[cfg(feature="apigateway")]
#[derive(Deserialize)]
struct LambdaRequestAuthenticationValidity {
    notAfter: String,
    notBefore: String,
}

#[derive(Deserialize)]
enum LambdaRequestAuthorizer {
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
