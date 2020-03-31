use std::collections::HashMap;

use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use serde_json::Value;

pub use crate::application::{Srvrls,SrvrlsApplication};
pub use crate::response::SrvrlsResponse;

mod application;
mod response;

/// Replaces a String match with an enum that only includes the most common `HttpMethod`s. This
/// reduces overhead from trash methods like `CONNECT`, `OPTIONS` or `TRACE` as well as non-legit
/// codes that are possible with `Strings`.
/// Seriously, if you're using one of those you're probably just trolling your users.
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    OTHER,
}

/// You can always return the precise error response, but using the specific error allows a much
/// cleaner rolloff using the `?` operator. All of these translate directly to their respective
/// 4xx or 5xx HTTP responses.
#[derive(Debug, Clone, PartialEq)]
pub enum SrvrlsError {
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
}

/// This replaces the inbound `Request` and `Context` entity with simpler, opinionated methods.
// TODO implement From<ApiGatewayProxyRequest> ??
pub struct SrvrlsRequest {
    event: ApiGatewayProxyRequest,
    path_parameters: HashMap<i32, String>,
}

impl SrvrlsRequest {
    // TODO remove public access after providing test options
    pub fn new(event: ApiGatewayProxyRequest) -> Self {
        let mut path_parameters = HashMap::new();
        let path_url = &event.path_parameters["proxy"];
        let path_iter: Vec<&str> = path_url.split("/").collect();
        for (i, path_segment) in path_iter.iter().enumerate() {
            path_parameters.insert(i as i32, path_segment.to_string());
        }
        SrvrlsRequest {
            event,
            path_parameters,
        }
    }

    /// Provides the path parameter as a String, if the parameter is missing an empty string will be
    /// returned in its' stead.
    pub fn path_parameter(&self, position: i32) -> String {
        match &self.path_parameters.get(&position) {
            None => "".to_string(),
            Some(val) => val.to_string(),
        }
    }

    /// Returns a fully owned Hashmap containing the one-to-one query key-value pairs
    pub fn query_parameters(&self) -> HashMap<String, String> {
        let parameters = &self.event.query_string_parameters;
        parameters.clone()
    }

    /// Returns the `HttpMethod` for the most common parameters and `HttpMethod::Other` when a rare
    /// or unknown method is passed.
    pub fn method(&self) -> HttpMethod {
        match &self.event.http_method {
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
        }
    }

    /// Returns a cloned, owned copy of the body in String form.
    pub fn body(&self) -> String {
        match &self.event.body {
            None => "".to_string(),
            Some(body) => body.clone(),
        }
    }

    /// This provides access to authentication claims (in AWS Lambda Proxy calls) that are `String`s.
    /// This signature is likely to change with Azure and Google Cloud Function implemenations.
    pub fn authentication_claim(&self, claim: &str) -> String {
        match &self.event.request_context.authorizer.get("claims") {
            None => "".to_string(),
            Some(claims) => {
                let value = serde_json::to_value(claims).unwrap();
                let claim = match value.get(claim) {
                    None => {
                        "".to_string()
                    }
                    Some(claim) => {
                        match claim {
                            Value::String(val) => val.to_string(),
                            Value::Null |
                            Value::Bool(_) |
                            Value::Number(_) |
                            Value::Array(_) |
                            Value::Object(_) => "".to_string(),
                        }
                    }
                };
                claim
            }
        }
    }
}


#[cfg(test)]
mod validation_tests {
    use aws_lambda_events::event::apigw::{ApiGatewayProxyRequestContext, ApiGatewayRequestIdentity, ApiGatewayProxyResponse};

    use crate::application::{Srvrls, SrvrlsApplication};

    use super::*;
    use lambda_runtime::{Context, Handler};

    struct TestApplication {
        response: SrvrlsResponse
    }

    impl TestApplication {
        fn new(response: SrvrlsResponse) -> Self {
            TestApplication { response }
        }
    }

    impl SrvrlsApplication for TestApplication {
        fn handle(&mut self, _event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
            Ok(self.response.clone())
        }
    }

    struct ErrorApplication {
        error: SrvrlsError
    }

    impl ErrorApplication {
        fn new(error: SrvrlsError) -> Self {
            ErrorApplication { error }
        }
    }

    impl SrvrlsApplication for ErrorApplication {
        fn handle(&mut self, _event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
            Err(self.error.clone())
        }
    }

    #[test]
    fn test_response_header_provider() {
        let application = TestApplication::new(SrvrlsResponse::ok_empty());
        let mut wrapper = Srvrls::new(application);
        wrapper.with_response_header_provider(Box::new(|h| {
            let mut header_provider = HashMap::new();
            for (key, value) in h.iter() {
                header_provider.insert(key.to_string(), value.to_string());
            }
            header_provider.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
            header_provider
        }));
        match wrapper.run(api_proxy_request(), Context::default()) {
            Ok(result) => {
                let mut headers = HashMap::new();
                headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
                assert_eq!(api_proxy_response(200, None, headers), result)
            }
            Err(e) => { panic!(e) }
        }
    }

    #[test]
    fn test_no_body() {
        let application = TestApplication::new(SrvrlsResponse::ok_empty());
        let mut wrapper = Srvrls::new(application);
        match wrapper.run(api_proxy_request(), Context::default()) {
            Ok(result) => {
                assert_eq!(api_proxy_response(200, None, Default::default()), result)
            }
            Err(e) => { panic!(e) }
        }
    }

    #[test]
    fn test_with_body() {
        let application = TestApplication::new(SrvrlsResponse::ok(SrvrlsResponse::simple_error("a message".to_string())));
        let mut srvrls = Srvrls::new(application);
        match srvrls.run(api_proxy_request(), Context::default()) {
            Ok(result) => {
                assert_eq!(api_proxy_response(200, Some(r#"{"error":"a message"}"#.to_string()), Default::default()), result)
            }
            Err(e) => { panic!(e) }
        }
    }

    #[test]
    fn test_error_bad_request() {
        let application = ErrorApplication::new(SrvrlsError::BadRequest("fail".to_string()));
        let mut srvrls = Srvrls::new(application);

        expect_error(&mut srvrls, 400, Some(r#"{"error":"fail"}"#.to_string()))
    }

    #[test]
    fn test_error_unauthorized() {
        let application = ErrorApplication::new(SrvrlsError::Unauthorized);
        let mut srvrls = Srvrls::new(application);

        expect_error(&mut srvrls, 401, None);
    }

    #[test]
    fn test_error_forbidden() {
        let application = ErrorApplication::new(SrvrlsError::Forbidden);
        let mut srvrls = Srvrls::new(application);

        expect_error(&mut srvrls, 403, None);
    }

    #[test]
    fn test_error_not_found() {
        let application = ErrorApplication::new(SrvrlsError::NotFound);
        let mut srvrls = Srvrls::new(application);

        expect_error(&mut srvrls, 404, None);
    }

    fn expect_error(srvrls: &mut Srvrls<ErrorApplication>, expected_status: i64, expected_boy: Option<String>) -> () {
        match srvrls.run(api_proxy_request(), Context::default()) {
            Ok(result) => {
                assert_eq!(result, api_proxy_response(expected_status, expected_boy, Default::default()))
            }
            Err(e) => { panic!(e) }
        }
    }

    fn api_proxy_response(status_code: i64, body: Option<String>, headers: HashMap<String, String>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code,
            headers,
            multi_value_headers: Default::default(),
            body,
            is_base64_encoded: None,
        }
    }

    fn api_proxy_request() -> ApiGatewayProxyRequest {
        let mut path_parameters: HashMap<String, String> = Default::default();
        path_parameters.insert("proxy".to_string(), "path/to/route".to_string());
        ApiGatewayProxyRequest {
            resource: None,
            path: None,
            http_method: None,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            query_string_parameters: Default::default(),
            multi_value_query_string_parameters: Default::default(),
            path_parameters,
            stage_variables: Default::default(),
            request_context: ApiGatewayProxyRequestContext {
                account_id: None,
                resource_id: None,
                stage: None,
                request_id: None,
                identity: ApiGatewayRequestIdentity {
                    cognito_identity_pool_id: None,
                    account_id: None,
                    cognito_identity_id: None,
                    caller: None,
                    api_key: None,
                    access_key: None,
                    source_ip: None,
                    cognito_authentication_type: None,
                    cognito_authentication_provider: None,
                    user_arn: None,
                    user_agent: None,
                    user: None,
                },
                resource_path: None,
                authorizer: Default::default(),
                http_method: None,
                apiid: None,
            },
            body: None,
            is_base64_encoded: None,
        }
    }
}
