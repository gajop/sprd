use std::path;
use std::str;
use std::u32;

use thiserror::Error;

use crate::gz::GzReadError;

use super::types::{Repo, Sdp, SdpPackage};

#[derive(Error, Debug)]
#[error("Corrupt Sdp Package")]
pub struct CorruptSdpPackage {
    #[source]
    source: anyhow::Error,
}

pub fn parse_repos_from_file(path: &path::Path) -> Result<Vec<Repo>, GzReadError> {
    let s = crate::gz::read_gz_from_file(path)?;
    Ok(parse_repos_from_str(&s))
}

pub fn parse_repos_from_str(s: &str) -> Vec<Repo> {
    let mut entries = Vec::new();

    for line in s.lines() {
        let line_entry: Vec<&str> = line.split(',').collect();
        let name = line_entry[0];
        let url = line_entry[1];

        entries.push(Repo {
            name: name.to_string(),
            url: url.to_string(),
        });
    }

    entries
}

pub fn read_rapid_from_file(path: &path::Path) -> Result<Vec<Sdp>, GzReadError> {
    let parsed_gz = crate::gz::read_gz_from_file(path)?;
    Ok(read_rapid_from_str(&parsed_gz))
}

pub fn read_rapid_from_str(parsed_gz: &str) -> Vec<Sdp> {
    let mut entries = Vec::new();

    for line in parsed_gz.lines() {
        let line_entry: Vec<&str> = line.split(',').collect();
        if line_entry.len() != 4 {
            println!("MALFORMED FILE"); // ignore?
            continue;
        }
        entries.push(Sdp {
            rapid_name: line_entry[0].to_string(),
            md5: line_entry[1].to_string(),
            depends: line_entry[2].to_string(),
            archive_name: line_entry[3].to_string(),
            // something: line_entry[3].to_string(),
            // alias: line_entry[2].to_string(),
        });
    }

    entries
}

pub fn load_sdp_packages_from_file(
    dest: &path::Path,
) -> Result<Vec<SdpPackage>, CorruptSdpPackage> {
    let data = crate::gz::read_binary_gz_from_file(dest)
        .map_err(|e| CorruptSdpPackage { source: e.into() })?;

    load_sdp_packages(&data)
}

pub fn load_sdp_packages(data: &[u8]) -> Result<Vec<SdpPackage>, CorruptSdpPackage> {
    let mut sdp_files = Vec::new();

    let mut index = 0;
    while index < data.len() {
        let length = data[index] as usize;
        index += 1;

        let name = str::from_utf8(&data[index..index + length])
            .map_err(|e| CorruptSdpPackage { source: e.into() })?;
        index += length;

        let md5_bin = &data[index..index + 16];
        index += 16;
        let crc32 = &data[index..index + 4];
        index += 4;
        let mut size: [u8; 4] = [0; 4];
        size.copy_from_slice(&data[index..index + 4]);
        index += 4;

        let mut sdp_file = SdpPackage {
            name: name.to_owned(),
            ..Default::default()
        };
        sdp_file.name = name.to_owned();
        for (i, byte) in md5_bin.iter().enumerate() {
            let hex = format!("{:02x}", byte);
            sdp_file.md5[2 * i..=2 * i + 1].copy_from_slice(hex.as_bytes());
        }
        sdp_file.md5_bin.copy_from_slice(md5_bin);
        sdp_file.crc32.copy_from_slice(crc32);
        sdp_file.size = u32::from_le_bytes(size);

        sdp_files.push(sdp_file);
    }

    Ok(sdp_files)
}
