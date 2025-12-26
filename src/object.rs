use aws_sdk_s3::operation::get_object::GetObjectOutput;
use axum::{
    body::Body,
    http::{Uri, header},
};
use chrono::DateTime;
use tokio_util::io::ReaderStream;

use std::str::FromStr;

pub struct Object(pub GetObjectOutput);

impl Object {
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self
            .0
            .expires_string()
            .and_then(|e| DateTime::parse_from_rfc2822(e).ok())
            && expiration < chrono::Utc::now()
        {
            return true;
        }
        false
    }

    pub fn headers(&self) -> Vec<(header::HeaderName, String)> {
        let mut headers = vec![(
            header::CONTENT_TYPE,
            self.0
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string(),
        )];

        if let Some(etag) = self.0.e_tag() {
            headers.push((header::ETAG, etag.to_string()));
        }

        if let Some(length) = self.0.content_length() {
            headers.push((header::CONTENT_LENGTH, length.to_string()));
        }

        if let Some(content_type) = self.0.content_type()
            && !["text/plain", "image/", "video/mp4"]
                .iter()
                .any(|t| content_type.contains(t))
            && let Some(filename) = self
                .0
                .metadata()
                .and_then(|m| m.get("file_name"))
                .and_then(|f| Uri::from_str(f).ok())
                .map(|u| u.to_string())
        {
            headers.push((
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ));
        }

        headers
    }

    pub fn body(self) -> Body {
        Body::from_stream(ReaderStream::new(self.0.body.into_async_read()))
    }
}
