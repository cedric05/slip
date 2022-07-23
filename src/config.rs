use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};


#[derive(Deserialize, Debug)]
pub struct Config {
    pub default: Option<WorkOrPersonal>,
    work: Option<RepoRoot>,
    personal: Option<RepoRoot>,
}

#[derive(Deserialize, Debug, Serialize)]
pub enum WorkOrPersonal {
    Work,
    Personal,
}

impl Display for WorkOrPersonal{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkOrPersonal::Work => write!(f, "Work"),
            WorkOrPersonal::Personal => write!(f, "Personal"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepoRoot {
    pub root: Option<String>,
    pub pattern: Option<RepoNamePattern>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum RepoNamePattern {
    Hyphen,
    Recursive,
    Plain,
}

impl RepoNamePattern {
    pub fn get_directory(&self, path: String) -> String {
        match &self {
            RepoNamePattern::Recursive => path,
            RepoNamePattern::Hyphen => path.replace('/', "-"),
            RepoNamePattern::Plain => {
                let at = path.rfind('/').unwrap() + 1;
                path[at..path.len()].to_string()
            }
        }
    }
}

#[test]
fn teste2() {
    assert_eq!(
        "asdfadf".to_string(),
        RepoNamePattern::Plain.get_directory("asdf/asdfadf".to_string())
    );

    assert_eq!(
        "asdf/asdfadf".to_string(),
        RepoNamePattern::Recursive.get_directory("asdf/asdfadf".to_string())
    );

    assert_eq!(
        "asdf-asdfadf".to_string(),
        RepoNamePattern::Hyphen.get_directory("asdf/asdfadf".to_string())
    );
}

impl RepoRoot {
    fn get_pattern(&self) -> RepoNamePattern {
        *self.pattern.as_ref().unwrap_or(&RepoNamePattern::Recursive)
    }
}

impl Config {
    pub fn work(&self) -> (String, RepoNamePattern) {
        if self.work.is_some() && self.work.as_ref().unwrap().root.is_some() {
            (
                String::from(shellexpand::tilde(
                    self.work.as_ref().unwrap().root.as_ref().unwrap(),
                )),
                self.work.as_ref().unwrap().get_pattern(),
            )
        } else {
            (
                String::from(shellexpand::tilde("~/projects/work")),
                RepoNamePattern::Recursive,
            )
        }
    }

    pub fn personal(&self) -> (String, RepoNamePattern) {
        if self.personal.is_some() && self.personal.as_ref().unwrap().root.is_some() {
            (
                String::from(shellexpand::tilde(
                    self.personal.as_ref().unwrap().root.as_ref().unwrap(),
                )),
                self.personal.as_ref().unwrap().get_pattern(),
            )
        } else {
            (
                String::from(shellexpand::tilde("~/projects/personal")),
                RepoNamePattern::Recursive,
            )
        }
    }
    pub fn default(&self) -> (String, RepoNamePattern) {
        match self.default.as_ref().unwrap() {
            WorkOrPersonal::Personal => self.personal(),
            WorkOrPersonal::Work => self.work(),
        }
    }
    pub fn new() -> Config {
        Config {
            work: None,
            personal: None,
            default: Some(WorkOrPersonal::Work),
        }
    }
}
