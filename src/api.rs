use super::rapid::{parsing, rapid_store};

pub fn check_if_sdp_needs_download(rapid_store: &rapid_store::RapidStore<'_>, md5: &str) -> bool {
    let sdp_path = rapid_store.get_sdp_path_from_md5(md5);

    if !sdp_path.exists() {
        return true;
    }

    if let Ok(sdp_packages) = parsing::load_sdp_packages_from_file(&sdp_path) {
        !rapid_store.find_nonexisting_files(sdp_packages).is_empty()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_file() {
        let springdir = tempfile::tempdir().unwrap();
        let rapid_store = rapid_store::RapidStore {
            root_folder: springdir.path(),
        };

        assert!(check_if_sdp_needs_download(&rapid_store, "test"));
        assert!(check_if_sdp_needs_download(&rapid_store, ""));
    }
}
