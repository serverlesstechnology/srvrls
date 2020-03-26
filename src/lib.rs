extern crate validator;
#[macro_use]
extern crate validator_derive;

use std::collections::HashMap;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::{Context, Handler};
use lambda_runtime::error::HandlerError;
use serde_json::Value;

pub use crate::response::SrvrlsResponse;

mod response;
mod validate;


pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    OTHER,
}

#[derive(Debug,Clone,PartialEq)]
pub enum SrvrlsError {
    BadRequest(String),
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
}

pub struct SrvrlsRequest {
    event: ApiGatewayProxyRequest,
    path_parameters: HashMap<i32, String>,
}

impl SrvrlsRequest {
    fn new(event: ApiGatewayProxyRequest) -> Self {
        let mut path_parameters = HashMap::new();
        let path_url = &event.path_parameters["proxy"];
        let path_iter: Vec<&str> = path_url.split("/").collect();
        for (i, path_segment) in path_iter.iter().enumerate() {
            path_parameters.insert(i as i32, path_segment.to_string());
        }
        let ll = 0 as i32;
        SrvrlsRequest {
            event,
            path_parameters,
        }
    }

    pub fn path_parameter(&self, position: i32) -> String {
        match &self.path_parameters.get(&position) {
            None => "".to_string(),
            Some(val) => val.to_string(),
        }
    }
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
    pub fn body(&self) -> String {
        match &self.event.body {
            None => "".to_string(),
            Some(body) => body.clone(),
        }
    }

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

pub trait SrvrlsApplication {
    fn handle(&mut self, event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError>;
}

pub struct Srvrls<T: SrvrlsApplication> {
    application: T,
}


impl<T: SrvrlsApplication> Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HandlerError> for Srvrls<T> {
    fn run(&mut self, event: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
        let request = SrvrlsRequest::new(event);
        match self.application.handle(request) {
            Ok(response) => {
                Ok(self.response(response.status_code as i64, response.body, response.headers))
            }
            Err(e) => {
                match e {
                    SrvrlsError::BadRequest(body) => {
                        let payload = serde_json::to_string(&SrvrlsResponse::simple_error(body))?;
                        Ok(self.response(400, Some(payload),Default::default()))
                    }
                    SrvrlsError::Unauthorized => {Ok(self.response(401, None, Default::default()))}
                    SrvrlsError::Forbidden => Ok(self.response(403, None, Default::default())),
                    SrvrlsError::NotFound => Ok(self.response(404, None, Default::default())),
                    SrvrlsError::MethodNotAllowed => Ok(self.response(405, None, Default::default())),
                    SrvrlsError::InternalServerError => Ok(self.response(500, None, Default::default())),
                }
            }
        }
    }
}

impl<T: SrvrlsApplication> Srvrls<T> {
    pub fn new(application: T) -> Self {
        Srvrls { application }
    }

    fn response(&self, status_code: i64, body: Option<String>, headers: HashMap<String,String>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: status_code,
            headers,
            multi_value_headers: Default::default(),
            body,
            is_base64_encoded: None,
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use aws_lambda_events::event::apigw::{ApiGatewayProxyRequestContext, ApiGatewayRequestIdentity};

    use super::*;

    struct TestApplication {
        response: SrvrlsResponse
    }

    impl TestApplication {
        fn new(response: SrvrlsResponse) -> Self {
            TestApplication { response }
        }
    }

    impl SrvrlsApplication for TestApplication {
        fn handle(&mut self, event: SrvrlsRequest) -> Result<SrvrlsResponse,SrvrlsError> {
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
        fn handle(&mut self, event: SrvrlsRequest) -> Result<SrvrlsResponse,SrvrlsError> {
            Err(self.error.clone())
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
    fn test_error() {
        let application = ErrorApplication::new(SrvrlsError::BadRequest("fail".to_string()));
        let mut srvrls = Srvrls::new(application);
        match srvrls.run(api_proxy_request(), Context::default()) {
            Ok(result) => {
                assert_eq!(result, api_proxy_response(400, Some(r#"{"error":"fail"}"#.to_string()), Default::default()))
            }
            Err(e) => { panic!(e) }
        }
    }

    fn api_proxy_response(status_code: i64, body: Option<String>, headers: HashMap<String,String>) -> ApiGatewayProxyResponse {
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
