use crate::{
    file_download,
    rapid::{self, rapid_store::RapidStore, types::Repo},
};

use serde::{Deserialize, Serialize};

pub async fn download<'a>(rapid_store: &RapidStore<'_>, repo_tag: &str) {
    let repo_tag = repo_tag.split(':').collect::<Vec<&str>>();
    let repo_basename = repo_tag[0];
    let repo = query_repo(rapid_store, repo_basename).await.unwrap();

    let tag = repo_tag[1];

    // Load or download repo SDP
    file_download::download_repo(rapid_store, &repo)
        .await
        .expect("Failed to download repository.");
    let sdp = match rapid_store.find_sdp(&repo, tag) {
        Err(err) => {
            println!(
                "Failed to load sdp: (repo: {}) (tag: {}). Error: {}",
                repo.name, tag, err
            );
            return;
        }
        Ok(sdp_opt) => sdp_opt.unwrap(),
    };

    let dest_sdp = rapid_store.get_sdp_path(&sdp);
    // if !dest_sdp.exists() {
    match file_download::download_sdp(rapid_store, &repo, &sdp).await {
        Ok(_) => {}
        Err(err) => {
            println!("Failed to download SDP: {err}");
            return;
        }
    }
    // }

    assert!(dest_sdp.exists());

    let sdp_files = rapid::parsing::load_sdp_packages_from_file(&dest_sdp)
        .expect("Failed to load SDP Package from file");

    let download_map = rapid_store.get_nonexisting_files_download_map(&sdp_files);
    file_download::download_sdp_files(rapid_store, &repo, &sdp, download_map, &sdp_files)
        .await
        .expect("Failed to download SDP files");
}

async fn query_repo(rapid_store: &RapidStore<'_>, repo_basename: &str) -> Option<Repo> {
    if true {
        query_repo_through_registry(rapid_store, repo_basename).await
    } else {
        query_repo_with_api(repo_basename).await
    }
}

async fn query_repo_through_registry(
    rapid_store: &RapidStore<'_>,
    repo_basename: &str,
) -> Option<Repo> {
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
struct RepoAPI {
    pub id: i32,
    pub name: String,
    pub url: String,
}

async fn query_repo_with_api(repo_basename: &str) -> Option<Repo> {
    let resp = reqwest::get(format!("http://localhost:8080/repo/{repo_basename}"))
        .await
        .unwrap()
        .json::<RepoAPI>()
        .await
        .unwrap();

    Some(Repo {
        name: resp.name,
        url: resp.url,
    })
}

#[cfg(test)]

mod tests {
    use crate::util;

    use super::*;

    #[tokio::test]
    async fn test_query() {
        let rapid_store = rapid::rapid_store::RapidStore {
            root_folder: &util::default_spring_dir(),
        };
        let repo = query_repo(&rapid_store, "byar").await.unwrap();
        assert_eq!(repo.name, "byar");
        assert_eq!(repo.url, "https://repos.springrts.com/byar");
    }

    #[tokio::test]
    #[ignore] // Need to have the local server
    async fn test_query_with_api() {
        let repo = query_repo_with_api("byar").await.unwrap();
        assert_eq!(repo.name, "byar");
        assert_eq!(repo.url, "https://repos.springrts.com/byar");
    }
}
