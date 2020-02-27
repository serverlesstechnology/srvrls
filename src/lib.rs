#[macro_use]
extern crate validator_derive;
extern crate validator;

use std::collections::HashMap;
use std::fmt::Debug;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::Context;
use lambda_runtime::error::HandlerError;

use crate::response::Response;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

mod response;
mod persist;
mod validate;

trait Processor {
    fn process(&self, req: ApiGatewayProxyRequest) -> ApiGatewayProxyResponse;
}
pub type ProxyHandler = fn(ApiGatewayProxyRequest) -> ApiGatewayProxyResponse;
pub type RpcHandler<T> = fn(T) -> ApiGatewayProxyResponse;

pub struct LambdaRouter {
    handlers: HashMap<String, Box<dyn Processor>>,
}

impl LambdaRouter {
    pub fn new() -> Self {
        LambdaRouter {
            handlers: HashMap::new(),
        }
    }
    pub fn register(&mut self, route: &str, f: ProxyHandler) {
        self.handlers.insert(route.to_string(), Box::new(GenericProcessor::new(f)));
    }
    pub fn register_rpc<T: 'static + Default + Debug + DeserializeOwned>(&mut self, route: &str, rpc_f: RpcHandler<T>) {
        self.handlers.insert(route.to_string(), Box::new(RpcProcessor::new(rpc_f)));
    }
}

struct GenericProcessor {
    proxy_handler: ProxyHandler,
}
impl GenericProcessor {
    fn new(proxy_handler: ProxyHandler)->Self {
        GenericProcessor{
            proxy_handler
        }
    }
}
impl Processor for GenericProcessor {
    fn process(&self, req: ApiGatewayProxyRequest) -> ApiGatewayProxyResponse {
        (self.proxy_handler)(req)
    }
}



struct RpcProcessor<T: Default + Debug> {
    h: RpcHandler<T>,
}
impl <T: Default + Debug> RpcProcessor<T> {
    fn new(h: RpcHandler<T>) -> Self {
        RpcProcessor {
            h: h,
        }
    }
    fn clean_val(&self)->T{
        let val = T::default();
        val
    }
}
impl <T: Default + Debug + DeserializeOwned> Processor for RpcProcessor<T> {
    fn process(&self, req: ApiGatewayProxyRequest) -> ApiGatewayProxyResponse {
        let body = match req.body {
            None => return Response::not_found(),
            Some(body) => body,
        };
        let dto: T = match serde_json::from_str(&body.as_str()) {
            Ok(dto) => {
                dto
            },
            Err(err) => return Response::internal_server_error(Response::simple_error(err.to_string())),
        };
        (self.h)(dto)
    }
}

impl lambda_runtime::Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HandlerError> for LambdaRouter {
    fn run(&mut self, req: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
        let path_url = &req.path_parameters["proxy"];
        let path_iter: Vec<&str> = path_url.split("/").collect();
        let route = path_iter.first().unwrap();
        let result = match self.handlers.get(&route.to_string()) {
            None => Response::not_found(),
            Some(f) => f.process(req),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod lambda_router_tests {
    use super::*;

    use std::fs::File;
    use std::io::Read;

    use std::path::Path;

    use lambda_runtime::Handler;

    fn load_file(filename: &str) -> Result<String, std::io::Error> {
        let path = Path::new(filename);
        let mut file = File::open(&path)?;

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(why) => Err(why),
        }
    }

    fn test_func(req: ApiGatewayProxyRequest) -> ApiGatewayProxyResponse {
        Response::ok(Some(req.headers))
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    struct TestDto {
        id: String,
        name: String,
    }

    fn test_rpc_func(dto: TestDto) -> ApiGatewayProxyResponse {
        Response::ok(Some(dto))
    }

    #[test]
    fn test_register() {
        let mut app = LambdaRouter::new();
        app.register("target", test_func);

        let s = load_file("test_data/happy_path_event.json").unwrap();
        let event: ApiGatewayProxyRequest = serde_json::from_str(s.as_str()).unwrap();
        let response = app.run(event, Context::default()).unwrap();
        assert_eq!(response.body.unwrap().len(), 1313);
        assert_eq!(response.status_code, 200);

        let s = load_file("test_data/not_supported_event.json").unwrap();
        let event: ApiGatewayProxyRequest = serde_json::from_str(s.as_str()).unwrap();
        let response = app.run(event, Context::default()).unwrap();
        assert_eq!(Response::not_found(), response);
    }

    #[test]
    fn test_rpc_register() {
        let mut app = LambdaRouter::new();
        app.register_rpc("target", test_rpc_func);

        let s = load_file("test_data/happy_path_event.json").unwrap();
        let event: ApiGatewayProxyRequest = serde_json::from_str(s.as_str()).unwrap();
        let response = app.run(event, Context::default()).unwrap();
        assert_eq!(response.body.unwrap(), r#"{"id":"tst-E3A216","name":"Steve Smith"}"#.to_string());
        assert_eq!(response.status_code, 200);
    }
}
