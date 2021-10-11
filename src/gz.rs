use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path;

use flate2::read::GzDecoder;
use flate2::read::GzEncoder;

pub fn read_gz_from_file(path: &path::Path) -> Result<String, Box<dyn Error>> {
    let data = read_binary_file(path)?;
    let unzipped = read_gz_from_data(data.as_slice())?;

    Ok(unzipped)
}

pub fn read_gz_from_data(data: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut d = GzDecoder::new(&data[..]);
    let mut s = String::new();
    d.read_to_string(&mut s)?;

    Ok(s)
}

pub fn read_binary_gz_from_file(path: &path::Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let data = read_binary_file(path)?;
    let unzipped = read_binary_gz_from_data(data.as_slice())?;

    Ok(unzipped)
}

pub fn read_binary_gz_from_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut d = GzDecoder::new(&data[..]);
    let mut s = Vec::new();
    d.read_to_end(&mut s)?;

    Ok(s)
}

fn read_binary_file(path: &path::Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut versions_file = fs::File::open(path)?;
    let mut contents = Vec::new();
    versions_file.read_to_end(&mut contents)?;

    Ok(contents)
}

pub fn gzip_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut d = GzEncoder::new(&data[..], flate2::Compression::default());
    let mut out = Vec::new();
    d.read_to_end(&mut out)?;

    Ok(out)
}
