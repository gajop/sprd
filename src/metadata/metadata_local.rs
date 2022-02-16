use std::error::Error;

use thiserror::Error;

use crate::rapid::{
    self,
    rapid_store::RapidStore,
    types::{Repo, Sdp, SdpPackage},
};

pub async fn query_metadata(rapid_store: &RapidStore, fullname: &str) -> (Repo, Sdp) {
    let repo_tag = fullname.split(':').collect::<Vec<&str>>();
    let repo_basename = repo_tag[0];

    let repo = query_repo(rapid_store, repo_basename)
        .await
        .unwrap()
        .unwrap();
    let sdp = query_sdp(rapid_store, &repo, fullname).await.unwrap();
    (repo, sdp)
}

#[derive(Error, Debug)]
pub enum QueryRepoError {
    #[error("failed to open repository registry")]
    FailedToOpenRepositoryRegistry(#[from] Box<dyn Error>),
}

pub async fn query_repo(
    rapid_store: &RapidStore,
    repo_basename: &str,
) -> Result<Option<Repo>, QueryRepoError> {
    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                return Err(QueryRepoError::FailedToOpenRepositoryRegistry(err));
            }
            Ok(repo_registry) => repo_registry,
        };

    Ok(repo_registry.into_iter().find(|r| r.name == repo_basename))
}

pub async fn query_sdp(rapid_store: &RapidStore, repo: &Repo, fullname: &str) -> Option<Sdp> {
    let sdp = match rapid_store.find_sdp(repo, fullname) {
        Err(err) => {
            println!(
                "Failed to load sdp: (repo: {}) (fullname: {}). Error: {}",
                repo.name, fullname, err
            );
            return None;
        }
        Ok(sdp) => return sdp,
    };
}

pub async fn query_sdp_files(rapid_store: &RapidStore, sdp: &Sdp) -> Vec<SdpPackage> {
    let dest_sdp = rapid_store.get_sdp_path(sdp);
    assert!(dest_sdp.exists());

    rapid::parsing::load_sdp_packages_from_file(&dest_sdp)
        .expect("Failed to load SDP Package from file")
}

#[cfg(test)]

mod tests {

    use std::{
        collections::HashSet,
        fs,
        path::{Path, PathBuf},
    };

    use super::*;

    #[tokio::test]
    async fn test_query_sdp_files() {
        let rapid_store = rapid::rapid_store::RapidStore::default();
        let (_, sdp) = query_metadata(
            &rapid_store,
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
        )
        .await;

        let mut sprd_files = HashSet::new();

        let sdp_files = query_sdp_files(&rapid_store, &sdp).await;
        for sdp_file in sdp_files.iter() {
            let dest = rapid_store.get_pool_path(sdp_file);
            sprd_files.insert(format!(
                "{}{}",
                dest.parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                dest.file_name().unwrap().to_str().unwrap()
            ));
        }

        let folders = list_files(Path::new("test_folders/test_sprd/pool/"));
        let prd_files: HashSet<String> = folders
            .iter()
            .map(|dir| list_files(dir))
            .flatten()
            .map(|f| {
                format!(
                    "{}{}",
                    f.parent().unwrap().file_name().unwrap().to_str().unwrap(),
                    f.file_name().unwrap().to_str().unwrap()
                )
            })
            .collect();

        let mut missing_sprd = 0;
        let mut missing_prd = 0;
        for sprd in sprd_files.iter() {
            if !prd_files.contains(sprd) {
                missing_prd += 1;
                println!("Extra: {}", sprd);
            }
        }
        for prd in prd_files.iter() {
            if !sprd_files.contains(prd) {
                missing_sprd += 1;
                println!("Missing: {}", prd);
            }
        }
        assert!(missing_prd == 0 && missing_sprd == 0);
    }

    fn list_files(path: &Path) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = fs::read_dir(path)
            .unwrap()
            .map(|file| file.unwrap().path())
            .collect();
        files.sort();
        files
    }
}
