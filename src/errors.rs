use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

// error messages enums
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ErrorMessage {
    EmptyPassword,
    ExceededMaxPasswordLength(usize),
    InvalidEmailFormat,
    InvalidToken,
    HashingError,
    ServerError,
    WrongCredentials,
    InvalidJwt,
    UserNotAuthenticated,
    PaawordNotValidated,
    InvalidHashFormat,
}

// error messages in strings
impl ToString for ErrorMessage {
    fn to_string(&self) -> String {
        match self {
            ErrorMessage::EmptyPassword => "password field is empty".to_string(),
            ErrorMessage::ExceededMaxPasswordLength(max) => {
                format!("password exceeds maximum length of {}", max)
            }
            ErrorMessage::InvalidEmailFormat => "email format is invalid".to_string(),
            ErrorMessage::InvalidToken => "token is invalid or expired".to_string(),
            ErrorMessage::HashingError => "failed to hash the password".to_string(),
            ErrorMessage::ServerError => "internal server error occurred".to_string(),
            ErrorMessage::WrongCredentials => "email or password is incorrect".to_string(),
            ErrorMessage::InvalidJwt => "JWT token is invalid".to_string(),
            ErrorMessage::UserNotAuthenticated => "user is not authenticated".to_string(),
            ErrorMessage::PaawordNotValidated => {
                "password not validated , wrong password".to_string()
            }
            ErrorMessage::InvalidHashFormat => {
                "incoming hashed password is not of correct format".to_string()
            }
        }
    }
}

// http error struct , if which we will show error
#[derive(Debug, Serialize, Clone)]
pub struct HttpError {
    pub message: String,
    #[serde(with = "http_serde::status_code")] //add serialie trait to the statuscode struct
    pub status: StatusCode,
}

// http error function
impl HttpError {
    pub fn new(message: impl Into<String>, status: StatusCode) -> Self {
        HttpError {
            message: message.into(),
            status,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::NOT_FOUND,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::UNAUTHORIZED,
        }
    }

    pub fn server_error(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        HttpError {
            message: message.into(),
            status: StatusCode::BAD_REQUEST,
        }
    }

    pub fn into_http_response(self) -> Response {
        let json_response = Json(ErrorResponse {
            error: "fail".to_string(),
            message: self.message.clone(),
        });

        (self.status, json_response).into_response()
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl std::error::Error for HttpError {}
