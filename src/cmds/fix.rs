use tokio::fs;

use crate::{
    api::{DownloadOptions, MetadataSource},
    rapid,
    validation::{self, FileError, ValidityErrors},
};

use super::download;

pub async fn fix(rapid_store: &rapid::rapid_store::RapidStore, fullname: &str) {
    for attempt in 0..5 {
        println!("Fix attempt {attempt}.");
        let success = fix_attempt(rapid_store, fullname).await;
        if success {
            println!("Success");
            return;
        }
    }

    println!("Failed to fix after five attempts.");
}

async fn fix_attempt(rapid_store: &rapid::rapid_store::RapidStore, fullname: &str) -> bool {
    let results = validation::validate_by_fullname(
        rapid_store,
        &DownloadOptions::new(MetadataSource::Local),
        fullname,
    )
    .await;

    match results {
        Ok(()) => {
            println!("Successfully verified {fullname}");
            return true;
        }
        Err(ValidityErrors::MissingSdp) => {
            println!("Sdp file is missing.");
        }
        Err(ValidityErrors::MetadataQueryError(error)) => {
            println!("Metadata query error: {error:?}");
        }
        Err(ValidityErrors::InvalidFiles { files }) => {
            let mut removed_files = 0;
            for (path, file_error) in files {
                match file_error {
                    FileError::Corrupt | FileError::WrongHash => {
                        if let Err(e) = fs::remove_file(&path).await {
                            println!("Error: {e:?} for {path:?}");
                        } else {
                            removed_files += 1;
                        }
                    }
                    _ => {}
                }
            }
            println!("Removed {removed_files} corrupt files");
        }
    }

    download::download(rapid_store, &DownloadOptions::default(), fullname).await;

    false
}
