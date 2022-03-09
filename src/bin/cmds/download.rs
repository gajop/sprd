use sprd::{api::DownloadOptions, rapid::rapid_store::RapidStore, rapid_download};

pub async fn download(rapid_store: &RapidStore, opts: &DownloadOptions, fullname: &str) {
    if let Err(err) = rapid_download::download(rapid_store, opts, fullname).await {
        println!("Failed to download {fullname}. Error: {err:?}");
    }
}
