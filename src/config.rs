use serde::Deserialize;

use crate::Result;

#[derive(Deserialize)]
pub struct Config {
    pub aws_bucket: String,
    pub max_upload_size: usize,
    pub allowed_durations: String,
    pub id_length: u8,
    pub socket_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(config::Config::builder()
            .add_source(config::Environment::default())
            .set_default("max_upload_size", 10)?
            .set_default("allowed_durations", "30,60,360,1440,10080")?
            .set_default("id_length", 8)?
            .set_default("socket_addr", "0.0.0.0:8080")?
            .build()?
            .try_deserialize()?)
    }
}
