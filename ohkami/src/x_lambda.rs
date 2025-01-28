#![cfg(feature="rt_lambda")]

//! Based on <https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html#http-api-develop-integrations-lambda.proxy-format>

use crate::{Method, request::RequestHeaders};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(non_snake_cases)]
pub struct LambdaRequest {
    /* @skip version: "2.0", */
    /* @lambda-unuse routeKey: "$default", */
    rawPath: String,
    rawQueryString: String,
    cookies: Vec<String>,
    headers: RequestHeaders,
    /* @skip queryStringParameters */
    requestContext: LambdaRequestContext,
}

#[derive(Deserialize)]
#[allow(non_snake_cases)]
struct LambdaRequestContext {
    accountId: String,
    apiId: String,
    /* @lambda-unuse authentication: null, */
    authorizer: Option<LambdaRequestAuthorizer>,
    domainName: String,
    domainPrefix: String,
    http: LambdaRequestHTTP,
    requestId: String,
    /* @lambda-unuse routeKey: $default */
    /* @lambda-unuse stage: $default */
    time: String,
    timeEpoch: u64,
    body: Option<String>,
    /* @skip pathParameters */
    isBase64Encoded: bool,
    /* @lambda-unuse stageVariables */
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
    } /* that's it in Function URLs (unlike API Gateway) */
}

#[derive(Deserialize)]
#[allow(non_snake_cases)]
struct LambdaRequestHTTP {
    method: Method,
    /* @skip path: */
    /* @skip protocol: */
    sourceIp: std::net::IpAddr,
    /* @skip userAgent */
}
