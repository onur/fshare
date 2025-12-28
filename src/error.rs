use axum::{
    body::Body,
    extract::multipart::{MultipartError, MultipartRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),

    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    MultiPart(#[from] MultipartError),

    #[error(transparent)]
    MultipartRejection(#[from] MultipartRejection),

    #[error(transparent)]
    Minijinja(#[from] minijinja::Error),

    #[error(transparent)]
    HeaderToStr(#[from] axum::http::header::ToStrError),

    #[error(transparent)]
    Http(#[from] axum::http::Error),

    #[error("{0}")]
    Generic(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("{self:#?}");
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal server error"))
            .unwrap()
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Generic(message.to_owned())
    }
}
