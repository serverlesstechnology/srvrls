use std::collections::HashMap;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::{Context, Handler};
use lambda_runtime::error::HandlerError;
use crate::request::SrvrlsRequest;
use crate::components::SrvrlsError;
use crate::response::SrvrlsResponse;

/// This trait should be implemented by your application to handle inbound events. The values for
/// these responses (e.g., status code, body, headers) will be mapped to the API Gateway response.
/// ```rust
/// # use srvrls::request::SrvrlsRequest;
/// # use srvrls::response::SrvrlsResponse;
/// # use srvrls::components::SrvrlsError;
/// # use srvrls::application::SrvrlsApplication;
/// # struct MyApplication {}
/// impl SrvrlsApplication for MyApplication {
///     fn handle(&mut self,event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
///         Ok(SrvrlsResponse::ok_empty())
///     }
/// }
/// ```
/// Errors can also be returned with will map directly to their respective response values.
/// ```rust
/// # use srvrls::request::SrvrlsRequest;
/// # use srvrls::response::SrvrlsResponse;
/// # use srvrls::components::SrvrlsError;
/// # use srvrls::application::SrvrlsApplication;
/// # struct MyApplication {}
/// impl SrvrlsApplication for MyApplication {
///     fn handle(&mut self,event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
///         Err(SrvrlsError::Unauthorized)
///     }
/// }
/// ```
pub trait SrvrlsApplication {
    /// This method receives the inbound request and should return a result composed of either
    /// a `SrvrlsResponse` or a `SrvrlsError` that will be mapped to a (4xx or 5xx) response.
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
    /// impl SrvrlsApplication for App {
    ///     fn handle(&mut self,event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
    ///         Ok(SrvrlsResponse::ok_empty())
    ///     }
    /// }
    ///
    /// fn build_srvrls() -> Srvrls<App> {
    ///     let app = App{};
    ///     Srvrls::new(app)
    /// }
    /// ```
    /// This `Srvrls` object is then used to build your lambda application within the `main`.
    /// ```ignore
    /// use lambda_runtime::lambda;
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let srvrls : Srvrls = build_srvrls();
    ///     lambda!(srvrls);
    ///     Ok(())
    /// }
    /// ```
    /// Note that you will need the
    /// [lambda runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
    /// included in your project.
    /// ```toml
    /// [dependencies]
    /// lambda_runtime = "0.2.1"
    /// ```
    /// And some additional steps are needed for packaging see the
    /// [lamba runtime deployment notes](https://github.com/awslabs/aws-lambda-rust-runtime#deployment).
    pub fn new(application: T) -> Self {
        let response_header_interceptor = Box::new(|_h: HashMap<String, String>| HashMap::new());
        Srvrls { application, response_header_interceptor }
    }

    /// This function allows for adding a closure that will function as a header interceptor.
    /// All responses will then have their headers enhanced by this interceptor.
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
