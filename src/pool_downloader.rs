use std::array::TryFromSliceError;
use std::fs::{self, File};
use std::io::Write;

use anyhow::Context;
use hyper::body::HttpBody;
use hyper::{Body, Request, Response, Uri};

// TODO: Start using tokio file IO again once we fix .write_all()
// use tokio::fs::File;
// use tokio::io::AsyncWriteExt;

use thiserror::Error;

use crate::api::DownloadOptions;
use crate::event::Event;
use crate::file_download::FileDownloadError;
use crate::http_download::{http_download_with_request, ResponseWithSize};
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
) -> Result<(), FileDownloadError> {
    let url = format!("{}/streamer.cgi?{}", repo.url, sdp.md5);
    let url = url
        .parse::<hyper::Uri>()
        .map_err(FileDownloadError::InvalidUri)?;

    download_sdp_files_with_url(rapid_store, opts, url, download_map, sdp_files).await
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

fn slice_to_u4(slice: &[u8]) -> Result<[u8; 4], TryFromSliceError> {
    let u4: [u8; 4] = slice.try_into()?;
    Ok(u4)
}

pub async fn download_sdp_files_with_url(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    url: Uri,
    download_map: Vec<u8>,
    sdp_files: &[SdpPackage],
) -> Result<(), FileDownloadError> {
    assert_ne!(sdp_files.len(), 0);
    assert!(download_map.iter().any(|f| *f != 0));
    let gzipped = gz::gzip_data(download_map.as_slice())
        .map_err(|e| FileDownloadError::FilesystemError(e.into()))?;

    let req = Request::builder()
        .method("POST")
        .uri(url)
        .body(hyper::Body::from(gzipped))
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;
    let ResponseWithSize { res, size } = http_download_with_request(req).await?;

    const LENGTH_SIZE: usize = 4;

    let mut reader = BufferedReader::new(res);
    let mut downloaded_size = 0;
    opts.print.event(Event::DownloadStarted(size));
    let print_function = opts.print.clone();
    reader.set_progress_function(Box::new(move |downloaded: usize| {
        downloaded_size += downloaded;
        print_function.event(Event::DownloadProgress(downloaded_size));
    }));
    let missing_files = rapid_store.find_missing_files(sdp_files);
    for sdp_package in missing_files.iter() {
        let file_size = reader
            .read_amount(LENGTH_SIZE)
            .await
            .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;
        let file_size = u32::from_be_bytes(
            slice_to_u4(&file_size)
                .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?,
        ) as usize;

        let file_data = reader
            .read_amount(file_size)
            .await
            .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;

        let dest = rapid_store.get_pool_path(sdp_package);
        std::fs::create_dir_all(
            dest.parent()
                .context(format!("No parent directory for path: {dest:?}"))
                .map_err(FileDownloadError::FilesystemError)?,
        )
        .map_err(|e| FileDownloadError::FilesystemError(e.into()))?;
        let mut file =
            File::create(&dest).map_err(|e| FileDownloadError::FilesystemError(e.into()))?;
        file.write_all(&file_data)
            .map_err(|e| FileDownloadError::FilesystemError(e.into()))?;
        file.flush()
            .map_err(|e| FileDownloadError::FilesystemError(e.into()))?;
        // let mut file = File::create(&dest).await?;
        // file.write(&file_data).await?;
        // file.flush().await?;

        match fs::metadata(&dest) {
            Ok(metadata) => {
                let file_size_on_disk = metadata.len();
                if (file_size_on_disk as usize) != file_size as usize {
                    opts.print.event(Event::Error(format!(
                        "File ({dest:?}) size on disk ({file_size_on_disk}) different than in memory ({file_size})"
                    )));
                }
            }
            Err(err) => {
                opts.print.event(Event::Error(format!(
                    "Cannot obtain file ({dest:?}) disk size ({err:?}). Unable to verify correctness ({file_size})"
                )));
            }
        };

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

    let remaining = reader
        .read_remainder()
        .await
        .map_err(|e| FileDownloadError::InvalidServerResponse(e.into()))?;
    if !remaining.is_empty() {
        opts.print.event(Event::Error(format!(
            "There are {} bytes remaining in the stream, should be empty.",
            remaining.len()
        )));
    }

    Ok(())
}
