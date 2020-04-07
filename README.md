# srvrls

**A lightweight wrapper for using AWS Lambda as an API Gateway proxy.**

![CodeBuild test indicator](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoib3o3dlJ5RkJuMEVTckIyR1p1WXAzZkxjVzFQTnZ1QjFMUzZ0OUc2Q1dkQlVhQVU2WjFFTExyQVladmRoc2tSRkozbHFVaHg2ZGhtY2xlN2N1ZFY4cDhjPSIsIml2UGFyYW1ldGVyU3BlYyI6IjdiZUk4RWRZeHpoemZxdEUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=master)
[![Crates.io](https://img.shields.io/crates/v/srvrls)](https://crates.io/crates/srvrls)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/srvrls)

--- 

## Installation

Srvrls is available from Crates.io or Github.

```toml
[dependencies]
srvrls = "0.1.5"
```

Or for a specific branch
```toml
[dependencies]
srvrls = { git = "https://github.com/serverlesstechnology/srvrls.git", branch = "master"}
```

## srvrls

We built this library to simplify building applications that use AWS API Gateway
as a proxy for AWS Lambda.

This library has opinions, strong ones, very possibly not your own.
Our design priorities here are simple:

- reduce needed boilerplate in serverless applications
- provide opinionated defaults to otherwise open questions (more on this later)
- provide decoupling between the serverless function provider and the application logic
(keeping open the option of supporting Google or Azure functions in the future)

This wrapper turns this
    
    impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HandlerError> for App {
        fn run(&mut self, event: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, HandlerError> {
            match some_function(event) {
                Ok(response) => {
                    ApiGatewayProxyResponse {
                        status_code: 200,
                        headers: hashmap!(String::from("Content-Type") => String::from("application/json")),
                        multi_value_headers: HashMap::new(),
                        body: Some(response),
                        is_base64_encoded: None,
                    }
                }, 
                Err(e) => {
                    ApiGatewayProxyResponse {
                        status_code: 400,
                        headers: hashmap!(String::from("Content-Type") => String::from("application/json")),
                        multi_value_headers: HashMap::new(),
                        body: Some(e.message),
                        is_base64_encoded: None,
                    }
                }
            }
        }
    }

into this

    impl SrvrlsApplication for App {
        fn handle(&mut self, event: SrvrlsRequest) -> Result<SrvrlsResponse, SrvrlsError> {
            let response = some_function(event)?;
            Ok(SrvrlsResponse::ok(response))
        }
    }
