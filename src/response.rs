use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use serde::Serialize;

pub struct Response {}

#[derive(Serialize)]
pub struct SimpleError {
    error: String,
}

impl Response {
    pub fn simple_error(error_message: String) -> Option<SimpleError> {
        Some(SimpleError {
            error: error_message
        })
    }

    pub fn ok<T: Serialize>(body: T) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 200,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Some(Response::derive_body(body)),
            is_base64_encoded: None,
        }
    }

    pub fn ok_empty() -> ApiGatewayProxyResponse { Response::with_status(200) }

    pub fn created() -> ApiGatewayProxyResponse { Response::with_status(201) }

    pub fn accepted() -> ApiGatewayProxyResponse { Response::with_status(204) }

    pub fn no_content() -> ApiGatewayProxyResponse { Response::with_status(204) }

    pub fn bad_request<T: Serialize>(body: T) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 400,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Some(Response::derive_body(body)),
            is_base64_encoded: None,
        }
    }

    pub fn bad_request_with_message(message: String) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 400,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Some(message),
            is_base64_encoded: None,
        }
    }
    pub fn unauthorized() -> ApiGatewayProxyResponse { Response::with_status(401) }

    pub fn forbidden() -> ApiGatewayProxyResponse { Response::with_status(403) }

    pub fn not_found() -> ApiGatewayProxyResponse { Response::with_status(404) }

    pub fn method_not_allowed() -> ApiGatewayProxyResponse { Response::with_status(405) }

    pub fn internal_server_error() -> ApiGatewayProxyResponse { Response::with_status(500) }

    pub fn service_unavailable() -> ApiGatewayProxyResponse { Response::with_status(503) }

    fn with_status(code: i64) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: code,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: None,
            is_base64_encoded: None,
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
        let res = Response::ok(Some(TestDto {
            id: "tst-2BAC456".to_string(),
            name: "Jimmy Jones".to_string(),
        }));

        assert_eq!(res.status_code, 200);
        assert_eq!(res.body, Some(r#"{"id":"tst-2BAC456","name":"Jimmy Jones"}"#.to_string()));
    }

    #[test]
    fn test_ok_empty() {
        let res = Response::ok_empty();

        assert_eq!(res.status_code, 200);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_bad_request() {
        let res = Response::bad_request(Response::simple_error("some problem occured".to_string()));

        assert_eq!(res.status_code, 400);
        assert_eq!(res.body, Some(r#"{"error":"some problem occured"}"#.to_string()));
    }

    #[test]
    fn test_not_found() {
        let res = Response::not_found();

        assert_eq!(res.status_code, 404);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_unauthorized() {
        let res = Response::unauthorized();

        assert_eq!(res.status_code, 401);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_forbidden() {
        let res = Response::forbidden();

        assert_eq!(res.status_code, 403);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_internal_server_error() {
        let res = Response::internal_server_error();

        assert_eq!(res.status_code, 500);
        assert_eq!(res.body, None);
    }

    #[test]
    fn test_service_unavailable() {
        let res = Response::service_unavailable();

        assert_eq!(res.status_code, 503);
        assert_eq!(res.body, None);
    }
}