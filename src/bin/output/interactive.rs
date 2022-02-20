use std::sync::{Arc, Mutex};

use indicatif::{ProgressBar, ProgressStyle};
use sprd::event::{Event, Print};

pub struct InteractiveOutput {
    inner: Arc<Mutex<Inner>>,
}

impl InteractiveOutput {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }
}

impl Print for InteractiveOutput {
    fn event(&self, event: Event) {
        match event {
            Event::DownloadStarted(total_size) => {
                let pb_template: String =
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
                            .to_owned();
                let pb = ProgressBar::new(total_size as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template(&pb_template)
                        .progress_chars("#>-"),
                );

                let mut inner = self.inner.lock().unwrap();
                inner.progress_bar = Some(pb);
            }
            Event::DownloadProgress(downloaded) => {
                let mut inner = self.inner.lock().unwrap();
                if let Some(pb) = &mut inner.progress_bar {
                    pb.set_position(downloaded as u64);
                }
            }
            Event::DownloadFinished => {
                let mut inner = self.inner.lock().unwrap();
                if let Some(pb) = &mut inner.progress_bar {
                    pb.finish_with_message("downloaded");
                }
            }
            _ => {
                println!("Event: {event:?}")
            }
        }
    }
}

struct Inner {
    progress_bar: Option<ProgressBar>,
}

impl Inner {
    pub fn new() -> Self {
        Self { progress_bar: None }
    }
}
