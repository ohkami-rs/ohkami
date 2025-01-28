#![cfg(feature="rt_lambda")]

//! Based on <https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format>

use crate::{Method, request::RequestHeaders};
use ohkami_lib::TupleMap;
use serde_json::Map as JsonMap;

#[derive(Deserialize)]
#[allow(non_snake_cases)]
pub struct LambdaRequest {
    /* @skip version: "2.0", */
    /* @unused routeKey: "$default", */
    /* @skip rawPath: String, // use requestContext.http.path */
    rawQueryString: String,
    cookies: Vec<String>,
    headers: RequestHeaders,
    /* @skip queryStringParameters */
    requestContext: LambdaRequestContext,
    body: Option<String>,
    #[cfg(feature="apigateway"/* useless in Function URLs because Ohkami howls at the single entry point and handle all */)]
    pathParameters: TupleMap<String, String>,
    isBase64Encoded: bool,
    #[cfg(feature="apigateway")]
    stageVariables: TupleMap<String, String>,
}

#[derive(Deserialize)]
#[allow(non_snake_cases)]
struct LambdaRequestContext {
    /* @skip accountId: String, */
    apiId: String,
    #[cfg(feature="apigateway")]
    authentication: Option<LambdaRequestAuthentication>,
    authorizer: Option<LambdaRequestAuthorizer>,
    domainName: String,
    /* @skip domainPrefix: String, */
    http: LambdaRequestHTTP,
    requestId: String,
    /* @unused routeKey: "$default" */
    /* @unused stage: "$default" */
    /* @skip time: String, */
    timeEpoch: u64,
}

#[cfg(feature="apigateway")]
#[derive(Deserialize)]
#[allow(non_snake_cases)]
struct LambdaRequestAuthentication {
    clientCertPem: String,
    subjectDN: String,
    issuerDN: String,
    serialNumber: String,
    validity: LambdaRequestAuthenticationValidity,
}
#[cfg(feature="apigateway")]
#[derive(Deserialize)]
#[allow(non_snake_cases)]
struct LambdaRequestAuthenticationValidity {
    notAfter: String,
    notBefore: String,
}

#[derive(Deserialize)]
#[allow(non_snake_cases, non_camel_case_types)]
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

#[derive(Deserialize)]
#[allow(non_snake_cases)]
struct LambdaRequestHTTP {
    method: Method,
    path: String,
    /* @skip protocol */
    sourceIp: std::net::IpAddr,
    /* @skip userAgent */
}
