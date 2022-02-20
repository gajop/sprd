use thiserror::Error;

use crate::{
    file_download,
    rapid::{self, rapid_store::RapidStore},
};

#[derive(Error, Debug)]
enum Errors {
    #[error("no such repo")]
    NoSuchRepo,
}

pub async fn download_repo(rapid_store: &RapidStore, repo: Option<&str>) {
    match repo {
        Some(repo) => handle_errors(download_one_repo(rapid_store, repo).await),
        None => handle_errors(download_all_repos(rapid_store).await),
    };
}

fn handle_errors(result: Result<(), Box<dyn std::error::Error>>) {
    match result {
        Ok(()) => println!("Download success"),
        Err(err) => {
            println!("Failed to download repository: {err}");
        }
    }
}

async fn download_one_repo(
    rapid_store: &RapidStore,
    repo: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // println!("Failed to open repository registry: {err}.")
    let repo_registry = rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path())?;

    let repo = repo_registry
        .into_iter()
        .find(|r| r.name == repo)
        .ok_or_else(|| Box::new(Errors::NoSuchRepo))?;

    file_download::download_repo(rapid_store, &repo).await
}

async fn download_all_repos(rapid_store: &RapidStore) -> Result<(), Box<dyn std::error::Error>> {
    file_download::download_all_repos(rapid_store).await
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::rapid;

    #[tokio::test]
    async fn download_one_repo_with_clean_install() {
        let temp_dir = tempfile::tempdir().unwrap();
        let rapid_store = rapid::rapid_store::RapidStore::new(temp_dir.into_path());

        download_one_repo(&rapid_store, "byar").await.unwrap();
    }
}
