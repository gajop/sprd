use std::fs;
use std::io::prelude::*;
use std::path;

use flate2::read::GzDecoder;
use flate2::read::GzEncoder;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("GzReadError")]
pub struct GzReadError {
    source: anyhow::Error,
}

pub fn read_gz_from_file(path: &path::Path) -> Result<String, GzReadError> {
    let data = read_binary_file(path)?;
    let unzipped = read_gz_from_data(data.as_slice())?;

    Ok(unzipped)
}

pub fn read_gz_from_data(data: &[u8]) -> Result<String, GzReadError> {
    let mut d = GzDecoder::new(data);
    let mut s = String::new();
    d.read_to_string(&mut s)
        .map_err(|e| GzReadError { source: e.into() })?;

    Ok(s)
}

pub fn read_binary_gz_from_file(path: &path::Path) -> Result<Vec<u8>, GzReadError> {
    let data = read_binary_file(path)?;
    let unzipped = read_binary_gz_from_data(data.as_slice())?;

    Ok(unzipped)
}

pub fn read_binary_gz_from_data(data: &[u8]) -> Result<Vec<u8>, GzReadError> {
    let mut d = GzDecoder::new(data);
    let mut s = Vec::new();
    d.read_to_end(&mut s)
        .map_err(|e| GzReadError { source: e.into() })?;

    Ok(s)
}

fn read_binary_file(path: &path::Path) -> Result<Vec<u8>, GzReadError> {
    let mut versions_file = fs::File::open(path).map_err(|e| GzReadError { source: e.into() })?;
    let mut contents = Vec::new();
    versions_file
        .read_to_end(&mut contents)
        .map_err(|e| GzReadError { source: e.into() })?;

    Ok(contents)
}

pub fn gzip_data(data: &[u8]) -> Result<Vec<u8>, GzReadError> {
    let mut d = GzEncoder::new(data, flate2::Compression::default());
    let mut out = Vec::new();
    d.read_to_end(&mut out)
        .map_err(|e| GzReadError { source: e.into() })?;

    Ok(out)
}
