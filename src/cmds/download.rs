use crate::{
    download,
    rapid::{self, rapid_store::RapidStore, types::Repo},
};

pub async fn download<'a>(rapid_store: &RapidStore<'_>, repo_tag: &str) {
    let repo_tag = repo_tag.split(':').collect::<Vec<&str>>();
    let repo_basename = repo_tag[0];
    let repo = query_repo(rapid_store, repo_basename).await;

    let tag = repo_tag[1];

    // Load or download repo SDP
    download::download_repo(rapid_store, &repo)
        .await
        .expect("Failed to download repository.");
    let sdp = match rapid_store.find_sdp(&repo, tag) {
        Err(err) => {
            println!(
                "Failed to load sdp: (repo: {}) (tag: {}). Error: {}",
                repo.name, tag, err
            );
            return;
        }
        Ok(sdp_opt) => sdp_opt.unwrap(),
    };

    let dest_sdp = rapid_store.get_sdp_path(&sdp);
    // if !dest_sdp.exists() {
    match download::download_sdp(rapid_store, &repo, &sdp).await {
        Ok(_) => {}
        Err(err) => {
            println!("Failed to download SDP: {err}");
            return;
        }
    }
    // }

    assert!(dest_sdp.exists());

    let sdp_files = rapid::parsing::load_sdp_packages_from_file(&dest_sdp)
        .expect("Failed to load SDP Package from file");

    let download_map = rapid_store.get_nonexisting_files_download_map(&sdp_files);
    download::download_sdp_files(rapid_store, &repo, &sdp, download_map, &sdp_files)
        .await
        .expect("Failed to download SDP files");
}

async fn query_repo(rapid_store: &RapidStore<'_>, repo_basename: &str) -> Repo {
    // if !rapid_store.get_registry_path().exists() {
    download::download_repo_registry(rapid_store)
        .await
        .expect("Failed to download repository registry");
    // }

    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                panic!("Failed to open repository registry: {err}.");
            }
            Ok(repo_registry) => repo_registry,
        };

    repo_registry
        .into_iter()
        .find(|r| r.name == repo_basename)
        .unwrap()
}