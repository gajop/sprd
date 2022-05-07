use tokio::fs;

use sprd::{
    api::DownloadOptions,
    event::Event,
    rapid,
    validation::{self, FileError, ValidityErrors},
};

use super::download;

pub async fn fix(
    rapid_store: &rapid::rapid_store::RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) {
    for attempt in 0..5 {
        opts.print
            .event(Event::Info(format!("Fix attempt {attempt}.")));
        let success = fix_attempt(rapid_store, opts, fullname).await;
        if success {
            opts.print.event(Event::Info("Success".to_owned()));
            return;
        }
    }

    opts.print.event(Event::Error(
        "Failed to fix after five attempts.".to_owned(),
    ));
}

async fn fix_attempt(
    rapid_store: &rapid::rapid_store::RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) -> bool {
    let results = validation::validate_by_fullname(rapid_store, opts, fullname).await;

    match results {
        Ok(()) => {
            opts.print
                .event(Event::Info(format!("Successfully verified: {fullname}")));
            // println!("Successfully verified {fullname}");
            return true;
        }
        Err(ValidityErrors::MissingSdp) => {
            opts.print
                .event(Event::Error("Sdp file is missing.".to_owned()));
        }
        Err(ValidityErrors::MetadataQueryError(error)) => {
            opts.print
                .event(Event::Error(format!("Metadata query error: {error:?}")));
        }
        Err(ValidityErrors::InvalidFiles { files }) => {
            let mut removed_files = 0;
            for (path, file_error) in files {
                match file_error {
                    FileError::Corrupt | FileError::WrongHash => {
                        if let Err(e) = fs::remove_file(&path).await {
                            opts.print
                                .event(Event::Error(format!("Error: {e:?} for {path:?}")));
                        } else {
                            removed_files += 1;
                        }
                    }
                    _ => {}
                }
            }
            opts.print.event(Event::Info(format!(
                "Removed {removed_files} corrupt files"
            )));
        }
    }

    download(rapid_store, opts, fullname).await;

    false
}
