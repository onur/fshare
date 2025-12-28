use axum::{
    Router,
    extract::{DefaultBodyLimit, FromRequest, Multipart, Path, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{AppendHeaders, Html, IntoResponse, Response},
    routing::{get, post},
};
use humantime::format_duration;
use minijinja::context;
use tokio::net::TcpListener;

use std::fmt::Write;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::Result;
use crate::config::Config;
use crate::s3::S3Upload;
use crate::store::Store;

#[derive(serde::Serialize)]
struct UploadOutput {
    url: String,
    upload: S3Upload,
}

async fn upload(State(store): State<Arc<Store>>, request: Request) -> Result<impl IntoResponse> {
    let mut uploads = Vec::new();
    let mut duration = Duration::from_secs(60 * store.allowed_durations.first().unwrap_or(&30));

    let origin = get_origin(request.headers());
    let is_terminal = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|ua| !ua.to_lowercase().contains("mozilla"));

    let mut multipart = Multipart::from_request(request, &store).await?;

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("expiration") => {
                if let Some(e) = field
                    .text()
                    .await
                    .ok()
                    .and_then(|t| t.parse().ok())
                    .filter(|e| store.allowed_durations.contains(e))
                {
                    duration = Duration::from_secs(60 * e);
                }
            }
            Some("file") => uploads.push(store.s3.upload_field(&duration, field).await?),
            Some(&_) | None => {}
        }
    }

    if is_terminal {
        return Ok(Response::builder()
            .status(StatusCode::CREATED)
            .header(header::CONTENT_TYPE, "text/html")
            .body(uploads.into_iter().fold(String::new(), |mut output, u| {
                let _ = writeln!(output, "{0}/{1}", origin, u.id);
                output
            }))?);
    }

    let uploads: Vec<UploadOutput> = uploads
        .into_iter()
        .map(|upload| UploadOutput {
            url: format!("{0}/{1}", origin, upload.id),
            upload,
        })
        .collect();

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, "text/html")
        .body(
            store
                .environment
                .get_template("upload.html")?
                .render(context! { uploads })?,
        )?)
}

async fn download(State(store): State<Arc<Store>>, Path(id): Path<String>) -> impl IntoResponse {
    let Some(object) = store.s3.get(&id).await else {
        return Err((StatusCode::NOT_FOUND, "Not found"));
    };

    if object.is_expired() {
        return Err((StatusCode::GONE, "Expired"));
    }

    Ok((AppendHeaders(object.headers()), object.body()))
}

fn index(store: &Store) -> Result<Html<String>> {
    let allowed_expiration_times: Vec<(u64, String)> = store
        .allowed_durations
        .iter()
        .map(|m| (*m, format_duration(Duration::from_secs(60 * m)).to_string()))
        .collect();
    let index = store
        .environment
        .get_template("index.html")?
        .render(context! { allowed_expiration_times })?;
    Ok(Html(index))
}

fn get_origin(headers: &HeaderMap) -> String {
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http")
        .to_string();

    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost")
        .to_string();

    format!("{proto}://{host}")
}

fn router(store: Store) -> Result<Router> {
    let max_upload_size = store.config.max_upload_size * 1024 * 1024;
    let index = index(&store)?;

    Ok(Router::new()
        .route("/", get(|| async move { index }))
        .route("/", post(upload))
        .route("/{id}", get(download))
        .with_state(store.into())
        .layer(DefaultBodyLimit::max(max_upload_size)))
}

pub async fn serve(config: Config) -> Result<()> {
    let addr: SocketAddr = config.socket_addr.parse()?;
    let listener = TcpListener::bind(addr).await?;
    let store = Store::new(config).await?;
    Ok(axum::serve(listener, router(store)?.into_make_service()).await?)
}
