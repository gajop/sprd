use crate::{
    file_download,
    rapid::{
        rapid_store::RapidStore,
        types::{Repo, Sdp, SdpPackage},
    },
};

use super::{metadata_local, MetadataQueryError};

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
    metadata_local::query_sdp_files(rapid_store, sdp).await
}

pub async fn query_metadata(
    rapid_store: &RapidStore,
    fullname: &str,
) -> Result<Option<(Repo, Sdp)>, MetadataQueryError> {
    let repo_tag = fullname.split(':').collect::<Vec<&str>>();
    let repo_basename = repo_tag[0];

    let repo = match query_repo(rapid_store, repo_basename).await? {
        None => return Ok(None),
        Some(repo) => repo,
    };
    let sdp = match query_sdp(rapid_store, &repo, fullname).await? {
        None => return Ok(None),
        Some(sdp) => sdp,
    };
    Ok(Some((repo, sdp)))
}

pub async fn query_repo(
    rapid_store: &RapidStore,
    repo_basename: &str,
) -> Result<Option<Repo>, MetadataQueryError> {
    // if !rapid_store.get_registry_path().exists() {
    file_download::download_repo_registry(rapid_store)
        .await
        .map_err(|e| MetadataQueryError::DownloadFailed(e))?;
    // }

    metadata_local::query_repo(rapid_store, repo_basename).await
}

pub async fn query_sdp(
    rapid_store: &RapidStore,
    repo: &Repo,
    fullname: &str,
) -> Result<Option<Sdp>, MetadataQueryError> {
    file_download::download_repo(rapid_store, repo)
        .await
        .map_err(|e| MetadataQueryError::DownloadFailed(e))?;

    metadata_local::query_sdp(rapid_store, repo, fullname).await
}

#[cfg(test)]

mod tests {

    use crate::rapid;

    use super::*;

    #[tokio::test]
    async fn test_query_repo() {
        let rapid_store = rapid::rapid_store::RapidStore::default();

        let repo = query_repo(&rapid_store, "byar").await.unwrap().unwrap();
        assert_eq!(repo.name, "byar");
        assert_eq!(repo.url, "https://repos.springrts.com/byar");
    }

    #[tokio::test]
    async fn test_query_sdp() {
        let rapid_store = rapid::rapid_store::RapidStore::default();

        let repo = query_repo(&rapid_store, "byar").await.unwrap().unwrap();
        let sdp = query_sdp(&rapid_store, &repo, "test")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(sdp.fullname, "byar:test");
    }
}
