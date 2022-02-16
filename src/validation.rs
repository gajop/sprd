use md5::{Digest, Md5};

use crate::{api::DownloadOptions, metadata, rapid::rapid_store::RapidStore};

use super::rapid::{parsing, rapid_store};

pub fn check_if_sdp_needs_download(rapid_store: &rapid_store::RapidStore, md5: &str) -> bool {
    let sdp_path = rapid_store.get_sdp_path_from_md5(md5);

    if !sdp_path.exists() {
        println!("NO SDP PATH");
        return true;
    }

    let sdp_packages = parsing::load_sdp_packages_from_file(&sdp_path);
    if sdp_packages.is_err() {
        println!("PACKAGES ARE ERR");
        return false;
    }
    let sdp_packages = sdp_packages.unwrap();

    // if rapid_store.find_missing_files(&sdp_packages).is_empty() {
    //     println!("MISSING FILES");
    //     return true;
    // }

    for package in sdp_packages.iter() {
        let pool_path = rapid_store.get_pool_path(package);
        println!("pool_path: {pool_path:?}");

        let parsed_gz = crate::gz::read_binary_gz_from_file(&pool_path).unwrap();
        // create a Md5 hasher instance
        let mut hasher = Md5::new();

        // process input message
        hasher.update(parsed_gz);

        // acquire hash digest in the form of GenericArray,
        // which in this case is equivalent to [u8; 16]
        let result = &hasher.finalize()[..];

        // let md5_chars =
        // package.md5
        if result != package.md5_bin {
            println!("MD5 diff: {:?} x {:?}", result, package.md5_bin);
            return true;
        }

        // package.md5
    }

    false
}

pub async fn check_if_tag_is_valid(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) -> bool {
    let (_, sdp) = metadata::query_metadata(rapid_store, opts, fullname).await;

    !check_if_sdp_needs_download(rapid_store, &sdp.md5)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{api::MetadataSource, rapid};

    use super::*;

    #[test]
    fn no_file() {
        let springdir = tempfile::tempdir().unwrap();
        let rapid_store = rapid_store::RapidStore {
            root_folder: springdir.into_path(),
        };

        assert!(check_if_sdp_needs_download(&rapid_store, "test"));
        assert!(check_if_sdp_needs_download(&rapid_store, ""));
    }

    #[tokio::test]
    async fn check_prd_tag() {
        let rapid_store = rapid::rapid_store::RapidStore {
            root_folder: PathBuf::from("test_folders/test_prd"),
        };
        assert!(
            check_if_tag_is_valid(
                &rapid_store,
                &DownloadOptions {
                    metadata_source: MetadataSource::Local
                },
                "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
            )
            .await
        );
    }

    #[tokio::test]
    async fn check_sprd_tag() {
        let rapid_store = rapid::rapid_store::RapidStore {
            root_folder: PathBuf::from("test_folders/test_sprd"),
        };
        assert!(
            check_if_tag_is_valid(
                &rapid_store,
                &DownloadOptions {
                    metadata_source: MetadataSource::Local
                },
                "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
            )
            .await
        );
    }
}
