use crate::{
    api::{check_if_sdp_needs_download, DownloadOptions},
    file_download, metadata_query,
    rapid::rapid_store::RapidStore,
};

pub async fn download<'a>(rapid_store: &RapidStore, opts: &DownloadOptions, fullname: &str) {
    let (repo, sdp) = metadata_query::query_metadata(rapid_store, opts, fullname).await;

    let sdp_files = metadata_query::query_sdp_files(rapid_store, &repo, &sdp).await;

    if !check_if_sdp_needs_download(rapid_store, &sdp.md5) {
        return;
    }

    let download_map = rapid_store.get_missing_files_indices(&sdp_files);
    file_download::download_sdp_files(rapid_store, &repo, &sdp, download_map, &sdp_files)
        .await
        .expect("Failed to download SDP files");
}
