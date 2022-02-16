use crate::{
    api::{DownloadOptions, MetadataSource},
    file_download,
    rapid::{self, types::SdpPackage},
    rapid::{
        rapid_store::RapidStore,
        types::{Repo, Sdp},
    },
};

mod metadata_file;
mod metadata_local;
mod metadata_rest;

pub async fn query_metadata(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) -> (Repo, Sdp) {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_metadata(rapid_store, fullname).await,
        MetadataSource::FileApi => metadata_file::query_metadata(rapid_store, fullname).await,
        MetadataSource::RestApi(api_server) => {
            metadata_rest::query_metadata(api_server, fullname).await
        }
    }
}

pub async fn query_repo(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo_basename: &str,
) -> Option<Repo> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_repo(rapid_store, repo_basename)
            .await
            .unwrap(),
        MetadataSource::FileApi => metadata_file::query_repo(rapid_store, repo_basename).await,
        MetadataSource::RestApi(api_server) => {
            metadata_rest::query_repo(api_server, repo_basename).await
        }
    }
}

pub async fn query_sdp(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: &Repo,
    tag: &str,
) -> Option<Sdp> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_sdp(rapid_store, repo, tag).await,
        MetadataSource::FileApi => metadata_file::query_sdp(rapid_store, repo, tag).await,
        MetadataSource::RestApi(api_server) => {
            Some(metadata_rest::query_sdp(api_server, &format!("{}:{}", &repo.name, tag)).await)
        }
    }
}

pub async fn query_sdp_files(rapid_store: &RapidStore, repo: &Repo, sdp: &Sdp) -> Vec<SdpPackage> {
    let dest_sdp = rapid_store.get_sdp_path(sdp);
    if !dest_sdp.exists() {
        match file_download::download_sdp(rapid_store, repo, sdp).await {
            Ok(_) => {}
            Err(err) => {
                panic!("Failed to download SDP: {err}");
            }
        }
    }
    assert!(dest_sdp.exists());

    rapid::parsing::load_sdp_packages_from_file(&dest_sdp)
        .expect("Failed to load SDP Package from file")
}

#[cfg(test)]

mod tests {

    use crate::api;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_query_metadata() {
        let rapid_store = rapid::rapid_store::RapidStore::default();
        let (_, sdp) = query_metadata(
            &rapid_store,
            &api::DownloadOptions::default(),
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
        )
        .await;

        assert_eq!(sdp.md5, "d80d786597510d1358be3b04a7e9146e");
    }
}
