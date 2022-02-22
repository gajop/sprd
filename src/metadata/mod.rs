use thiserror::Error;

use crate::{
    api::{DownloadOptions, MetadataSource},
    rapid::types::SdpPackage,
    rapid::{
        rapid_store::RapidStore,
        types::{Repo, Sdp},
    },
};

mod metadata_file;
mod metadata_local;
mod metadata_rest;

#[derive(Error, Debug)]
pub enum MetadataQueryError {
    #[error("corrupt file")]
    CorruptFile(#[source] anyhow::Error),

    #[error("download failed")]
    DownloadFailed(#[source] anyhow::Error),
}

pub async fn query_metadata(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) -> Result<Option<(Repo, Sdp)>, MetadataQueryError> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_metadata(rapid_store, fullname).await,
        MetadataSource::FileApi => metadata_file::query_metadata(rapid_store, opts, fullname).await,
        MetadataSource::RestApi(api_server) => {
            metadata_rest::query_metadata(api_server, fullname).await
        }
    }
}

pub async fn query_repo(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo_basename: &str,
) -> Result<Option<Repo>, MetadataQueryError> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_repo(rapid_store, repo_basename).await,
        MetadataSource::FileApi => {
            metadata_file::query_repo(rapid_store, opts, repo_basename).await
        }
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
) -> Result<Option<Sdp>, MetadataQueryError> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_sdp(rapid_store, repo, tag).await,
        MetadataSource::FileApi => metadata_file::query_sdp(rapid_store, opts, repo, tag).await,
        MetadataSource::RestApi(api_server) => {
            metadata_rest::query_sdp(api_server, &format!("{}:{}", &repo.name, tag)).await
        }
    }
}

pub async fn query_sdp_files(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: &Repo,
    sdp: &Sdp,
) -> Result<Vec<SdpPackage>, MetadataQueryError> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_sdp_files(rapid_store, sdp).await,
        MetadataSource::FileApi => {
            metadata_file::query_sdp_files(rapid_store, opts, repo, sdp).await
        }
        MetadataSource::RestApi(_api_server) => {
            unimplemented!("Can't query SDP files from the Rest API at this moment.");
        }
    }
}

#[cfg(test)]

mod tests {

    use crate::api;

    use super::*;

    #[tokio::test]
    async fn test_query_metadata() {
        let rapid_store = RapidStore::new(test_utils::setup_sprd_folders().await);
        let (_, sdp) = query_metadata(
            &rapid_store,
            &api::DownloadOptions::default(),
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(sdp.md5, "d80d786597510d1358be3b04a7e9146e");
    }
}
