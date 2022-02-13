use crate::{
    file_download,
    rapid::{self, rapid_store::RapidStore},
};

pub async fn download_repo<'a>(rapid_store: &RapidStore, repo: Option<&str>) {
    match repo {
        Some(repo) => download_one_repo(rapid_store, repo).await,
        None => download_all_repos(rapid_store).await,
    };
}

async fn download_one_repo<'a>(rapid_store: &RapidStore, repo: &str) {
    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                println!("Failed to open repository registry: {err}.");
                return;
            }
            Ok(repo_registry) => repo_registry,
        };

    let repo = match repo_registry.into_iter().find(|r| r.name == repo) {
        Some(repo) => repo,
        None => {
            println!("No such repository: {repo}");
            return;
        }
    };

    match file_download::download_repo(rapid_store, &repo).await {
        Ok(()) => println!("Download success"),
        Err(err) => {
            println!("Failed to download repository: {err}");
        }
    }
}

async fn download_all_repos<'a>(rapid_store: &RapidStore) {
    match file_download::download_all_repos(rapid_store).await {
        Ok(()) => {}
        Err(err) => {
            println!("Failed to download all repositories: {err}");
        }
    }
}
