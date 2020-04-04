use std::collections::HashMap;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::{Context, Handler};
use lambda_runtime::error::HandlerError;
use crate::request::SrvrlsRequest;
use crate::components::SrvrlsError;
use crate::response::SrvrlsResponse;

/// This trait should be implemented by your application to handle inbound events.
pub trait SrvrlsApplication {
    /// This method receives the inbound request and should return a result
    fn handle(&mut self, event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError>;
}

type HeaderInterceptor = Box<dyn Fn(HashMap<String, String>) -> HashMap<String, String>>;

/// Srvrls wraps your application that implements `SrvrlsApplication` and interfaces with the
/// AWS Lambda to handle the logic of translating requests and responses.
pub struct Srvrls<T: SrvrlsApplication> {
    application: T,
    pub(crate) response_header_interceptor: HeaderInterceptor,
}

impl<T: SrvrlsApplication> Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HandlerError> for Srvrls<T> {
    fn run(&mut self, event: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
        let request: SrvrlsRequest = event.into();
        match self.application.handle(request) {
            Ok(response) => {
                let headers = (self.response_header_interceptor)(response.headers);
                Ok(self.response(i64::from(response.status_code), response.body, headers))
            }
            Err(e) => {
                let headers = (self.response_header_interceptor)(HashMap::new());
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
    /// Create a new `Srvrls` instance to interface with AWS Lambda
    /// ```rust
    /// # use std::error::Error;
    /// # use srvrls::application::Srvrls;
    /// # use srvrls::application::SrvrlsApplication;
    /// # use srvrls::components::SrvrlsError;
    /// # use srvrls::request::SrvrlsRequest;
    /// # use srvrls::response::SrvrlsResponse;
    /// struct App {}
    ///
    /// impl SrvrlsApplication for App {fn handle(&mut self,event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
    ///         Ok(SrvrlsResponse::ok_empty())
    ///     }
    /// }
    ///
    /// fn build_srvrls() -> Srvrls<App> {
    ///     let app = App{};
    ///     Srvrls::new(app)
    /// }
    /// ```
    /// This is then used to build your lambda application with
    /// ```ignore
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let srvrls = build_srvrls();
    ///     lambda!(srvrls);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(application: T) -> Self {
        let response_header_provider = Box::new(|_h: HashMap<String, String>| HashMap::new());
        Srvrls { application, response_header_interceptor: response_header_provider }
    }

    /// Allows for adding a header interceptor that modifies the response headers for all calls.
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use std::error::Error;
    /// # use srvrls::application::Srvrls;
    /// # use srvrls::application::SrvrlsApplication;
    /// # use srvrls::components::SrvrlsError;
    /// # use srvrls::request::SrvrlsRequest;
    /// # use srvrls::response::SrvrlsResponse;
    /// # struct App {}
    /// # impl SrvrlsApplication for App {fn handle(&mut self,event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
    /// #         Ok(SrvrlsResponse::ok_empty())
    /// #     }
    /// # }
    /// fn build_srvrls() -> Srvrls<App> {
    ///     let app = App{};
    ///     let mut srvrls = Srvrls::new(app);
    ///     let header_interceptor = |mut h: HashMap<String,String>| {
    ///         h.insert("Content-Type".to_string(), "application/json".to_string());
    ///         h
    ///     };
    ///     srvrls.with_response_header_interceptor(Box::new(header_interceptor));
    ///     srvrls
    /// }
    ///```
    pub fn with_response_header_interceptor(&mut self, header_interceptor: HeaderInterceptor) {
        self.response_header_interceptor = header_interceptor;
    }

    fn response(&self, status_code: i64, body: Option<String>, headers: HashMap<String, String>) -> ApiGatewayProxyResponse {
        ApiGatewayProxyResponse {
            status_code,
            headers,
            multi_value_headers: Default::default(),
            body,
            is_base64_encoded: None,
        }
    }
}

struct App {}
impl SrvrlsApplication for App {
    fn handle(&mut self, event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
        Ok(SrvrlsResponse::ok_empty())
    }
}
fn application_build_test_code() -> App {
    App {}
}
