use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use sprd::{api, cmds, rapid};

// These tests require that you have pr-downloader installed and available in Path.

fn setup_prd_folders() {
    let path = Path::new("test_folders/test_prd/");
    if path.exists() {
        return;
    }

    println!("Setting up prd folders. This might take a while...");
    std::fs::create_dir_all(path).unwrap();
    let output = Command::new("pr-downloader")
        .arg("--filesystem-writepath")
        .arg("test_prd")
        .arg("--download-game")
        .arg("sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc")
        .output()
        .expect("Failed to execute command");

    println!("{output:?}");
}

#[tokio::test]
#[ignore] // Need to have pr-downloader installed
async fn test_file_api() {
    setup_prd_folders();

    let rapid_store = rapid::rapid_store::RapidStore {
        root_folder: PathBuf::from("test_folders/test_sprd"),
    };
    cmds::download(
        &rapid_store,
        &api::DownloadOptions::default(),
        "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
    )
    .await;
}

#[ignore]
#[tokio::test]
async fn test_folder_equality() {
    assert_files_equal(
        Path::new("test_folders/test_sprd/rapid/repos.springrts.com/repos.gz"),
        Path::new("test_folders/test_prd/rapid/repos.springrts.com/repos.gz"),
    );

    assert_files_equal(
        Path::new("test_folders/test_sprd/rapid/repos.springrts.com/sbc/versions.gz"),
        Path::new("test_folders/test_prd/rapid/repos.springrts.com/sbc/versions.gz"),
    );

    // fn main() -> io::Result<()> {
    //     let mut entries = fs::read_dir(".")?
    //         .map(|res| res.map(|e| e.path()))
    //         .collect::<Result<Vec<_>, io::Error>>()?;

    assert_file_identity(
        Path::new("test_folders/test_sprd/packages/"),
        Path::new("test_folders/test_prd/packages/"),
    );

    assert_file_identity(
        Path::new("test_folders/test_sprd/pool/"),
        Path::new("test_folders/test_prd/pool/"),
    );
    // for entry in  {
    //     let entry = entry.unwrap();
    //     let path = entry.path();
    //     println!("path: {path:?}");
    // }

    // assert_folders_equal(
    //     Path::new("test_folders/test_sprd"),
    //     Path::new("test_folders/test_prd"),
    // );
}

fn assert_files_equal(path1: &Path, path2: &Path) {
    let first_contents = fs::read(path1).unwrap();
    let second_contents = fs::read(path2).unwrap();

    assert_eq!(first_contents, second_contents);
}

fn assert_file_identity(path1: &Path, path2: &Path) {
    let files1: Vec<PathBuf> = fs::read_dir(path1)
        .unwrap()
        .map(|file| file.unwrap().path())
        .collect();
    let files2: Vec<PathBuf> = fs::read_dir(path2)
        .unwrap()
        .map(|file| file.unwrap().path())
        .collect();

    let mut names1 = files1
        .iter()
        .map(|p| p.file_name().unwrap().to_owned())
        .collect::<Vec<OsString>>();
    names1.sort();
    let mut names2: Vec<OsString> = files2
        .iter()
        .map(|p| p.file_name().unwrap().to_owned())
        .collect::<Vec<OsString>>();
    names2.sort();

    assert_eq!(names1, names2);
}
