use crate::api;
use crate::download;
use crate::rapid;
use crate::rapid::rapid_store::RapidStore;

pub fn check_sdp(rapid_store: &rapid::rapid_store::RapidStore, sdp_md5: &str) {
    if api::check_if_sdp_needs_download(&rapid_store, sdp_md5) {
        println!("Download necessary");
        std::process::exit(1);
    } else {
        println!("No download necessary");
        std::process::exit(0);
    }
}

pub async fn download<'a>(rapid_store: &RapidStore<'_>, tag: &str) {
    // if !rapid_store.get_registry_path().exists() {
    download::download_repo_registry(&rapid_store)
        .await
        .expect("Failed to download repository registry");
    // }

    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                println!("Failed to open repository registry: {}.", err.to_string());
                return;
            }
            Ok(repo_registry) => repo_registry,
        };

    let repo_basename = tag.split(":").collect::<Vec<&str>>()[0];
    let tag = tag.split(":").collect::<Vec<&str>>()[1];
    let repo = repo_registry
        .into_iter()
        .find(|r| r.name == repo_basename)
        .unwrap();

    // Load or download repo SDP
    download::download_repo(&rapid_store, &repo)
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
    match download::download_sdp(&rapid_store, &repo, &sdp).await {
        Ok(_) => {}
        Err(err) => {
            println!("Failed to download SDP: {}", err);
            return;
        }
    }
    // }

    assert!(dest_sdp.exists());

    let sdp_files = rapid::parsing::load_sdp_packages_from_file(&dest_sdp)
        .expect("Failed to load SDP Package from file");

    let download_map = rapid_store.get_nonexisting_files_download_map(&sdp_files);
    download::download_sdp_files(&rapid_store, &repo, &sdp, download_map, &sdp_files)
        .await
        .expect("Failed to download SDP files");
}

pub async fn download_sdp<'a>(rapid_store: &RapidStore<'_>, sdp_md5: &str) {
    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                println!("Failed to open repository registry: {}.", err.to_string());
                return;
            }
            Ok(repo_registry) => repo_registry,
        };

    let mut found_sdp: Option<rapid::types::SDP> = None;
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
            println!("No such sdp: {}", sdp_md5);
            return;
        }
    };

    match download::download_sdp(&rapid_store, &repo, &sdp).await {
        Ok(()) => {}
        Err(err) => println!("Failed to update registry: {}", err.to_string()),
    }
}

pub async fn download_registry<'a>(rapid_store: &RapidStore<'_>) {
    match download::download_repo_registry(&rapid_store).await {
        Ok(()) => {}
        Err(err) => println!("Failed to update registry: {}", err.to_string()),
    }
}

pub async fn download_repo<'a>(rapid_store: &RapidStore<'_>, repo: Option<&str>) {
    match repo {
        Some(repo) => download_one_repo(rapid_store, repo).await,
        None => download_all_repos(rapid_store).await,
    };
}

async fn download_one_repo<'a>(rapid_store: &RapidStore<'_>, repo: &str) {
    let repo_registry =
        match rapid::parsing::parse_repos_from_file(&rapid_store.get_registry_path()) {
            Err(err) => {
                println!("Failed to open repository registry: {}.", err.to_string());
                return;
            }
            Ok(repo_registry) => repo_registry,
        };

    let repo = match repo_registry.into_iter().find(|r| r.name == repo) {
        Some(repo) => repo,
        None => {
            println!("No such repository: {}", repo);
            return;
        }
    };

    match download::download_repo(&rapid_store, &repo).await {
        Ok(()) => println!("Download success"),
        Err(err) => {
            println!("Failed to download repository: {}", err.to_string());
            return;
        }
    }
}

async fn download_all_repos<'a>(rapid_store: &RapidStore<'_>) {
    match download::download_all_repos(&rapid_store).await {
        Ok(()) => {}
        Err(err) => {
            println!("Failed to download all repositories: {}", err.to_string());
            return;
        }
    }
}
