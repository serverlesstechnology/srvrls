use std::collections::HashMap;

use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use serde_json::Value;

use crate::components::HttpMethod;

/// This replaces the inbound `Request` and `Context` entity with simpler, opinionated methods.
/// The data members can be used directly or one of the provided helper functions can simplify
/// access by providing sensible defaults (e.g., empty string) to simplify match statements.
///
/// E.g.,
/// ```rust
///   # use crate::srvrls::request::SrvrlsRequest;
///   # use crate::srvrls::response::SrvrlsResponse;
///   # use crate::srvrls::components::SrvrlsError;
///   # use crate::srvrls::components::HttpMethod;
///
///   fn test_handler(request: SrvrlsRequest) -> Result<SrvrlsResponse,SrvrlsError> {
///       match (&request.method, request.path_parameter(0).as_str()) {
///           (HttpMethod::POST, "customer") => Ok(SrvrlsResponse::no_content()),
///           (HttpMethod::POST, "account") => Ok(SrvrlsResponse::created()),
///           (HttpMethod::GET, "customer") => Ok(SrvrlsResponse::ok_empty()),
///           _ => return Err(SrvrlsError::NotFound)
///       }
///   }
///
///   let mut request : SrvrlsRequest = Default::default();
///   request.method = HttpMethod::POST;
///   request.path = "customer/CUST-A23948".to_string();
///   assert_eq!(SrvrlsResponse::no_content(), test_handler(request).unwrap());
///
///   let mut request : SrvrlsRequest = Default::default();
///   request.method = HttpMethod::POST;
///   request.path = "account/ACCT-G10291".to_string();
///   assert_eq!(SrvrlsResponse::created(), test_handler(request).unwrap());
///
///   let mut request : SrvrlsRequest = Default::default();
///   request.method = HttpMethod::GET;
///   request.path = "customer/CUST-A23948".to_string();
///   assert_eq!(SrvrlsResponse::ok_empty(), test_handler(request).unwrap());
///
///   let mut request : SrvrlsRequest = Default::default();
///   request.method = HttpMethod::GET;
///   request.path = "account/ACCT-G10291".to_string();
///   assert_eq!(SrvrlsError::NotFound, test_handler(request).unwrap_err());
/// ```


pub struct SrvrlsRequest {
    /// All query parameters in a map by key value.
    pub query_parameters: HashMap<String, Vec<String>>,
    /// The path of the request. This is taken from the inbound event field `path_parameter` for
    /// that has the value `proxy`.
    ///
    /// This value is always provided without a leading '/'
    pub path: String,
    /// All String claims within the authorizer field.
    pub string_claims: HashMap<String, String>,
    /// All Numeric (i64) claims within the authorizer field.
    pub integer_claims: HashMap<String, i64>,
    /// The `HttpMethod` of the request.
    pub method: HttpMethod,
    /// The request payload, or empty String if none exists.
    pub body: String,
}

impl Default for SrvrlsRequest {
    fn default() -> Self {
        SrvrlsRequest {
            query_parameters: Default::default(),
            path: "".to_string(),
            string_claims: Default::default(),
            integer_claims: Default::default(),
            method: HttpMethod::GET,
            body: "".to_string()
        }
    }
}

impl SrvrlsRequest {
    /// Provides the path parameter as a String, if the parameter is missing an empty string will be
    /// returned in its' stead.
    /// ```rust
    ///   # use crate::srvrls::request::SrvrlsRequest;
    ///   # use crate::srvrls::response::SrvrlsResponse;
    ///   # use crate::srvrls::components::SrvrlsError;
    ///   # use crate::srvrls::components::HttpMethod;
    ///
    ///     fn test_handler(request: SrvrlsRequest) -> Result<SrvrlsResponse,SrvrlsError> {
    ///         match (&request.method, request.path_parameter(0).as_str()) {
    ///             (HttpMethod::POST, "customer") => Ok(SrvrlsResponse::no_content()),
    ///             (HttpMethod::POST, "account") => Ok(SrvrlsResponse::created()),
    ///             (HttpMethod::GET, "customer") => Ok(SrvrlsResponse::ok_empty()),
    ///             _ => return Err(SrvrlsError::NotFound)
    ///         }
    ///     }
    ///   let mut request : SrvrlsRequest = Default::default();
    ///   request.path = "customer/update/CUST-A23948".to_string();
    ///   assert_eq!("customer", request.path_parameter(0));
    ///   assert_eq!("update", request.path_parameter(1));
    ///   assert_eq!("CUST-A23948", request.path_parameter(2));
    /// ```
    pub fn path_parameter(&self, position: usize) -> String {
        let parameters : Vec<&str> = self.path.split('/').collect();
        match parameters.get(position) {
            None => "".to_string(),
            Some(parameter) => parameter.to_string(),
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
        let path = event.path_parameters["proxy"].clone();
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
            // path_parameters,
            path,
            string_claims,
            integer_claims,
            query_parameters: query_string_parameters,
            method,
            body,
        }
    }
}

#[cfg(test)]
mod request_tests {
    use crate::request::SrvrlsRequest;
    use crate::components::{HttpMethod, SrvrlsError};
    use std::collections::HashMap;
    use crate::response::SrvrlsResponse;

    #[test]
    fn test_path() {
        let mut request : SrvrlsRequest = Default::default();
        request.path = "customer/update/CUST-A23948".to_string();
        assert_eq!("customer", request.path_parameter(0));
        assert_eq!("update", request.path_parameter(1));
        assert_eq!("CUST-A23948", request.path_parameter(2));
        // let response = test_handler(request);
    }
    #[test]
    fn test_complex_switch() {
        let mut request : SrvrlsRequest = Default::default();
        request.path = "customer/update/CUST-A23948".to_string();
        assert_eq!("customer", request.path_parameter(0));
        assert_eq!("update", request.path_parameter(1));
        assert_eq!("CUST-A23948", request.path_parameter(2));
        // let response = test_handler(request);
    }
    fn test_handler(request: SrvrlsRequest) -> Result<SrvrlsResponse,SrvrlsError> {
        let result = match (&request.method, request.path_parameter(0).as_str()) {
            (HttpMethod::POST, "customer") => add_customer(request.body)?,
            (HttpMethod::POST, "account") => update_account(request.body)?,
            (HttpMethod::GET, "customer") => find_customer(request.path_parameter(1))?,
            _ => return Err(SrvrlsError::NotFound)
        };
        Ok(result)
    }
    fn add_customer(r: String) -> Result<SrvrlsResponse,SrvrlsError> { Ok(SrvrlsResponse::ok_empty()) }
    fn update_account(r: String) -> Result<SrvrlsResponse,SrvrlsError> { Ok(SrvrlsResponse::ok_empty()) }
    fn find_customer(r: String) -> Result<SrvrlsResponse,SrvrlsError> { Ok(SrvrlsResponse::ok_empty()) }
}