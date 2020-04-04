use std::{fmt, error};

/// Replaces a String match with an enum that only includes the most common `HttpMethod`s. This
/// reduces overhead from trash methods like `CONNECT`, `OPTIONS` or `TRACE` as well as non-legit
/// codes that are possible with `Strings`.
/// Seriously, if you're using one of those you're probably just trolling your users.
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    OTHER,
}

/// You can always return the precise error response, but using the specific error allows a much
/// cleaner rolloff using the `?` operator. All of these translate directly to their respective
/// 4xx or 5xx HTTP responses.
#[derive(Debug, Clone, PartialEq)]
pub enum SrvrlsError {
    BadRequest(String),
    BadRequestNoMessage(),
    BadRequestWithSimpleMessage(String),
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
}

impl error::Error for SrvrlsError {}

impl fmt::Display for SrvrlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SrvrlsError::BadRequestWithSimpleMessage(msg) => write!(f, "Bad Request: {}", msg),
            SrvrlsError::BadRequestNoMessage() => write!(f, "Bad Request"),
            SrvrlsError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            SrvrlsError::Unauthorized => write!(f, "Unauthorized"),
            SrvrlsError::Forbidden => write!(f, "Forbidden"),
            SrvrlsError::NotFound => write!(f, "Not Found"),
            SrvrlsError::MethodNotAllowed => write!(f, "Method Not Allowed"),
            SrvrlsError::InternalServerError => write!(f, "InternalServerError"),
        }
    }
}
