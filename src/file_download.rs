use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path;
use std::str::FromStr;

use hyper::body::HttpBody;
use hyper_tls::HttpsConnector;
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt;

use crate::{api::DownloadOptions, event::Event};

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
) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/packages/{}.sdp", repo.url, sdp.md5);
    let url = hyper::Uri::from_str(&url).unwrap();
    let dest = rapid_store.get_sdp_path_from_md5(&sdp.md5);
    download_file(opts, url, &dest, "Downloading SDP").await
}

pub async fn download_all_repos(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
) -> Result<(), Box<dyn Error>> {
    let registry_file = rapid_store.get_registry_path();
    let repos = parse_repos_from_file(&registry_file)?;
    for repo in repos {
        download_repo(rapid_store, opts, &repo).await?;
    }

    Ok(())
}

pub async fn download_repo(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: &Repo,
) -> Result<(), Box<dyn Error>> {
    let repo_file = rapid_store.get_repo_path(repo);
    let versions_url = repo.url.to_owned() + "/versions.gz";

    let url = hyper::Uri::from_str(&versions_url).unwrap();
    download_file(opts, url, &repo_file, "Downloading repository").await
}

pub async fn download_repo_registry(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
) -> Result<(), Box<dyn Error>> {
    let url = hyper::Uri::from_static("https://repos.springrts.com/repos.gz");
    let registry_file = rapid_store.get_registry_path();
    download_file(opts, url, &registry_file, "Downloading registry").await
}

pub async fn download_file(
    opts: &DownloadOptions,
    url: hyper::Uri,
    dest: &path::Path,
    title: &str,
) -> Result<(), Box<dyn Error>> {
    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let mut res = client.get(url).await?;

    let total_size = res
        .headers()
        .get("content-length")
        .unwrap()
        .to_str()
        .unwrap();
    let total_size = total_size.parse::<i64>().unwrap();

    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    let mut downloaded_size = 0;
    std::fs::create_dir_all(dest.parent().unwrap())?;
    // let mut file = File::create(dest).await?;
    let mut file = File::create(dest)?;

    opts.print
        .event(Event::Info(format!("Downloading {title}")));
    opts.print
        .event(Event::DownloadStarted(total_size as usize));

    while let Some(next) = res.data().await {
        let chunk = next?;
        downloaded_size += chunk.len();
        opts.print.event(Event::DownloadProgress(downloaded_size));

        // file.write_all(&chunk).await?;
        file.write_all(&chunk)?;
    }
    opts.print.event(Event::DownloadFinished);

    Ok(())
}
