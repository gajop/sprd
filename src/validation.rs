use std::path::PathBuf;

use md5::{Digest, Md5};

use crate::{
    api::DownloadOptions,
    metadata::{self, MetadataQueryError},
    rapid::rapid_store::RapidStore,
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
        let pool_path = rapid_store.get_pool_path(package);

        if !pool_path.exists() {
            files_with_errors.push((pool_path, FileError::Missing));
            continue;
        }

        let parsed_gz = match crate::gz::read_binary_gz_from_file(&pool_path) {
            Ok(pool_path) => pool_path,
            Err(_) => {
                files_with_errors.push((pool_path, FileError::Corrupt));
                continue;
            }
        };

        let mut hasher = Md5::new();
        hasher.update(parsed_gz);
        let result = &hasher.finalize()[..];

        if result != package.md5_bin {
            files_with_errors.push((pool_path, FileError::WrongHash));
        }
    }

    if files_with_errors.is_empty() {
        return Ok(());
    }

    Err(ValidityErrors::InvalidFiles {
        files: files_with_errors,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
        let rapid_store = RapidStore::new(PathBuf::from("test_folders/test_prd"));
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
        let rapid_store = RapidStore::new(PathBuf::from("test_folders/test_sprd"));
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
