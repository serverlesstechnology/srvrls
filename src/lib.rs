use std::collections::HashMap;

use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use lambda_runtime::Context;
use lambda_runtime::error::HandlerError;
use crate::response::Response;


mod response;

pub type ProxyHandler = fn(ApiGatewayProxyRequest) -> ApiGatewayProxyResponse;

pub struct App {
    handlers: HashMap<String,ProxyHandler>,
}

impl App {
    pub fn new()->Self {
        App{
            handlers: HashMap::new(),
        }
    }
    pub fn register(&mut self, route: &str, f: ProxyHandler){
        self.handlers.insert(route.to_string(), f);
    }
}

impl lambda_runtime::Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HandlerError> for App {
    fn run(&mut self, req: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
        let path_url = &req.path_parameters["proxy"];
        let path_iter : Vec<&str> = path_url.split("/").collect();
        let route = path_iter.first().unwrap();
        let result = match self.handlers.get(&route.to_string()) {
            None => Response::not_found(),
            Some(f) => f(req),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lambda_runtime::Handler;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;


    fn load_file(filename: &str) ->Result<String,std::io::Error> {
        let path = Path::new(filename);
        let mut file = File::open(&path)?;

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(why) => Err(why),
        }
    }

    fn test_func(req: ApiGatewayProxyRequest) -> ApiGatewayProxyResponse {
        Response::ok("{}".to_string())
    }
    #[test]
    fn it_works() {
        let mut app = App::new();
        app.register("target", test_func);

        let s = load_file("test_data/happy_path_event.json").unwrap();
        let event : ApiGatewayProxyRequest = serde_json::from_str(s.as_str()).unwrap();
        let response = app.run(event, Context::default()).unwrap();
        assert_eq!(Response::ok("{}".to_string()), response);

        let s = load_file("test_data/not_supported_event.json").unwrap();
        let event : ApiGatewayProxyRequest = serde_json::from_str(s.as_str()).unwrap();
        let response = app.run(event, Context::default()).unwrap();
        assert_eq!(Response::not_found(), response);
    }
}
