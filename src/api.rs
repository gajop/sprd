pub mod metadata_query {
    // This seems silly?
    pub use super::super::metadata_query::*;
}

pub enum MetadataSource {
    // File(str),
    FileApi,
    RestApi(String),
}

pub struct DownloadOptions {
    pub metadata_source: MetadataSource,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        DownloadOptions {
            metadata_source: MetadataSource::FileApi,
        }
    }
}
