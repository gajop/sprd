use crate::{rapid, validation};

pub fn check_sdp(rapid_store: &rapid::rapid_store::RapidStore, sdp_md5: &str) {
    if validation::check_if_sdp_needs_download(rapid_store, sdp_md5) {
        println!("Download necessary");
        std::process::exit(1);
    } else {
        println!("No download necessary");
        std::process::exit(0);
    }
}
