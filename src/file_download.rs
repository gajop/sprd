use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path;
use std::str::FromStr;

use hyper::body::HttpBody;
use hyper_tls::HttpsConnector;
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt;

use indicatif::{ProgressBar, ProgressStyle};

use super::rapid::{
    parsing::parse_repos_from_file,
    rapid_store::RapidStore,
    types::{Repo, Sdp},
};

pub async fn download_sdp(
    rapid_store: &RapidStore,
    repo: &Repo,
    sdp: &Sdp,
) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/packages/{}.sdp", repo.url, sdp.md5);
    let url = hyper::Uri::from_str(&url).unwrap();
    let dest = rapid_store.get_sdp_path_from_md5(&sdp.md5);
    download_file(url, &dest, "Downloading SDP").await
}

pub async fn download_all_repos(rapid_store: &RapidStore) -> Result<(), Box<dyn Error>> {
    let registry_file = rapid_store.get_registry_path();
    let repos = parse_repos_from_file(&registry_file)?;
    for repo in repos {
        download_repo(rapid_store, &repo).await?;
    }

    Ok(())
}

pub async fn download_repo(rapid_store: &RapidStore, repo: &Repo) -> Result<(), Box<dyn Error>> {
    let repo_file = rapid_store.get_repo_path(repo);
    let versions_url = repo.url.to_owned() + "/versions.gz";

    let url = hyper::Uri::from_str(&versions_url).unwrap();
    download_file(url, &repo_file, "Downloading repository").await
}

pub async fn download_repo_registry(rapid_store: &RapidStore) -> Result<(), Box<dyn Error>> {
    let url = hyper::Uri::from_static("https://repos.springrts.com/repos.gz");
    let registry_file = rapid_store.get_registry_path();
    download_file(url, &registry_file, "Downloading registry").await
}

pub async fn download_file(
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

    let mut pb_template: String =
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            .to_owned();
    pb_template.push_str(title);

    let pb = ProgressBar::new(total_size as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&pb_template)
            .progress_chars("#>-"),
    );

    while let Some(next) = res.data().await {
        let chunk = next?;
        downloaded_size += chunk.len();
        pb.set_position(downloaded_size as u64);

        // file.write_all(&chunk).await?;
        file.write_all(&chunk)?;
    }
    pb.finish_with_message("downloaded");

    Ok(())
}
