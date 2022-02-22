use sprd::{
    api::DownloadOptions,
    event::Event,
    rapid,
    validation::{self, ValidityErrors},
};

pub async fn validate_by_fullname(
    rapid_store: &rapid::rapid_store::RapidStore,
    opts: &DownloadOptions,
    fullname: &str,
) {
    let results = validation::validate_by_fullname(rapid_store, opts, fullname).await;

    match results {
        Ok(()) => {
            opts.print
                .event(Event::Info(format!("Successfully verified {fullname}")));
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
            for (path, file_error) in files {
                opts.print
                    .event(Event::Error(format!("{path:?} is {file_error:?}")));
            }
        }
    }
}
