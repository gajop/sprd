use hyper::{Body, Request, Response};
use hyper_tls::HttpsConnector;
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
    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

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
    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

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
