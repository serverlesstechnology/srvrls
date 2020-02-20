use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;

pub struct Response {}

impl Response {
    pub fn not_found() -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 404,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: None,
            is_base64_encoded: None,
        }
    }
    pub fn ok(body: String) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: 404,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Some(body),
            is_base64_encoded: None,
        }
    }
}
