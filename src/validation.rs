use std::path::{Path, PathBuf};

use md5::{Digest, Md5};

use crate::{
    api::DownloadOptions,
    metadata::{self, MetadataQueryError},
    rapid::{rapid_store::RapidStore, types::SdpPackage},
};

use super::rapid::{parsing, rapid_store};

#[derive(Debug)]
pub enum FileError {
    Missing,
    Corrupt,
    WrongHash,
}

pub enum ValidityErrors {
    MetadataQueryError(MetadataQueryError),
    MissingSdp,
    InvalidFiles { files: Vec<(PathBuf, FileError)> },
}

pub fn check_if_sdp_needs_download(rapid_store: &rapid_store::RapidStore, md5: &str) -> bool {
    validate_by_sdp_md5(rapid_store, md5).is_err()
}

pub async fn validate_by_fullname(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) -> Result<(), ValidityErrors> {
    let query = metadata::query_metadata(rapid_store, opts, fullname)
        .await
        .map_err(ValidityErrors::MetadataQueryError)?;
    let (_, sdp) = query.ok_or(ValidityErrors::MissingSdp)?;
    validate_by_sdp_md5(rapid_store, &sdp.md5)
}

pub fn validate_by_sdp_md5(rapid_store: &RapidStore, md5: &str) -> Result<(), ValidityErrors> {
    let sdp_path = rapid_store.get_sdp_path_from_md5(md5);

    let sdp_packages = parsing::load_sdp_packages_from_file(&sdp_path)
        .map_err(|e| ValidityErrors::MetadataQueryError(MetadataQueryError::CorruptFile(e)))?;

    let mut files_with_errors = Vec::new();

    for package in sdp_packages.iter() {
        let validation = validate_sdp_package(rapid_store, package);
        let pool_path = rapid_store.get_pool_path(package);
        if let Some(file_error) = validation {
            files_with_errors.push((pool_path, file_error));
        }
    }

    if files_with_errors.is_empty() {
        return Ok(());
    }

    Err(ValidityErrors::InvalidFiles {
        files: files_with_errors,
    })
}

pub fn validate_sdp_package(rapid_store: &RapidStore, package: &SdpPackage) -> Option<FileError> {
    let pool_path = rapid_store.get_pool_path(package);
    validate_sdp_package_with_path(&pool_path, package.md5_bin)
}

pub fn validate_sdp_package_with_path(path: &Path, md5_bin: [u8; 16]) -> Option<FileError> {
    if !path.exists() {
        return Some(FileError::Missing);
    }

    let parsed_gz = match crate::gz::read_binary_gz_from_file(path) {
        Ok(path) => path,
        Err(_) => {
            return Some(FileError::Corrupt);
        }
    };

    let mut hasher = Md5::new();
    hasher.update(parsed_gz);
    let hashed = &hasher.finalize()[..];

    if hashed != md5_bin {
        return Some(FileError::WrongHash);
    }

    None
}

#[cfg(test)]
mod tests {

    use crate::api::MetadataSource;

    use super::*;

    #[test]
    fn no_file() {
        let springdir = tempfile::tempdir().unwrap();
        let rapid_store = RapidStore::new(springdir.into_path());

        assert!(check_if_sdp_needs_download(&rapid_store, "test"));
        assert!(check_if_sdp_needs_download(&rapid_store, ""));
    }

    #[tokio::test]
    async fn check_prd_tag() {
        let rapid_store = RapidStore::new(test_utils::setup_pr_downloader_folders());
        assert!(validate_by_fullname(
            &rapid_store,
            &DownloadOptions {
                metadata_source: MetadataSource::Local
            },
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
        )
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn check_sprd_tag() {
        let rapid_store = RapidStore::new(test_utils::setup_sprd_folders().await);
        assert!(validate_by_fullname(
            &rapid_store,
            &DownloadOptions {
                metadata_source: MetadataSource::Local
            },
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
        )
        .await
        .is_ok());
    }
}
