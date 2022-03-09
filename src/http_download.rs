use hyper::{Body, Client, Request, Response};
use hyper_rustls::HttpsConnectorBuilder;
use thiserror::Error;

use crate::file_download::FileDownloadError;

#[derive(Debug, Error)]
#[error("Missing content length")]
struct MissingContentLength {}

pub struct ResponseWithSize {
    pub res: Response<Body>,
    pub size: usize,
}

pub async fn http_download_with_url(
    url: hyper::Uri,
) -> Result<ResponseWithSize, FileDownloadError> {
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();
    let client = Client::builder().build::<_, Body>(https);

    let res = client
        .get(url.clone())
        .await
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;

    let total_size = res
        .headers()
        .get("content-length")
        .ok_or_else(|| FileDownloadError::InvalidServerResponse(MissingContentLength {}.into()))?
        .to_str()
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;
    let total_size = total_size
        .parse::<i64>()
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;

    Ok(ResponseWithSize {
        res,
        size: total_size as usize,
    })
}

pub async fn http_download_with_request(
    request: Request<Body>,
) -> Result<ResponseWithSize, FileDownloadError> {
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();
    let client = Client::builder().build::<_, Body>(https);

    let res = client
        .request(request)
        .await
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;

    let total_size = res
        .headers()
        .get("content-length")
        .ok_or_else(|| FileDownloadError::InvalidServerResponse(MissingContentLength {}.into()))?
        .to_str()
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;
    let total_size = total_size
        .parse::<i64>()
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;

    Ok(ResponseWithSize {
        res,
        size: total_size as usize,
    })
}
