use serde_derive::{Deserialize, Serialize};
use slip_git::config::WorkOrPersonal;
use std::error::Error;
use std::fmt::Display;
use std::fs;

const DEFAULT_REPOS_CONFIG_LOCATION: &str = "~/.slip.repos.toml";

#[derive(Serialize, Deserialize)]
pub struct Repo {
    pub url: String,
    pub location: String,
    pub name: String,
    pub category: WorkOrPersonal,
}

impl Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} project: {}, Location: {}, url: {},",
            self.category, self.name, self.location, self.url
        )
    }
}

#[derive(Serialize, Deserialize)]

pub struct RepoList {
    pub repos: Vec<Repo>,
}

impl RepoList {
    pub fn get_config() -> Result<RepoList, Box<dyn Error>> {
        let cloned_repos_list_location = shellexpand::tilde(DEFAULT_REPOS_CONFIG_LOCATION);
        let repos: Self = fs::read_to_string(cloned_repos_list_location.as_ref())
            .map(|x| toml::from_str(&x))
            .unwrap_or_else(|_| Ok(RepoList { repos: vec![] }))?;
        Ok(repos)
    }

    pub fn save_config(self: &Self) {
        let cloned_repos_list_location = shellexpand::tilde(DEFAULT_REPOS_CONFIG_LOCATION);
        let dump = toml::to_vec(&self).unwrap();
        fs::write(cloned_repos_list_location.as_ref(), dump).unwrap();
    }
}
