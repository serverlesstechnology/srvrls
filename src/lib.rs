extern crate validator;
#[macro_use]
extern crate validator_derive;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::{Context, Handler};
use lambda_runtime::error::HandlerError;
use serde_json::Value;

pub use crate::response::SrvrlsResponse;
use std::collections::HashMap;

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

pub struct SrvrlsRequest {
    event: ApiGatewayProxyRequest,
    path_parameters: HashMap<i32,String>
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
            path_parameters
        }
    }

    pub fn path_parameter(&self, position: i32) -> String {
        let pos = position.clone();
        match &self.event.path_parameters.get(&pos) {
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
    fn handle(&mut self, event: SrvrlsRequest) -> SrvrlsResponse;
}

pub struct Srvrls<T: SrvrlsApplication> {
    application: T,
}


impl<T: SrvrlsApplication> Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HandlerError> for Srvrls<T> {
    fn run(&mut self, event: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
        let request = SrvrlsRequest::new(event);
        let response = self.application.handle(request);
        Ok(ApiGatewayProxyResponse {
            status_code: response.status_code as i64,
            headers: response.headers,
            multi_value_headers: Default::default(),
            body: response.body,
            is_base64_encoded: None,
        })
    }
}

impl<T: SrvrlsApplication> Srvrls<T> {
    pub fn new(application: T) -> Self {
        Srvrls { application }
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
        fn handle(&mut self, event: SrvrlsRequest) -> SrvrlsResponse {
            self.response.clone()
        }
    }

    #[test]
    fn test_no_body() {
        let application = TestApplication::new(SrvrlsResponse::ok_empty());
        let mut wrapper = Srvrls::new(application);
        match wrapper.run(api_proxy_request(), Context::default()) {
            Ok(result) => {
                assert_eq!(api_proxy_response(200, None), result)
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
                assert_eq!(api_proxy_response(200, Some(r#"{"error":"a message"}"#.to_string())), result)
            }
            Err(e) => { panic!(e) }
        }
    }

    fn api_proxy_response(status_code: i64, body: Option<String>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body,
            is_base64_encoded: None,
        }
    }

    fn api_proxy_request() -> ApiGatewayProxyRequest {
        let mut path_parameters : HashMap<String,String> = Default::default();
        path_parameters.insert("proxy".to_string(),"path/to/route".to_string());
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
