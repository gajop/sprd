use thiserror::Error;

use sprd::{
    api::DownloadOptions,
    event::Event,
    file_download,
    rapid::{self, rapid_store::RapidStore},
};

#[derive(Error, Debug)]
enum Errors {
    #[error("no such repo")]
    NoSuchRepo,
}

pub async fn meta_download_repo(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: Option<&str>,
) {
    match repo {
        Some(repo) => handle_errors(download_one_repo(rapid_store, opts, repo).await, opts),
        None => handle_errors(download_all_repos(rapid_store, opts).await, opts),
    };
}

fn handle_errors(result: anyhow::Result<()>, opts: &DownloadOptions) {
    match result {
        Ok(()) => opts
            .print
            .event(Event::Error("Download success".to_owned())),
        Err(err) => opts.print.event(Event::Error(format!(
            "Failed to download repository: {err}"
        ))),
    }
}

async fn download_one_repo(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    repo: &str,
) -> anyhow::Result<()> {
    let mut attempt = 0;
    let repo_registry = loop {
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                attempt += 1;
                if attempt >= 5 {
                    return Err(err.into());
                }
                file_download::download_repo_registry(rapid_store, opts).await?;
            }
            Ok(repos) => break repos,
        }
    };

    let repo = repo_registry
        .into_iter()
        .find(|r| r.name == repo)
        .ok_or_else(|| Box::new(Errors::NoSuchRepo))?;

    file_download::download_repo(rapid_store, opts, &repo)
        .await
        .map_err(|e| e.into())
}

async fn download_all_repos(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
) -> anyhow::Result<()> {
    file_download::download_all_repos(rapid_store, opts)
        .await
        .map_err(|e| e.into())
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::rapid;

    #[tokio::test]
    async fn download_one_repo_with_clean_install() {
        let temp_dir = tempfile::tempdir().unwrap();
        let rapid_store = rapid::rapid_store::RapidStore::new(temp_dir.into_path());

        download_one_repo(&rapid_store, &DownloadOptions::default(), "byar")
            .await
            .unwrap();
    }
}
