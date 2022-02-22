use anyhow::Context;

use sprd::{
    api::DownloadOptions, metadata, pool_downloader, rapid::rapid_store::RapidStore,
    validation::check_if_sdp_needs_download,
};

pub async fn download(rapid_store: &RapidStore, opts: &DownloadOptions, fullname: &str) {
    if let Err(err) = download_with_err(rapid_store, opts, fullname).await {
        println!("Failed to download {fullname}. Error: {err:?}");
    }
}

async fn download_with_err(
    rapid_store: &RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) -> anyhow::Result<()> {
    let (repo, sdp) = metadata::query_metadata(rapid_store, opts, fullname)
        .await?
        .context("No such item")?;

    let sdp_files = metadata::query_sdp_files(rapid_store, opts, &repo, &sdp).await?;

    if !check_if_sdp_needs_download(rapid_store, &sdp.md5) {
        return Ok(());
    }

    let download_map = rapid_store.get_missing_files_indices(&sdp_files);
    pool_downloader::download_sdp_files(rapid_store, opts, &repo, &sdp, download_map, &sdp_files)
        .await?;

    Ok(())
}
