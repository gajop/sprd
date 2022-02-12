use crate::{file_download, rapid::rapid_store::RapidStore};

pub async fn download_registry<'a>(rapid_store: &RapidStore<'_>) {
    match file_download::download_repo_registry(rapid_store).await {
        Ok(()) => {}
        Err(err) => println!("Failed to update registry: {err}"),
    }
}
