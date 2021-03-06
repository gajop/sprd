use sprd::{api::DownloadOptions, event::Event, file_download, rapid::rapid_store::RapidStore};

pub async fn meta_download_registry(rapid_store: &RapidStore, opts: &DownloadOptions) {
    match file_download::download_repo_registry(rapid_store, opts).await {
        Ok(()) => {}
        Err(err) => {
            opts.print
                .event(Event::Error(format!("Failed to update registry: {err}")));
        }
    }
}
