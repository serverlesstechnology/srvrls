use std::collections::HashMap;

use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use serde_json::Value;

use crate::components::HttpMethod;

/// This replaces the inbound `Request` and `Context` entity with simpler, opinionated methods.
pub struct SrvrlsRequest {
    /// All query parameters in a map by key value.
    pub query_parameters: HashMap<String, Vec<String>>,
    /// All path parameters in a map by position withing the proxy path parameter field.
    pub path_parameters: HashMap<i32, String>,
    /// All String claims within the authorizer field.
    pub string_claims: HashMap<String, String>,
    /// All Numeric (i64) claims within the authorizer field.
    pub integer_claims: HashMap<String, i64>,
    /// The `HttpMethod` of the request.
    pub method: HttpMethod,
    /// The request payload, or empty String if none exists.
    pub body: String,
}

impl SrvrlsRequest {
    /// Provides the path parameter as a String, if the parameter is missing an empty string will be
    /// returned in its' stead.
    pub fn path_parameter(&self, position: i32) -> String {
        match &self.path_parameters.get(&position) {
            None => "".to_string(),
            Some(val) => val.to_string(),
        }
    }

    /// Returns a `Vec<String>` for a requested query parameter
    pub fn query_parameter(&self, key: &str) -> Vec<String> {
        match self.query_parameters.get(key) {
            None => Vec::new(),
            Some(v) => v.clone(),
        }
    }

    /// This provides access to authentication claims (in AWS Lambda Proxy calls) that are `String`s.
    /// This signature is likely to change with Azure and Google Cloud Function implemenations.
    pub fn authentication_claim(&self, claim: &str) -> String {
        match self.string_claims.get(claim) {
            None => "".to_string(),
            Some(value) => value.clone(),
        }
    }
}

impl From<ApiGatewayProxyRequest> for SrvrlsRequest {
    fn from(event: ApiGatewayProxyRequest) -> Self {
        let mut path_parameters = HashMap::new();
        let path_url = &event.path_parameters["proxy"];
        let path_iter: Vec<&str> = path_url.split('/').collect();
        for (i, path_segment) in path_iter.iter().enumerate() {
            path_parameters.insert(i as i32, path_segment.to_string());
        }
        let mut query_string_parameters = event.multi_value_query_string_parameters;
        for (k, v) in event.query_string_parameters {
            query_string_parameters.insert(k, vec![v]);
        }
        let method = match event.http_method {
            None => HttpMethod::OTHER,
            Some(method) => {
                match method.as_str() {
                    "GET" => HttpMethod::GET,
                    "POST" => HttpMethod::POST,
                    "PUT" => HttpMethod::PUT,
                    "HEAD" => HttpMethod::HEAD,
                    "DELETE" => HttpMethod::DELETE,
                    _ => HttpMethod::OTHER,
                }
            }
        };
        let body = match event.body {
            None => "".to_string(),
            Some(body) => body.clone(),
        };
        let mut string_claims = HashMap::new();
        let mut integer_claims = HashMap::new();
        match event.request_context.authorizer.get("claims") {
            None => {}
            Some(claims) => {
                match serde_json::to_value(claims).unwrap() {
                    Value::Null |
                    Value::Bool(_) |
                    Value::Number(_) |
                    Value::String(_) |
                    Value::Array(_) => { panic!() }
                    Value::Object(claims) => {
                        for (k, v) in claims {
                            match v {
                                Value::Null => {}
                                Value::Bool(_) => {}
                                Value::Number(number_value) => {
                                    if number_value.is_i64() {
                                        integer_claims.insert(k, number_value.as_i64().unwrap());
                                    }
                                }
                                Value::String(string_value) => {
                                    string_claims.insert(k, string_value);
                                }
                                Value::Array(_) => {}
                                Value::Object(_) => {}
                            };
                        }
                    }
                };
            }
        };

        SrvrlsRequest {
            path_parameters,
            string_claims,
            integer_claims,
            query_parameters: query_string_parameters,
            method,
            body,
        }
    }
}
