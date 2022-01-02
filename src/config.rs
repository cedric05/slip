use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub default: Option<WorkOrPersonal>,
    work: Option<RepoRoot>,
    personal: Option<RepoRoot>,
}

#[derive(Deserialize, Debug)]
pub enum WorkOrPersonal {
    Work,
    Personal,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepoRoot {
    root: Option<String>,
    // owner: Option<bool>,
}

impl Config {
    pub fn work(&self) -> String {
        if self.work.is_some() && self.work.as_ref().unwrap().root.is_some() {
            String::from(shellexpand::tilde(
                self.work.as_ref().unwrap().root.as_ref().unwrap(),
            ))
        } else {
            String::from(shellexpand::tilde("~/projects/work"))
        }
    }

    pub fn personal(&self) -> String {
        if self.personal.is_some() && self.personal.as_ref().unwrap().root.is_some() {
            String::from(shellexpand::tilde(
                self.personal.as_ref().unwrap().root.as_ref().unwrap(),
            ))
        } else {
            String::from(shellexpand::tilde("~/projects/personal"))
        }
    }
    pub fn default(&self) -> String {
        match self.default.as_ref().unwrap() {
            WorkOrPersonal::Personal => self.personal(),
            WorkOrPersonal::Work => self.work(),
        }
    }

    pub fn new() -> Self {
        Self {
            work: None,
            personal: None,
            default: Some(WorkOrPersonal::Work),
        }
    }
}
