use crate::rapid::types::{Repo, Sdp};
use serde::{Deserialize, Serialize};

use super::MetadataQueryError;

#[derive(Debug, Serialize, Deserialize)]
struct RepoResponse {
    id: i32,
    name: String,
    url: String,
}

pub async fn query_repo(
    server: &str,
    repo_basename: &str,
) -> Result<Option<Repo>, MetadataQueryError> {
    let resp = reqwest::get(format!("{server}/repo/{repo_basename}"))
        .await
        .map_err(|e| MetadataQueryError::DownloadFailed(Box::new(e)))?
        .json::<RepoResponse>()
        .await
        .map_err(|e| MetadataQueryError::DownloadFailed(Box::new(e)))?;

    Ok(Some(Repo {
        name: resp.name,
        url: resp.url,
    }))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RapidResponse {
    id: i32,
    repo_id: i32,
    fullname: String,
    hash: String,
    something: String,
    alias: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SdpResponse {
    rapid: RapidResponse,
    repo: RepoResponse,
}

pub async fn query_metadata(
    server: &str,
    fullname: &str,
) -> Result<Option<(Repo, Sdp)>, MetadataQueryError> {
    let resp = reqwest::get(format!("{server}/sdp/{fullname}"))
        .await
        .map_err(|e| MetadataQueryError::DownloadFailed(Box::new(e)))?
        .json::<SdpResponse>()
        .await
        .map_err(|e| MetadataQueryError::DownloadFailed(Box::new(e)))?;

    let rapid = resp.rapid;
    let repo = resp.repo;
    Ok(Some((
        Repo {
            name: repo.name,
            url: repo.url,
        },
        Sdp {
            fullname: rapid.fullname,
            something: rapid.something,
            md5: rapid.hash,
            alias: rapid.alias,
        },
    )))
}

pub async fn query_sdp(server: &str, fullname: &str) -> Result<Option<Sdp>, MetadataQueryError> {
    if let Some(metadata) = query_metadata(server, fullname).await? {
        Ok(Some(metadata.1))
    } else {
        Ok(None)
    }
}

#[cfg(test)]

mod tests {
    const LOCAL_API_SERVER: &str = "http://localhost:8080";

    use crate::{metadata::metadata_file, rapid};

    use super::*;

    #[tokio::test]
    #[ignore] // Need to have the local server
    async fn test_query_repo() {
        let repo = query_repo(LOCAL_API_SERVER, "byar").await.unwrap().unwrap();
        assert_eq!(repo.name, "byar");
        assert_eq!(repo.url, "https://repos.springrts.com/byar");

        let rapid_store = rapid::rapid_store::RapidStore::default();

        let repo_with_file = metadata_file::query_repo(&rapid_store, "byar")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(repo, repo_with_file);
    }

    #[tokio::test]
    #[ignore] // Need to have the local server
    async fn test_query_sdp() {
        let sdp = query_sdp(LOCAL_API_SERVER, "sbc:test")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(sdp.fullname, "sbc:test");

        let rapid_store = rapid::rapid_store::RapidStore::default();

        let repo = metadata_file::query_repo(&rapid_store, "sbc")
            .await
            .unwrap()
            .unwrap();
        let sdp_file = metadata_file::query_sdp(&rapid_store, &repo, "test")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(sdp, sdp_file);
    }

    #[tokio::test]
    #[ignore] // Need to have the local server
    async fn test_query_metadata() {
        let (_, sdp) = query_metadata(
            LOCAL_API_SERVER,
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(sdp.md5, "d80d786597510d1358be3b04a7e9146e");
    }
}
