use std::{
    path::{Path, PathBuf},
    process::Command,
};

use sprd::{api, cmds, rapid::rapid_store};

// These tests require that you have pr-downloader installed and available in Path.

pub fn setup_pr_downloader_folders() -> PathBuf {
    let path = PathBuf::from("test_folders/test_prd/");
    if path.exists() {
        return path;
    }

    println!("Setting up prd folders. This might take a while...");
    std::fs::create_dir_all(&path).unwrap();

    let program = if Path::new("./pr-downloader").exists() {
        println!("Found local pr-downloader");
        "./pr-downloader"
    } else {
        println!("Couldn't find local pr-downloader. Falling back to system one - hopefully it's installed");
        "pr-downloader"
    };
    let output = Command::new(program)
        .arg("--filesystem-writepath")
        .arg("test_folders/test_prd")
        .arg("--download-game")
        .arg("sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc")
        .output()
        .expect("Failed to execute command");

    println!("{output:?}");

    path
}

pub async fn setup_sprd_folders() -> PathBuf {
    let root_folder = PathBuf::from("test_folders/test_sprd");
    let rapid_store = rapid_store::RapidStore::new(root_folder.clone());
    cmds::download(
        &rapid_store,
        &api::DownloadOptions::default(),
        "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
    )
    .await;

    root_folder
}
