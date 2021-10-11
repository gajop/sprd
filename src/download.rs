use std::error::Error;
use std::path;
use std::str::FromStr;

use hyper::body::HttpBody;
use hyper::http::Request;
use hyper_tls::HttpsConnector;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use indicatif::{ProgressBar, ProgressStyle};

use super::gz;
use super::rapid::{
    parsing::parse_repos_from_file,
    rapid_store::RapidStore,
    types::{Repo, SDPPackage, SDP},
};

extern crate hyper;

fn get_next_dl_file(
    rapid_store: &RapidStore,
    files: &Vec<SDPPackage>,
    start_index: usize,
) -> Option<usize> {
    for i in start_index..files.len() {
        let file_path = rapid_store.get_pool_path(&files[i]);
        if !file_path.exists() {
            return Some(i);
        }
    }

    None
}

pub async fn download_sdp_files<'a>(
    rapid_store: &RapidStore<'a>,
    repo: &Repo,
    sdp: &SDP,
    download_map: Vec<u8>,
    sdp_files: &Vec<SDPPackage>,
) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/streamer.cgi?{}", repo.url, sdp.md5);
    let url = url.parse::<hyper::Uri>().unwrap();
    // println!("{}", url);

    let gzipped = gz::gzip_data(download_map.as_slice())?;

    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let req = Request::builder()
        .method("POST")
        .uri(url.to_string())
        .body(hyper::Body::from(gzipped))
        .expect("request builder");

    let mut res = client.request(req).await?;

    // let res = client.post(&url).body(gzipped).send().await?;
    // let mut res = client.get(url).await?;

    // println!("Response: {}", res.status());
    // println!("Headers: {:#?}\n", res.headers());

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
    let mut size_bytes: [u8; 4] = [0; 4];
    let mut size = 0;
    let mut read_amount = 0;

    let mut file_index = get_next_dl_file(&rapid_store, &sdp_files, 0).unwrap();
    let mut sdp_file = &sdp_files[file_index];

    let mut dest = rapid_store.get_pool_path(sdp_file);
    std::fs::create_dir_all(dest.parent().unwrap())?;
    let mut file = File::create(dest).await?;
    let mut file_read_size = 0;

    const LENGHT_SIZE: usize = 4;

    let pb_template: String =
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            .to_owned();
    let pb = ProgressBar::new(total_size as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&pb_template)
            .progress_chars("#>-"),
    );

    while let Some(next) = res.data().await {
        let chunk = next?;
        downloaded_size += chunk.len();

        let mut chunk_index = 0;
        while chunk_index < chunk.len() {
            let chunk_remaining = chunk.len() - chunk_index;

            if read_amount < LENGHT_SIZE {
                let read_chunk = chunk_remaining.min(LENGHT_SIZE - read_amount);
                for i in 0..read_chunk {
                    size_bytes[read_amount + i] = chunk[chunk_index + i];
                }
                read_amount += read_chunk;
                chunk_index += read_chunk;

                if read_amount == LENGHT_SIZE {
                    size = u32::from_be_bytes(size_bytes);
                    // println!("File size: {}", size);
                }
            } else {
                let read_chunk = chunk_remaining.min(size as usize - file_read_size);

                file.write(&chunk[chunk_index..chunk_index + read_chunk])
                    .await?;

                file_read_size += read_chunk;
                chunk_index += read_chunk;

                if file_read_size == size as usize {
                    file_index =
                        get_next_dl_file(&rapid_store, &sdp_files, file_index + 1).unwrap();
                    sdp_file = &sdp_files[file_index];

                    dest = rapid_store.get_pool_path(sdp_file);
                    std::fs::create_dir_all(dest.parent().unwrap())?;
                    file = File::create(dest).await?;
                    file_read_size = 0;

                    size = 0;
                    read_amount = 0;

                    // println!("Downloading file: {}", sdp_file.name);
                }
            }
        }

        pb.set_position(downloaded_size as u64);
    }

    Ok(())
}

pub async fn download_sdp(
    rapid_store: &RapidStore<'_>,
    repo: &Repo,
    sdp: &SDP,
) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/packages/{}.sdp", repo.url, sdp.md5);
    let url = hyper::Uri::from_str(&url).unwrap();
    let dest = rapid_store.get_sdp_path_from_md5(&sdp.md5);
    download_file(url, &dest, "Downloading SDP").await
}

pub async fn download_all_repos(rapid_store: &RapidStore<'_>) -> Result<(), Box<dyn Error>> {
    let registry_file = rapid_store.get_registry_path();
    let repos = parse_repos_from_file(&registry_file)?;
    for repo in repos {
        download_repo(&rapid_store, &repo).await?;
    }

    Ok(())
}

pub async fn download_repo(
    rapid_store: &RapidStore<'_>,
    repo: &Repo,
) -> Result<(), Box<dyn Error>> {
    let repo_file = rapid_store.get_repo_path(repo);
    let versions_url = repo.url.to_owned() + "/versions.gz";

    let url = hyper::Uri::from_str(&versions_url).unwrap();
    download_file(url, &repo_file, "Downloading repository").await
}

pub async fn download_repo_registry(rapid_store: &RapidStore<'_>) -> Result<(), Box<dyn Error>> {
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
    let mut file = File::create(dest).await?;

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

        file.write_all(&chunk).await?;
    }
    pb.finish_with_message("downloaded");

    Ok(())
}
