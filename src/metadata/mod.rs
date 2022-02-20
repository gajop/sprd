use std::error::Error;
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
    CorruptFile(#[source] Box<dyn Error>),

    #[error("download failed")]
    DownloadFailed(#[source] Box<dyn Error>),
}

pub async fn query_metadata(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) -> Result<Option<(Repo, Sdp)>, MetadataQueryError> {
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
) -> Result<Option<Repo>, MetadataQueryError> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_repo(rapid_store, repo_basename).await,
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
) -> Result<Option<Sdp>, MetadataQueryError> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_sdp(rapid_store, repo, tag).await,
        MetadataSource::FileApi => metadata_file::query_sdp(rapid_store, repo, tag).await,
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
) -> Vec<SdpPackage> {
    match &opts.metadata_source {
        MetadataSource::Local => metadata_local::query_sdp_files(rapid_store, sdp).await,
        MetadataSource::FileApi => metadata_file::query_sdp_files(rapid_store, repo, sdp).await,
        MetadataSource::RestApi(_api_server) => {
            unimplemented!("Can't query SDP files from the Rest API at this moment.");
        }
    }
}

#[cfg(test)]

mod tests {

    use crate::{api, rapid};

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
        .await
        .unwrap()
        .unwrap();

        assert_eq!(sdp.md5, "d80d786597510d1358be3b04a7e9146e");
    }
}