use aws_sdk_s3::{
    Client,
    primitives::{ByteStream, DateTime},
    types::{CompletedMultipartUpload, CompletedPart},
};
use axum::extract::multipart::Field;
use bytes::{BufMut, BytesMut};
use rand::Rng;

use std::{
    io,
    time::{Duration, SystemTime},
};

use crate::Result;
use crate::config::Config;
use crate::object::Object;

const BUFFER_SIZE: usize = 5 * 1024 * 1024;

pub struct S3 {
    client: Client,
    bucket: String,
    id_length: u8,
}

#[derive(serde::Serialize)]
pub struct S3Upload {
    pub id: String,
    pub length: u64,
    pub file_name: String,
    pub content_type: String,
    pub expiration_date: String,
}

impl S3 {
    pub async fn new(config: &Config) -> Result<Self> {
        let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .build();

        Ok(Self {
            client: Client::from_conf(s3_config),
            bucket: config.aws_bucket.clone(),
            id_length: config.id_length,
        })
    }

    fn generate_id(&self) -> String {
        let mut rng = rand::rng();
        (0..self.id_length)
            .map(|_| rng.sample(rand::distr::Alphanumeric) as char)
            .collect()
    }

    pub async fn upload_field(
        &self,
        duration: &Duration,
        mut field: Field<'_>,
    ) -> Result<S3Upload> {
        let id = self.generate_id();
        let expiration_date = DateTime::from(SystemTime::now() + *duration);
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let multipart_upload = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(&id)
            .content_type(&content_type)
            .metadata("file_name", &file_name)
            .expires(expiration_date)
            .send()
            .await
            .map_err(io::Error::other)?;

        let upload_id = multipart_upload
            .upload_id()
            .ok_or("Failed to get upload id")?;

        let mut completed_parts = Vec::new();
        let mut part_number = 1;
        let mut length = 0;

        loop {
            let mut buffer = BytesMut::new();

            while let Some(chunk) = field.chunk().await? {
                buffer.put(chunk);
                if buffer.len() > BUFFER_SIZE {
                    break;
                }
            }

            if buffer.is_empty() {
                break;
            }

            length += buffer.len() as u64;

            let upload_part = self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(&id)
                .upload_id(upload_id)
                .body(ByteStream::from(buffer.freeze()))
                .part_number(part_number)
                .send()
                .await
                .map_err(io::Error::other)?;

            completed_parts.push(
                CompletedPart::builder()
                    .e_tag(
                        upload_part
                            .e_tag()
                            .ok_or("Failed to get e_tag from upload part")?,
                    )
                    .part_number(part_number)
                    .build(),
            );
            part_number += 1;
        }

        let completed_upload = CompletedMultipartUpload::builder()
            .set_parts(Some(completed_parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(&id)
            .upload_id(upload_id)
            .multipart_upload(completed_upload)
            .send()
            .await
            .map_err(io::Error::other)?;

        Ok(S3Upload {
            id,
            length,
            file_name,
            content_type,
            expiration_date: expiration_date.to_string(),
        })
    }

    pub async fn get(&self, id: &str) -> Option<Object> {
        self.client
            .get_object()
            .bucket(&self.bucket)
            .key(id)
            .send()
            .await
            .map(Object)
            .ok()
    }
}
