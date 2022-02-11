use std::error::Error;
use std::path;

use super::parsing::read_rapid_from_file;
use super::types::{Repo, Sdp, SdpPackage};

pub struct RapidStore<'a> {
    pub root_folder: &'a path::Path,
}

impl<'a> RapidStore<'a> {
    // pub fn find_repo(&self, name: &str) -> Result<Option<Repo>, Box<dyn Error>> {
    //     let repos = parse_repos_from_file(&self.get_repo_path(name))?;
    //     Ok(repos.into_iter().find(|repo| repo.name.contains(name)))
    // }

    pub fn find_sdp(&self, repo: &Repo, name: &str) -> Result<Option<Sdp>, Box<dyn Error>> {
        let repo_path = self.root_folder.join(&format!(
            "rapid/repos.springrts.com/{}/version.gz",
            repo.name
        ));
        let sdps = read_rapid_from_file(&repo_path)?;
        Ok(sdps
            .into_iter()
            .find(|sdp| sdp.fullname.contains(name) || sdp.alias.contains(name)))
    }

    pub fn find_nonexisting_files(&self, sdp_files: Vec<SdpPackage>) -> Vec<SdpPackage> {
        sdp_files
            .into_iter()
            .filter(|sdp_file| !self.get_pool_path(sdp_file).exists())
            .collect()
    }

    pub fn get_registry_path(&self) -> path::PathBuf {
        self.root_folder.join("rapid/repos.springrts.com/repos.gz")
    }

    pub fn get_repo_path(&self, repo: &Repo) -> path::PathBuf {
        let mut http_split: Vec<&str> = repo.url.split("http://").collect();
        if http_split.len() != 2 {
            http_split = repo.url.split("https://").collect();
        }
        let name = http_split[1];
        self.root_folder.join(format!("rapid/{name}/version.gz"))
    }

    pub fn get_sdp_path(&self, sdp: &Sdp) -> path::PathBuf {
        self.get_sdp_path_from_md5(&sdp.md5)
    }

    pub fn get_sdp_path_from_md5(&self, sdp_md5: &str) -> path::PathBuf {
        self.root_folder
            .join(path::PathBuf::from(format!("packages/{sdp_md5}.sdp")))
    }

    pub fn get_pool_path(&self, sdp_package: &SdpPackage) -> path::PathBuf {
        let file_path = self.root_folder.join(format!(
            "pool/{}{}/{}.gz",
            sdp_package.md5[0],
            sdp_package.md5[1],
            &sdp_package.md5[2..32].iter().collect::<String>()
        ));

        file_path
    }

    pub fn get_nonexisting_files_download_map(&self, sdp_files: &[SdpPackage]) -> Vec<u8> {
        let map_length = sdp_files.len() / 8 + 1;
        let mut download_map: Vec<u8> = vec![0; map_length];

        for (i, sdp_file) in sdp_files.iter().enumerate() {
            let file_path = self.get_pool_path(sdp_file);

            if !file_path.exists() {
                download_map[i / 8] |= 1 << (i % 8);
            }
        }

        download_map
    }
}
