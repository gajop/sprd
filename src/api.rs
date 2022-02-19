pub enum MetadataSource {
    Local,
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

impl DownloadOptions {
    pub fn new(metadata_source: MetadataSource) -> Self {
        Self { metadata_source }
    }
}
