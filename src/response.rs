use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use serde::Serialize;

pub struct Response {}

#[derive(Serialize)]
pub struct SimpleError {
    error: String,
}

impl Response {

    pub fn simple_error(error_message: String) -> Option<SimpleError> {
        Some(SimpleError{
            error: error_message
        })
    }

    pub fn ok<T: Serialize>(body: Option<T>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 200,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Response::derive_body(body),
            is_base64_encoded: None,
        }
    }

    pub fn ok_empty() -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 200,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Default::default(),
            is_base64_encoded: None,
        }
    }

    pub fn bad_request<T: Serialize>(body: Option<T>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 400,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Response::derive_body(body),
            is_base64_encoded: None,
        }
    }
    pub fn unauthorized() -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 401,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: None,
            is_base64_encoded: None,
        }
    }
    pub fn forbidden() -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 403,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: None,
            is_base64_encoded: None,
        }
    }
    pub fn not_found() -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 404,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: None,
            is_base64_encoded: None,
        }
    }
    pub fn method_not_allowed() -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 405,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: None,
            is_base64_encoded: None,
        }
    }

    pub fn internal_server_error<T: Serialize>(body: Option<T>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 500,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Response::derive_body(body),
            is_base64_encoded: None,
        }
    }
    pub fn service_unavailable<T: Serialize>(body: Option<T>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 503,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Response::derive_body(body),
            is_base64_encoded: None,
        }
    }


    fn derive_body<T: Serialize>(body: Option<T>) -> Option<String> {
        let payload = match body {
            None => None,
            Some(dto) => {
                let payload = serde_json::to_string(&dto).unwrap();
                Some(payload)
            }
        };
        payload
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
        let res = Response::ok(Some(TestDto{
            id: "tst-2BAC456".to_string(),
            name: "Jimmy Jones".to_string()
        }));

        assert_eq!(res.status_code, 200);
        assert_eq!(res.body, Some(r#"{"id":"tst-2BAC456","name":"Jimmy Jones"}"#.to_string()));
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
        let res = Response::internal_server_error(Response::simple_error("something broke".to_string()));

        assert_eq!(res.status_code, 500);
        assert_eq!(res.body, Some(r#"{"error":"something broke"}"#.to_string()));
    }
    #[test]
    fn test_service_unavailable() {
        let res = Response::service_unavailable(Response::simple_error("server is down".to_string()));

        assert_eq!(res.status_code, 503);
        assert_eq!(res.body, Some(r#"{"error":"server is down"}"#.to_string()));
    }




}