use std::collections::HashMap;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::{Context, Handler};
use lambda_runtime::error::HandlerError;

use crate::{SrvrlsError, SrvrlsRequest, SrvrlsResponse};

pub trait SrvrlsApplication {
    fn handle(&mut self, event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError>;
}

pub struct Srvrls<T: SrvrlsApplication> {
    application: T,
    pub(crate) response_header_provider: Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>,
}


impl<T: SrvrlsApplication> Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HandlerError> for Srvrls<T> {
    fn run(&mut self, event: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
        let request = SrvrlsRequest::new(event);
        match self.application.handle(request) {
            Ok(response) => {
                let headers = (self.response_header_provider)(response.headers);
                Ok(self.response(response.status_code as i64, response.body, headers))
            }
            Err(e) => {
                let headers = (self.response_header_provider)(HashMap::new());
                match e {
                    SrvrlsError::BadRequest(body) => Ok(self.response(400, Some(body), headers)),
                    SrvrlsError::BadRequestNoMessage() => Ok(self.response(400, None, headers)),
                    SrvrlsError::BadRequestWithSimpleMessage(simple_message) => {
                        let payload = serde_json::to_string(&SrvrlsResponse::simple_error(simple_message))?;
                        Ok(self.response(400, Some(payload), headers))
                    }
                    SrvrlsError::Unauthorized => Ok(self.response(401, None, headers)),
                    SrvrlsError::Forbidden => Ok(self.response(403, None, headers)),
                    SrvrlsError::NotFound => Ok(self.response(404, None, headers)),
                    SrvrlsError::MethodNotAllowed => Ok(self.response(405, None, headers)),
                    SrvrlsError::InternalServerError => Ok(self.response(500, None, headers)),
                }
            }
        }
    }
}

impl<T: SrvrlsApplication> Srvrls<T> {
    pub fn new(application: T) -> Self {
        let response_header_provider = Box::new(|_h: HashMap<String, String>| HashMap::new());
        Srvrls { application, response_header_provider }
    }
    pub fn with_response_header_provider(&mut self, header_provider: Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>) {
        self.response_header_provider = header_provider;
    }


    fn response(&self, status_code: i64, body: Option<String>, headers: HashMap<String, String>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code: status_code,
            headers,
            multi_value_headers: Default::default(),
            body,
            is_base64_encoded: None,
        }
    }
}
