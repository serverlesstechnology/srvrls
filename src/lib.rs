#[macro_use]
extern crate validator_derive;
extern crate validator;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest};

pub use crate::response::Response;

mod response;
mod validate;


pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    OTHER,
}

pub struct Srvrls {}

impl Srvrls {
    pub fn method(event: &ApiGatewayProxyRequest) -> HttpMethod {
        match &event.http_method {
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
        }
    }
}
