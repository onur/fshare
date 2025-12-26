use crate::Result;
use crate::config::Config;
use crate::s3::S3;

pub struct Store {
    pub config: Config,
    pub s3: S3,
    pub environment: minijinja::Environment<'static>,
    pub allowed_durations: Vec<u64>,
}

impl Store {
    pub async fn new(config: Config) -> Result<Self> {
        let s3 = S3::new(&config).await?;

        let mut environment = minijinja::Environment::new();
        environment.add_template("base.html", include_str!("./templates/base.html"))?;
        environment.add_template("index.html", include_str!("./templates/index.html"))?;
        environment.add_template("upload.html", include_str!("./templates/upload.html"))?;

        let allowed_durations: Vec<u64> = config
            .allowed_durations
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        Ok(Self {
            config,
            s3,
            environment,
            allowed_durations,
        })
    }
}
