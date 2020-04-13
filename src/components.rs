use std::{fmt, error};
use crate::response::SrvrlsResponse;

/// Replaces a String match with an enum that only includes the most common `HttpMethod`s. This
/// reduces overhead from trash methods like `CONNECT`, `OPTIONS` or `TRACE` as well as non-legit
/// codes that are possible with `Strings`.
/// Seriously, if you're using one of those you're probably just trolling your users.
pub enum HttpMethod {
    /// GET http method
    GET,
    /// POST http method
    POST,
    /// PUT http method
    PUT,
    /// HEAD http method
    HEAD,
    /// DELETE http method
    DELETE,
    /// One of the other http methods not implemented here
    /// (intentionally, there is no reason to have 20 line match statements for the 1% of people
    /// using `TRACE`)
    OTHER,
}

/// You can always return the precise error response, but using the specific error allows a much
/// cleaner rolloff using the `?` operator. All of these translate directly to their respective
/// 4xx or 5xx HTTP responses.
#[derive(Debug, Clone, PartialEq)]
pub enum SrvrlsError {
    /// Responds with a 400 - Bad Request response using the provided payload
    BadRequest(String),
    /// Responds with a 400 - Bad Request response with no payload
    BadRequestNoMessage(),
    /// Responds with a 400 - Bad Request response using the provided message in a `SimpleMessage`
    BadRequestWithSimpleMessage(String),
    /// Responds with a 401 - Unauthorized response
    Unauthorized,
    /// Responds with a 403 - Forbidden response
    Forbidden,
    /// Responds with a 404 - Not Found response
    NotFound,
    /// Responds with a 405 - Method Not Allowed response
    MethodNotAllowed,
    /// Responds with a 500 - Internal Server Error response
    InternalServerError,
}

impl error::Error for SrvrlsError {}

impl fmt::Display for SrvrlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SrvrlsError::BadRequestNoMessage() => write!(f, "Bad Request"),
            SrvrlsError::BadRequestWithSimpleMessage(msg) => {
                let response = SrvrlsResponse::simple_error((*msg).clone());
                let body = serde_json::to_string(&response).unwrap();
                write!(f, "Bad Request: {}", body)
            },
            SrvrlsError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            SrvrlsError::Unauthorized => write!(f, "Unauthorized"),
            SrvrlsError::Forbidden => write!(f, "Forbidden"),
            SrvrlsError::NotFound => write!(f, "Not Found"),
            SrvrlsError::MethodNotAllowed => write!(f, "Method Not Allowed"),
            SrvrlsError::InternalServerError => write!(f, "InternalServerError"),
        }
    }
}
