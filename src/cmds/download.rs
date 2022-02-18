use crate::{
    api::DownloadOptions, file_download, metadata, rapid::rapid_store::RapidStore,
    validation::check_if_sdp_needs_download,
};

pub async fn download<'a>(rapid_store: &RapidStore, opts: &DownloadOptions, fullname: &str) {
    let (repo, sdp) = metadata::query_metadata(rapid_store, opts, fullname).await;

    let sdp_files = metadata::query_sdp_files(rapid_store, opts, &repo, &sdp).await;

    if !check_if_sdp_needs_download(rapid_store, &sdp.md5) {
        return;
    }

    let download_map = rapid_store.get_missing_files_indices(&sdp_files);
    file_download::download_sdp_files(rapid_store, &repo, &sdp, download_map, &sdp_files)
        .await
        .expect("Failed to download SDP files");
}
