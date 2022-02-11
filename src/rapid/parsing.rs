use std::error::Error;
use std::fmt::Write;
use std::path;
use std::str;
use std::u32;

use super::types::{Repo, Sdp, SdpPackage};

pub fn parse_repos_from_file(path: &path::Path) -> Result<Vec<Repo>, Box<dyn Error>> {
    let s = crate::gz::read_gz_from_file(path)?;
    parse_repos_from_str(&s)
}

pub fn parse_repos_from_str(s: &str) -> Result<Vec<Repo>, Box<dyn Error>> {
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

    Ok(entries)
}

pub fn read_rapid_from_file(path: &path::Path) -> Result<Vec<Sdp>, Box<dyn Error>> {
    let parsed_gz = crate::gz::read_gz_from_file(path)?;
    read_rapid_from_str(&parsed_gz)
}

pub fn read_rapid_from_str(parsed_gz: &str) -> Result<Vec<Sdp>, Box<dyn Error>> {
    let mut entries = Vec::new();

    for line in parsed_gz.lines() {
        let line_entry: Vec<&str> = line.split(',').collect();
        if line_entry.len() != 4 {
            println!("MALFORMED FILE");
            continue;
        }
        entries.push(Sdp {
            fullname: line_entry[0].to_string(),
            md5: line_entry[1].to_string(),
            something: line_entry[3].to_string(),
            alias: line_entry[2].to_string(),
        });
    }

    Ok(entries)
}

pub fn load_sdp_packages_from_file(dest: &path::Path) -> Result<Vec<SdpPackage>, Box<dyn Error>> {
    let data = crate::gz::read_binary_gz_from_file(dest)?;

    load_sdp_packages(&data)
}

pub fn load_sdp_packages(data: &[u8]) -> Result<Vec<SdpPackage>, Box<dyn Error>> {
    let mut sdp_files = Vec::new();

    let mut index = 0;
    while index < data.len() {
        let length = data[index] as usize;
        index += 1;

        let name = str::from_utf8(&data[index..index + length]).unwrap();
        index += length;

        let md5_bin = &data[index..index + 16];
        index += 16;
        let crc32 = &data[index..index + 4];
        index += 4;
        let mut size: [u8; 4] = [0; 4];
        size.copy_from_slice(&data[index..index + 4]);
        index += 4;

        let mut md5 = String::new();
        for byte in md5_bin {
            write!(md5, "{:02x}", byte)?;
        }

        let mut sdp_file = SdpPackage {
            name: name.to_owned(),
            ..Default::default()
        };
        sdp_file.name = name.to_owned();
        sdp_file.crc32.copy_from_slice(crc32);

        let md5_chars: Vec<char> = md5.chars().collect();
        sdp_file.md5[..md5_chars.len()].clone_from_slice(&md5_chars[..]);
        sdp_file.size = u32::from_le_bytes(size);

        sdp_files.push(sdp_file);
    }

    Ok(sdp_files)
}
