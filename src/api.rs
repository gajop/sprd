use std::sync::Arc;

use crate::event::{Print, PrintOutput};

pub enum MetadataSource {
    Local,
    FileApi,
    RestApi(String),
}

pub struct DownloadOptions {
    pub metadata_source: MetadataSource,
    pub print: Arc<Box<dyn Print>>,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        DownloadOptions {
            metadata_source: MetadataSource::FileApi,
            print: Arc::new(Box::new(PrintOutput {})),
        }
    }
}

impl DownloadOptions {
    pub fn new(metadata_source: MetadataSource) -> Self {
        Self {
            metadata_source,
            ..Default::default()
        }
    }
}
