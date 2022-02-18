use std::error::Error;
use std::path::{self, PathBuf};

use super::super::util;
use super::parsing::read_rapid_from_file;
use super::types::{Repo, Sdp, SdpPackage};

pub struct RapidStore {
    pub root_folder: PathBuf,
}

impl Default for RapidStore {
    fn default() -> Self {
        RapidStore {
            root_folder: util::default_spring_dir(),
        }
    }
}

impl RapidStore {
    pub fn new(path: PathBuf) -> Self {
        Self { root_folder: path }
    }

    // pub fn find_repo(&self, name: &str) -> Result<Option<Repo>, Box<dyn Error>> {
    //     let repos = parse_repos_from_file(&self.get_repo_path(name))?;
    //     Ok(repos.into_iter().find(|repo| repo.name.contains(name)))
    // }

    pub fn find_sdp(&self, repo: &Repo, name: &str) -> Result<Option<Sdp>, Box<dyn Error>> {
        let repo_path = self.root_folder.join(&format!(
            "rapid/repos.springrts.com/{}/versions.gz",
            repo.name
        ));
        let sdps = read_rapid_from_file(&repo_path)?;
        Ok(sdps
            .into_iter()
            .find(|sdp| sdp.fullname == name || sdp.alias == name))
    }

    pub fn find_missing_files<'a>(&self, sdp_files: &'a [SdpPackage]) -> Vec<&'a SdpPackage> {
        sdp_files
            .iter()
            .filter(|sdp_file| !self.get_pool_path(sdp_file).exists())
            .collect::<Vec<&SdpPackage>>()
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
        self.root_folder.join(format!("rapid/{name}/versions.gz"))
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
            sdp_package.md5[0] as char,
            sdp_package.md5[1] as char,
            std::str::from_utf8(&sdp_package.md5[2..32]).unwrap()
        ));

        file_path
    }

    pub fn get_missing_files_indices(&self, sdp_files: &[SdpPackage]) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_find_sdp() {
        let rapid_store = RapidStore::default();
        let sdp = rapid_store
            .find_sdp(
                &Repo {
                    name: "sbc".to_owned(),
                    url: "-unused-".to_owned(),
                },
                "sbc:git:860aac5eb5ce292121b741ca8514516777ae14dc",
            )
            .unwrap()
            .unwrap();

        assert_eq!(sdp.md5, "d80d786597510d1358be3b04a7e9146e");
    }
}
