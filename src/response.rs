use std::collections::HashMap;

use serde::Serialize;

/// The struct used to house details of the call response for custom responses.
/// For most responses the helper methods are most useful.
#[derive(Debug, Clone, PartialEq)]
pub struct SrvrlsResponse {
    /// Http status code.
    pub status_code: i32,
    /// Any custom response headers, this will be improved by any configured `header_interceptor`.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: Option<String>,
}

/// A simple error message wrapper.
#[derive(Serialize)]
pub struct SimpleError {
    error: String,
}

impl SrvrlsResponse {

    /// Wraps a simple String error message in a [`SimpleError`] to be serialized for the response.
    pub fn simple_error(error_message: String) -> Option<SimpleError> {
        Some(SimpleError {
            error: error_message,
        })
    }

    /// Helper method to provide a response for 200 - Ok with the provided response body
    pub fn ok<T: Serialize>(body: T) -> SrvrlsResponse { SrvrlsResponse::with_status_and_body(200, body) }

    /// Helper method to provide a response for 200 - Ok with no body
    pub fn ok_empty() -> SrvrlsResponse { SrvrlsResponse::with_status(200) }

    /// Helper method to provide a response for 201 - Created
    pub fn created() -> SrvrlsResponse { SrvrlsResponse::with_status(201) }

    /// Helper method to provide a response for 202 - Accepted
    pub fn accepted() -> SrvrlsResponse { SrvrlsResponse::with_status(204) }

    /// Helper method to provide a response for 204 - No Content
    pub fn no_content() -> SrvrlsResponse { SrvrlsResponse::with_status(204) }

    /// Helper method to provide a response for 400 - Bad Request
    pub fn bad_request<T: Serialize>(body: T) -> SrvrlsResponse { SrvrlsResponse::with_status_and_body(400, body) }

    /// Helper method to provide a response for 401 - Unauthorized
    pub fn unauthorized() -> SrvrlsResponse { SrvrlsResponse::with_status(401) }

    /// Helper method to provide a response for 403 - Forbidden
    pub fn forbidden() -> SrvrlsResponse { SrvrlsResponse::with_status(403) }

    /// Helper method to provide a response for 404 - Not Found
    pub fn not_found() -> SrvrlsResponse { SrvrlsResponse::with_status(404) }

    /// Helper method to provide a response for 405 - Method Not Allowed
    pub fn method_not_allowed() -> SrvrlsResponse { SrvrlsResponse::with_status(405) }

    /// Helper method to provide a response for 500 - Internal Server Error
    pub fn internal_server_error() -> SrvrlsResponse { SrvrlsResponse::with_status(500) }

    /// Helper method to provide a response for 503 - Service Unavailable
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