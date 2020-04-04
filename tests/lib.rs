#[cfg(test)]
mod application_tests {
    use std::collections::HashMap;

    use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayProxyResponse, ApiGatewayRequestIdentity};
    use lambda_runtime::{Context, Handler};

    use srvrls::application::{SrvrlsApplication, Srvrls};
    use srvrls::components::SrvrlsError;
    use srvrls::request::SrvrlsRequest;
    use srvrls::response::SrvrlsResponse;

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
        wrapper.with_response_header_interceptor(Box::new(|h| {
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
        let application = ErrorApplication::new(SrvrlsError::BadRequest(r#"{"a_key":"a-value"}"#.to_string()));
        let mut srvrls = Srvrls::new(application);

        expect_error(&mut srvrls, 400, Some(r#"{"a_key":"a-value"}"#.to_string()))
    }

    #[test]
    fn test_error_bad_request_with_simple_message() {
        let application = ErrorApplication::new(SrvrlsError::BadRequestWithSimpleMessage("fail".to_string()));
        let mut srvrls = Srvrls::new(application);

        expect_error(&mut srvrls, 400, Some(r#"{"error":"fail"}"#.to_string()))
    }

    #[test]
    fn test_error_bad_request_no_message() {
        let application = ErrorApplication::new(SrvrlsError::BadRequestNoMessage());
        let mut srvrls = Srvrls::new(application);

        expect_error(&mut srvrls, 400, None)
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
