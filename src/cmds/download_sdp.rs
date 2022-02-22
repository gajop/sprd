use crate::{
    api::DownloadOptions,
    event::Event,
    file_download,
    rapid::{self, rapid_store::RapidStore},
};

pub async fn download_sdp(rapid_store: &RapidStore, opts: &DownloadOptions, sdp_md5: &str) {
    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                opts.print.event(Event::Error(format!(
                    "Failed to open repository registry: {err}."
                )));
                return;
            }
            Ok(repo_registry) => repo_registry,
        };

    let mut found_sdp: Option<rapid::types::Sdp> = None;
    let mut found_repo: Option<rapid::types::Repo> = None;
    for repo in repo_registry {
        let sdps = match rapid::parsing::read_rapid_from_file(&rapid_store.get_repo_path(&repo)) {
            Ok(sdps) => sdps,
            Err(_) => {
                break;
            }
        };
        for sdp in sdps {
            if sdp.md5 == sdp_md5 {
                found_sdp = Some(sdp);
                break;
            }
        }
        if found_sdp.is_some() {
            found_repo = Some(repo);
            break;
        }
    }

    let (sdp, repo) = match (found_sdp, found_repo) {
        (Some(sdp), Some(repo)) => (sdp, repo),
        _ => {
            opts.print
                .event(Event::Error(format!("No such sdp: {sdp_md5}")));
            return;
        }
    };

    match file_download::download_sdp(rapid_store, opts, &repo, &sdp).await {
        Ok(()) => {}
        Err(err) => {
            opts.print
                .event(Event::Error(format!("Failed to update registry: {err}")));
        }
    }
}
