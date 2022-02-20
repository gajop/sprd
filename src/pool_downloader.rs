use std::error::Error;
use std::fs::{self, File};
use std::io::Write;

use hyper::body::HttpBody;
use hyper::http::Request;
use hyper::{Body, Response, Uri};
use hyper_tls::HttpsConnector;

// TODO: Start using tokio file IO again once we fix .write_all()
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt;

use thiserror::Error;

use crate::api::DownloadOptions;
use crate::event::Event;
use crate::validation::validate_sdp_package;

use super::gz;
use super::rapid::{
    rapid_store::RapidStore,
    types::{Repo, Sdp, SdpPackage},
};

pub async fn download_sdp_files(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: &Repo,
    sdp: &Sdp,
    download_map: Vec<u8>,
    sdp_files: &[SdpPackage],
) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/streamer.cgi?{}", repo.url, sdp.md5);
    let url = url.parse::<hyper::Uri>().unwrap();
    // println!("{url}");

    download_sdp_files_with_url(rapid_store, opts, &url, download_map, sdp_files).await
}

struct BufferedReader {
    res: Response<Body>,
    buf: Vec<u8>,
    current: usize,
    progress_function: Option<Box<dyn FnMut(usize)>>,
}

#[derive(Error, Debug)]
enum ReadingError {
    #[error("no more data")]
    NoMoreData,
    #[error("network error")]
    NetworkError,
}

impl BufferedReader {
    pub fn new(res: Response<Body>) -> Self {
        Self {
            res,
            buf: Vec::new(),
            current: 0,
            progress_function: None,
        }
    }

    pub fn set_progress_function(&mut self, f: Box<dyn FnMut(usize)>) -> &mut BufferedReader {
        self.progress_function = Some(f);
        self
    }

    pub async fn read_amount(&mut self, size: usize) -> Result<Vec<u8>, ReadingError> {
        while self.current < size {
            let next = self
                .res
                .data()
                .await
                .ok_or(ReadingError::NoMoreData)?
                .map_err(|_e| ReadingError::NetworkError)?;
            let chunk = next;
            self.current += chunk.len();
            self.buf.extend_from_slice(&chunk[..]);

            if let Some(progress_function) = &mut self.progress_function {
                progress_function(chunk.len());
            }
        }
        self.current -= size;
        let result: Vec<u8> = self.buf.drain(..size).collect();
        Ok(result)
    }

    pub async fn read_remainder(&mut self) -> Result<Vec<u8>, ReadingError> {
        while let Some(next) = self.res.data().await {
            let chunk = next.map_err(|_e| ReadingError::NetworkError)?;
            self.buf.extend_from_slice(&chunk[..]);
        }

        Ok(self.buf.clone())
    }
}

fn slice_to_u4(slice: &[u8]) -> [u8; 4] {
    slice
        .try_into()
        .unwrap_or_else(|e| panic!("slice with incorrect length: {} {e:?}", slice.len()))
}

pub async fn download_sdp_files_with_url(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    url: &Uri,
    download_map: Vec<u8>,
    sdp_files: &[SdpPackage],
) -> Result<(), Box<dyn Error>> {
    assert_ne!(sdp_files.len(), 0);
    assert!(download_map.iter().any(|f| *f != 0));
    let gzipped = gz::gzip_data(download_map.as_slice())?;

    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let req = Request::builder()
        .method("POST")
        .uri(url.to_string())
        .body(hyper::Body::from(gzipped))
        .expect("request builder");

    let res = client.request(req).await?;

    let total_size = res
        .headers()
        .get("content-length")
        .unwrap()
        .to_str()
        .unwrap();
    let total_size = total_size.parse::<i64>().unwrap();

    const LENGTH_SIZE: usize = 4;

    let mut reader = BufferedReader::new(res);
    let mut downloaded_size = 0;
    opts.print
        .event(Event::DownloadStarted(total_size as usize));
    let print_function = opts.print.clone();
    reader.set_progress_function(Box::new(move |downloaded: usize| {
        downloaded_size += downloaded;
        print_function.event(Event::DownloadProgress(downloaded_size));
    }));
    let missing_files = rapid_store.find_missing_files(sdp_files);
    for sdp_package in missing_files.iter() {
        let file_size = reader.read_amount(LENGTH_SIZE).await.map_err(Box::new)?;
        let file_size = u32::from_be_bytes(slice_to_u4(&file_size)) as usize;

        let file_data = reader.read_amount(file_size).await.map_err(Box::new)?;

        let dest = rapid_store.get_pool_path(sdp_package);
        std::fs::create_dir_all(dest.parent().expect("No parent directory"))?;
        let mut file = File::create(&dest)?;
        file.write_all(&file_data)?;
        file.flush()?;
        // let mut file = File::create(&dest).await?;
        // file.write(&file_data).await?;
        // file.flush().await?;

        let file_size_on_disk = fs::metadata(dest).unwrap().len();
        if (file_size_on_disk as usize) != file_size as usize {
            opts.print.event(Event::Error(format!(
                "File size on disk ({file_size_on_disk}) different than in memory ({file_size})"
            )));
        }

        let validation = validate_sdp_package(rapid_store, sdp_package);
        let pool_path = rapid_store.get_pool_path(sdp_package);
        match validation {
            None => {
                // println!("File OK: {pool_path:?}");
            }
            Some(err) => {
                opts.print
                    .event(Event::Error(format!("Invalid file: {err:?} {pool_path:?}")));
            }
        }
    }
    opts.print.event(Event::DownloadFinished {});

    let remaining = reader.read_remainder().await?;
    if !remaining.is_empty() {
        opts.print.event(Event::Error(format!(
            "There are {} bytes remaining in the stream, should be empty.",
            remaining.len()
        )));
    }

    Ok(())
}
