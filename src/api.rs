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
