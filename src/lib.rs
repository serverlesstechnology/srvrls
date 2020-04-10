// #![deny(missing_docs)]
//! # srvrls - a lightweight serverless framework
//!
//! ![CodeBuild test indicator](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoib3o3dlJ5RkJuMEVTckIyR1p1WXAzZkxjVzFQTnZ1QjFMUzZ0OUc2Q1dkQlVhQVU2WjFFTExyQVladmRoc2tSRkozbHFVaHg2ZGhtY2xlN2N1ZFY4cDhjPSIsIml2UGFyYW1ldGVyU3BlYyI6IjdiZUk4RWRZeHpoemZxdEUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=master)
//!
//! We built this library to simplify building applications that use AWS API Gateway
//! as a proxy for AWS Lambda.
//!
//! This library has opinions, strong ones, very possibly not your own.
//! Our design priorities here are simple:
//!
//! - reduce needed boilerplate in serverless applications
//! - provide opinionated defaults to otherwise open questions (more on this later)
//! - provide decoupling between the serverless function provider and the application logic
//! (keeping open the option of supporting Google or Azure functions in the future)
//!

/// Application provides the AWS Lambda wrapper and response handling.
pub mod application;

/// Components holds the utility structs including the library error `SrvrlsError`.
pub mod components;

/// Response provides the mappings from the library response and error to AWS Lambda events.
pub mod response;

/// Request provides a simplified input request struct with opinionated getter methods.
pub mod request;
