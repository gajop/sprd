use std::fs::File;
use std::io::Write;
use std::path;
use std::str::FromStr;

use hyper::body::HttpBody;
use thiserror::Error;
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt;

use crate::{
    api::DownloadOptions,
    event::Event,
    http_download::{http_download_with_url, ResponseWithSize},
};

#[derive(Debug, Error)]
pub enum FileDownloadError {
    #[error("invalid Uri")]
    InvalidUri(#[source] hyper::http::uri::InvalidUri),

    #[error("invalid server response")]
    InvalidServerResponse(#[source] anyhow::Error),

    #[error("filesystem error")]
    FilesystemError(#[source] anyhow::Error),
}

#[derive(Debug, Error)]
#[error("No parent folder")]
struct NoParentFolder {}

use super::rapid::{
    parsing::parse_repos_from_file,
    rapid_store::RapidStore,
    types::{Repo, Sdp},
};

pub async fn download_sdp(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: &Repo,
    sdp: &Sdp,
) -> Result<(), FileDownloadError> {
    let url = format!("{}/packages/{}.sdp", repo.url, sdp.md5);
    let url = hyper::Uri::from_str(&url).map_err(FileDownloadError::InvalidUri)?;
    let dest = rapid_store.get_sdp_path_from_md5(&sdp.md5);
    download_file(opts, url, &dest, "Downloading SDP").await
}

pub async fn download_all_repos(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
) -> Result<(), FileDownloadError> {
    let registry_file = rapid_store.get_registry_path();
    let repos = parse_repos_from_file(&registry_file)
        .map_err(|e| FileDownloadError::FilesystemError(e.into()))?;
    for repo in repos {
        download_repo(rapid_store, opts, &repo).await?;
    }

    Ok(())
}

pub async fn download_repo(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: &Repo,
) -> Result<(), FileDownloadError> {
    let repo_file = rapid_store.get_repo_path(repo);
    let versions_url = repo.url.to_owned() + "/versions.gz";

    let url = hyper::Uri::from_str(&versions_url).map_err(FileDownloadError::InvalidUri)?;
    download_file(opts, url, &repo_file, "Downloading repository").await
}

pub async fn download_repo_registry(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
) -> Result<(), FileDownloadError> {
    let url = hyper::Uri::from_static("https://repos.springrts.com/repos.gz");
    let registry_file = rapid_store.get_registry_path();
    download_file(opts, url, &registry_file, "Downloading registry").await
}

pub async fn download_file(
    opts: &DownloadOptions,
    url: hyper::Uri,
    dest: &path::Path,
    title: &str,
) -> Result<(), FileDownloadError> {
    let ResponseWithSize { mut res, size } = http_download_with_url(url).await?;

    let mut downloaded_size = 0;
    std::fs::create_dir_all(dest.parent().ok_or_else(|| {
        FileDownloadError::FilesystemError(anyhow::Error::new(NoParentFolder {}))
    })?)
    .map_err(|e| FileDownloadError::FilesystemError(e.into()))?;
    // let mut file = File::create(dest).await?;
    let mut file = File::create(dest).map_err(|e| FileDownloadError::FilesystemError(e.into()))?;

    opts.print
        .event(Event::Info(format!("Downloading {title}")));
    opts.print.event(Event::DownloadStarted(size));

    while let Some(next) = res.data().await {
        let chunk = next.map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;
        downloaded_size += chunk.len();
        opts.print.event(Event::DownloadProgress(downloaded_size));

        // file.write_all(&chunk).await?;
        file.write_all(&chunk)
            .map_err(|e| FileDownloadError::FilesystemError(e.into()))?;
    }
    opts.print.event(Event::DownloadFinished);

    Ok(())
}
