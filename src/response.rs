use std::collections::HashMap;

use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct SrvrlsResponse {
    pub status_code: i32,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Serialize)]
pub struct SimpleError {
    error: String,
}

impl SrvrlsResponse {
    pub fn simple_error(error_message: String) -> Option<SimpleError> {
        Some(SimpleError {
            error: error_message,
        })
    }

    pub fn ok<T: Serialize>(body: T) -> SrvrlsResponse { SrvrlsResponse::with_status_and_body(200, body) }

    pub fn ok_empty() -> SrvrlsResponse { SrvrlsResponse::with_status(200) }

    pub fn created() -> SrvrlsResponse { SrvrlsResponse::with_status(201) }

    pub fn accepted() -> SrvrlsResponse { SrvrlsResponse::with_status(204) }

    pub fn no_content() -> SrvrlsResponse { SrvrlsResponse::with_status(204) }

    pub fn bad_request<T: Serialize>(body: T) -> SrvrlsResponse { SrvrlsResponse::with_status_and_body(400, body) }

    pub fn unauthorized() -> SrvrlsResponse { SrvrlsResponse::with_status(401) }

    pub fn forbidden() -> SrvrlsResponse { SrvrlsResponse::with_status(403) }

    pub fn not_found() -> SrvrlsResponse { SrvrlsResponse::with_status(404) }

    pub fn method_not_allowed() -> SrvrlsResponse { SrvrlsResponse::with_status(405) }

    pub fn internal_server_error() -> SrvrlsResponse { SrvrlsResponse::with_status(500) }

    pub fn service_unavailable() -> SrvrlsResponse { SrvrlsResponse::with_status(503) }

    fn with_status(status_code: i32) -> SrvrlsResponse {
        SrvrlsResponse {
            status_code,
            headers: Default::default(),
            body: None,
        }
    }
    fn with_status_and_body<T: Serialize>(status_code: i32, body: T) -> SrvrlsResponse {
        SrvrlsResponse {
            status_code,
            headers: Default::default(),
            body: Some(SrvrlsResponse::derive_body(body)),
        }
    }

    fn derive_body<T: Serialize>(body: T) -> String {
        serde_json::to_string(&body).unwrap()
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;

    #[derive(Default, Debug, Serialize)]
    struct TestDto {
        id: String,
        name: String,
    }

    #[test]
    fn test_ok() {
        let res = SrvrlsResponse::ok(Some(TestDto {
            id: "tst-2BAC456".to_string(),
            name: "Jimmy Jones".to_string(),
        }));

        assert_eq!(res.status_code, 200);
        assert_eq!(res.body, Some(r#"{"id":"tst-2BAC456","name":"Jimmy Jones"}"#.to_string()));
    }

    #[test]
    fn test_ok_empty() {
        let res = SrvrlsResponse::ok_empty();

        assert_eq!(res.status_code, 200);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_bad_request() {
        let res = SrvrlsResponse::bad_request(SrvrlsResponse::simple_error("some problem occured".to_string()));

        assert_eq!(res.status_code, 400);
        assert_eq!(res.body, Some(r#"{"error":"some problem occured"}"#.to_string()));
    }

    #[test]
    fn test_not_found() {
        let res = SrvrlsResponse::not_found();

        assert_eq!(res.status_code, 404);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_unauthorized() {
        let res = SrvrlsResponse::unauthorized();

        assert_eq!(res.status_code, 401);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_forbidden() {
        let res = SrvrlsResponse::forbidden();

        assert_eq!(res.status_code, 403);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_internal_server_error() {
        let res = SrvrlsResponse::internal_server_error();

        assert_eq!(res.status_code, 500);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_service_unavailable() {
        let res = SrvrlsResponse::service_unavailable();

        assert_eq!(res.status_code, 503);
        assert_eq!(res.body, None);
    }
}