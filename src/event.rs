use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Info(String),
    Error(String),
    DownloadStarted(usize),
    DownloadProgress(usize),
    DownloadFinished,
    DownloadFailed,
}

pub trait Print {
    fn event(&self, event: Event);
}

pub struct PrintOutput {}

impl Print for PrintOutput {
    fn event(&self, event: Event) {
        println!("{event:?}")
    }
}

pub struct SilentOutput {}
impl Print for SilentOutput {
    fn event(&self, _event: Event) {}
}
