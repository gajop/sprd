use crate::{
    api::{DownloadOptions, MetadataSource},
    rapid,
    validation::{self, ValidityErrors},
};

pub async fn validate_by_fullname(rapid_store: &rapid::rapid_store::RapidStore, fullname: &str) {
    let results = validation::validate_by_fullname(
        rapid_store,
        &DownloadOptions::new(MetadataSource::Local),
        fullname,
    )
    .await;

    match results {
        Ok(()) => {
            println!("Successfully verified {fullname}");
        }
        Err(ValidityErrors::MissingSdp) => {
            println!("Sdp file is missing.");
        }
        Err(ValidityErrors::MetadataQueryError(error)) => {
            println!("Metadata query error: {error:?}");
        }
        Err(ValidityErrors::InvalidFiles { files }) => {
            for (path, file_error) in files {
                println!("{path:?} is {file_error:?}");
            }
        }
    }
}
