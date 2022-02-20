use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

#[tokio::test]
async fn test_folder_equality() {
    test_utils::setup_pr_downloader_folders();
    test_utils::setup_sprd_folders().await;

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
        true,
    );

    assert_file_identity(
        Path::new("test_folders/test_sprd/pool/"),
        Path::new("test_folders/test_prd/pool/"),
        true,
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

fn assert_file_identity(path1: &Path, path2: &Path, ignore_empty: bool) {
    let get_names = |path: &Path| {
        let files: Vec<PathBuf> = fs::read_dir(path)
            .unwrap()
            .map(|file| {
                // TODO: Support nested directories
                file.unwrap().path()
            })
            .flat_map(|path| {
                let metadata = fs::metadata(&path).unwrap();
                if metadata.is_dir() {
                    fs::read_dir(path)
                        .unwrap()
                        .map(|f| f.unwrap().path())
                        .collect::<Vec<PathBuf>>()
                } else {
                    vec![path]
                }
            })
            .filter(|file| {
                if !ignore_empty {
                    return true;
                }
                let metadata = fs::metadata(file).unwrap();
                if metadata.is_file() {
                    return metadata.len() > 0;
                }

                if metadata.is_dir() {
                    panic!("Shouldn't have directories at this point: {file:?}");
                } else {
                    panic!("Found unexpected file type: neither file nor directory: {file:?}");
                }
            })
            .collect();

        let mut names = files
            .iter()
            .map(|p| p.file_name().unwrap().to_owned())
            .collect::<Vec<OsString>>();

        names.sort();
        names
    };

    let names1 = get_names(path1);
    let names2 = get_names(path2);

    if names1 != names2 {
        for n1 in names1.iter() {
            if !names2.contains(n1) {
                println!("Second doesn't contain: {n1:?}");
            }
        }
        for n2 in names2.iter() {
            if !names1.contains(n2) {
                println!("First doesn't contain: {n2:?}");
            }
        }
    }
    assert_eq!(names1, names2);
}
