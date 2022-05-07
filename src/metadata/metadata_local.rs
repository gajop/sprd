use crate::rapid::{
    self,
    rapid_store::RapidStore,
    types::{Repo, Sdp, SdpPackage},
};

use super::MetadataQueryError;

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

// TODO: Should I just move rapid_store API here?
// It is doing the same thing basically...

pub async fn query_repo(
    rapid_store: &RapidStore,
    repo_basename: &str,
) -> Result<Option<Repo>, MetadataQueryError> {
    let repo_registry = rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path())
        .map_err(|e| MetadataQueryError::CorruptFile(e.into()))?;

    Ok(repo_registry.into_iter().find(|r| r.name == repo_basename))
}

pub async fn query_sdp(
    rapid_store: &RapidStore,
    repo: &Repo,
    fullname: &str,
) -> Result<Option<Sdp>, MetadataQueryError> {
    rapid_store
        .find_sdp(repo, fullname)
        .map_err(|e| MetadataQueryError::CorruptFile(e.into()))
    // return match rapid_store.find_sdp(repo, fullname) {
    //     Err(err) => {
    //         println!(
    //             "Failed to load sdp: (repo: {}) (fullname: {}). Error: {}",
    //             repo.name, fullname, err
    //         );
    //         return None;
    //     }
    //     Ok(sdp) => sdp,
    // };
}

pub async fn query_sdp_files(
    rapid_store: &RapidStore,
    sdp: &Sdp,
) -> Result<Vec<SdpPackage>, MetadataQueryError> {
    let dest_sdp = rapid_store.get_sdp_path(sdp);
    assert!(dest_sdp.exists());

    rapid::parsing::load_sdp_packages_from_file(&dest_sdp)
        .map_err(|e| MetadataQueryError::CorruptFile(e.into()))
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
    async fn test_query_by_rapid_version() {
        let query_names = [
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
            "SpringBoard Core 0.5.2",
            "sbc:test",
            "sbc:stable",
        ];

        let rapid_store = RapidStore::new(test_utils::setup_pr_downloader_folders());

        for query_name in query_names {
            let result = query_metadata(&rapid_store, query_name).await;

            assert!(result.is_ok(), "Query is Err: {query_name}");
            assert!(result.unwrap().is_some(), "Query is None: {query_name}");
        }
    }

    #[tokio::test]
    async fn test_query_sdp_files() {
        let rapid_store = RapidStore::new(test_utils::setup_pr_downloader_folders());

        let (_, sdp) = query_metadata(
            &rapid_store,
            "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
        )
        .await
        .unwrap()
        .unwrap();

        let mut sprd_files = HashSet::new();

        let sdp_files = query_sdp_files(&rapid_store, &sdp).await.unwrap();
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

        let _rapid_store_sprd = test_utils::setup_sprd_folders().await;
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
