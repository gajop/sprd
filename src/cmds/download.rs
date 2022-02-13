use crate::{
    file_download,
    rapid::{
        self,
        rapid_store::RapidStore,
        types::{Repo, Sdp, SdpPackage},
    },
};
use serde::{Deserialize, Serialize};

pub enum MetadataSource {
    // File(str),
    FileApi,
    RestApi(String),
}

pub struct DownloadOptions {
    pub metadata_source: MetadataSource,
}

pub async fn download<'a>(rapid_store: &RapidStore<'_>, opts: &DownloadOptions, fullname: &str) {
    let (repo, sdp) = match &opts.metadata_source {
        MetadataSource::FileApi => {
            let repo_tag = fullname.split(':').collect::<Vec<&str>>();
            let repo_basename = repo_tag[0];
            let tag = repo_tag[1];

            let repo = query_repo_with_file(rapid_store, repo_basename)
                .await
                .unwrap();
            let sdp = query_sdp_with_file(rapid_store, &repo, tag).await.unwrap();
            (repo, sdp)
        }
        MetadataSource::RestApi(api_server) => query_sdp_with_api(api_server, fullname).await,
    };

    let sdp_files = query_sdp_files(rapid_store, &repo, &sdp).await;

    let download_map = rapid_store.get_nonexisting_files_download_map(&sdp_files);
    file_download::download_sdp_files(rapid_store, &repo, &sdp, download_map, &sdp_files)
        .await
        .expect("Failed to download SDP files");
}

pub async fn query_repo(
    rapid_store: &RapidStore<'_>,
    opts: &DownloadOptions,
    repo_basename: &str,
) -> Option<Repo> {
    match &opts.metadata_source {
        MetadataSource::FileApi => query_repo_with_file(rapid_store, repo_basename).await,
        MetadataSource::RestApi(api_server) => query_repo_with_api(api_server, repo_basename).await,
    }
}

pub async fn query_sdp(
    rapid_store: &RapidStore<'_>,
    opts: &DownloadOptions,
    repo: &Repo,
    tag: &str,
) -> Option<Sdp> {
    match &opts.metadata_source {
        MetadataSource::FileApi => query_sdp_with_file(rapid_store, repo, tag).await,
        MetadataSource::RestApi(api_server) => Some(
            query_sdp_with_api(api_server, &format!("{}:{}", &repo.name, tag))
                .await
                .1,
        ),
    }
}

async fn query_repo_with_file(rapid_store: &RapidStore<'_>, repo_basename: &str) -> Option<Repo> {
    // if !rapid_store.get_registry_path().exists() {
    file_download::download_repo_registry(rapid_store)
        .await
        .expect("Failed to download repository registry");
    // }

    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                panic!("Failed to open repository registry: {err}.");
            }
            Ok(repo_registry) => repo_registry,
        };

    repo_registry.into_iter().find(|r| r.name == repo_basename)
}

#[derive(Debug, Serialize, Deserialize)]
struct RepoResponse {
    id: i32,
    name: String,
    url: String,
}

async fn query_repo_with_api(server: &str, repo_basename: &str) -> Option<Repo> {
    let resp = reqwest::get(format!("{server}/repo/{repo_basename}"))
        .await
        .unwrap()
        .json::<RepoResponse>()
        .await
        .unwrap();

    Some(Repo {
        name: resp.name,
        url: resp.url,
    })
}

async fn query_sdp_with_file(rapid_store: &RapidStore<'_>, repo: &Repo, tag: &str) -> Option<Sdp> {
    // Load or download repo SDP
    file_download::download_repo(rapid_store, repo)
        .await
        .expect("Failed to download repository.");
    let sdp = match rapid_store.find_sdp(repo, tag) {
        Err(err) => {
            println!(
                "Failed to load sdp: (repo: {}) (tag: {}). Error: {}",
                repo.name, tag, err
            );
            return None;
        }
        Ok(sdp_opt) => sdp_opt.unwrap(),
    };

    Some(sdp)
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

async fn query_sdp_with_api(server: &str, fullname: &str) -> (Repo, Sdp) {
    let resp = reqwest::get(format!("{server}/sdp/{fullname}"))
        .await
        .unwrap()
        .json::<SdpResponse>()
        .await
        .unwrap();

    let rapid = resp.rapid;
    let repo = resp.repo;
    (
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
    )
}

async fn query_sdp_files(rapid_store: &RapidStore<'_>, repo: &Repo, sdp: &Sdp) -> Vec<SdpPackage> {
    // if !dest_sdp.exists() {
    match file_download::download_sdp(rapid_store, repo, sdp).await {
        Ok(_) => {}
        Err(err) => {
            panic!("Failed to download SDP: {err}");
        }
    }
    // }
    let dest_sdp = rapid_store.get_sdp_path(sdp);
    assert!(dest_sdp.exists());

    rapid::parsing::load_sdp_packages_from_file(&dest_sdp)
        .expect("Failed to load SDP Package from file")
}

#[cfg(test)]

mod tests {
    const LOCAL_API_SERVER: &str = "http://localhost:8080";

    use crate::util;

    use super::*;

    #[tokio::test]
    async fn test_query_repo_with_file() {
        let rapid_store = rapid::rapid_store::RapidStore {
            root_folder: &util::default_spring_dir(),
        };
        let repo = query_repo_with_file(&rapid_store, "byar").await.unwrap();
        assert_eq!(repo.name, "byar");
        assert_eq!(repo.url, "https://repos.springrts.com/byar");
    }

    #[tokio::test]
    #[ignore] // Need to have the local server
    async fn test_query_repo_with_api() {
        let repo = query_repo_with_api(LOCAL_API_SERVER, "byar").await.unwrap();
        assert_eq!(repo.name, "byar");
        assert_eq!(repo.url, "https://repos.springrts.com/byar");
    }

    #[tokio::test]
    async fn test_query_sdp_with_file() {
        let rapid_store = rapid::rapid_store::RapidStore {
            root_folder: &util::default_spring_dir(),
        };
        let repo = query_repo_with_file(&rapid_store, "byar").await.unwrap();
        let sdp = query_sdp_with_file(&rapid_store, &repo, "test")
            .await
            .unwrap();
        assert_eq!(sdp.fullname, "byar:test");
    }

    #[tokio::test]
    #[ignore] // Need to have the local server
    async fn test_query_sdp_with_api() {
        let sdp = query_sdp_with_api(LOCAL_API_SERVER, "sbc:test").await.1;
        assert_eq!(sdp.fullname, "sbc:test");

        let rapid_store = rapid::rapid_store::RapidStore {
            root_folder: &util::default_spring_dir(),
        };
        let repo = query_repo_with_file(&rapid_store, "sbc").await.unwrap();
        let sdp_classic = query_sdp_with_file(&rapid_store, &repo, "test")
            .await
            .unwrap();

        assert_eq!(sdp.md5, sdp_classic.md5);
        // assert_eq!(sdp, sdp_classic);
    }
}
