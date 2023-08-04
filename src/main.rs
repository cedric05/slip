#![feature(is_sorted)]
use std::error::Error;
use std::fs;
use std::path::Path;

use clap::StructOpt;
use slip_git::args::{Args, SubCommands};
use slip_git::config::{Config, WorkOrPersonal};
use slip_git::execute;
use slip_git::GitConfig;
use url::Url;

mod repolist;
mod tui;

use repolist::*;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();
    let command = match cli.command {
        Some(ref command) => command,
        None => &SubCommands::Ui,
    };
    match command {
        SubCommands::Ui => {
            let category_filter = if cli.personal {
                Some(WorkOrPersonal::Personal)
            } else if cli.work {
                Some(WorkOrPersonal::Work)
            } else {
                None
            };
            let launch = tui::main(category_filter)?;
            if let Some(launch) = launch {
                match launch.launch_type {
                    tui::LaunchType::LaunchShell => {
                        let shell = if cfg!(target_os = "windows") {
                            "cmd".to_string()
                        } else {
                            std::env::var("SHELL").expect("shell vairable not set")
                        };
                        execute(shell, Some(&launch.directory))?;
                    }
                    tui::LaunchType::LaunchCode => {
                        execute(format!("code {}", launch.directory), None)?;
                    }
                };
            }
        }
        SubCommands::List { filter } => {
            let filter = match filter {
                Some(filter) => filter,
                None => "",
            };
            let repos_list = RepoList::get_config()?;
            repos_list
                .repos
                .iter()
                .filter(|repo| {
                    repo.name.contains(&filter)
                        || repo.location.contains(&filter)
                        || repo.url.contains(&filter)
                })
                .for_each(|repo| println!("{}", repo))
        }
        SubCommands::Clone { url, dir } => {
            let config = get_config(&cli);
            let category = get_profile(&cli, &config);
            let (reporoot, pattern) = match category {
                WorkOrPersonal::Work => config.work(),
                WorkOrPersonal::Personal => config.personal(),
            };
            let reporoot = Path::new(&reporoot);
            if !reporoot.exists() {
                fs::create_dir_all(reporoot).expect("not able to create directory")
            }
            let directory_to_clone = if let Some(dir) = dir {
                Path::new(reporoot).join(dir)
            } else {
                /*
                 *
                 * Its created for my case. may not work for all
                 * where git clone github.com/adsfdsafa
                 * here, username is username of user running
                 */
                // https://github.com/gitignore/gitignore
                // use pattern to better create repos
                let strip_hostname = if url.starts_with("https://") {
                    let from = String::from(Url::parse(url)?.path());
                    String::from(from.split_at(1).1)
                } else if url.starts_with("git@") {
                    // git@github.com:gitignore/gitignore
                    url[(url.rfind(':').unwrap() + 1)..(url.len())].to_string()
                } else {
                    "".to_owned()
                };
                Path::new(reporoot).join(pattern.get_directory(strip_hostname))
            };
            let clone_command_str = format!(
                "git clone {} {}",
                url,
                directory_to_clone
                    .to_str()
                    .expect("this should not error out")
            );
            match execute(clone_command_str, None) {
                Ok(exit_code) => {
                    if exit_code.success() {
                        add_to_slip_repo_list(url, directory_to_clone, category, config)?;
                    }
                }
                Err(_) => println!("child command ran into error"),
            };
        }
        SubCommands::Reconfig => {
            let config = cli.config();
            let config: Config = fs::read_to_string(config)
                .map(|x| toml::from_str(&x))
                .unwrap_or_else(|_| Ok(Config::new()))
                .unwrap();
            let repos_list = RepoList::get_config()?;
            println!("config is {config:?}");
            for repo in &repos_list.repos {
                configure_git(repo, &config)?;
            }
        }
        SubCommands::New { repo } => {
            let config = get_config(&cli);
            let category = get_profile(&cli, &config);
            let (reporoot, _) = match category {
                WorkOrPersonal::Work => config.work(),
                WorkOrPersonal::Personal => config.personal(),
            };
            let reporoot = Path::new(&reporoot).join(&repo);
            if !reporoot.exists() {
                fs::create_dir_all(&reporoot).expect("not able to create directory")
            }
            let init_command_str = format!("git init");
            let location = reporoot.to_str().unwrap().to_owned();
            execute(init_command_str, Some(&location))?;
            let mut repos_list = RepoList::get_config()?;
            let repo = Repo {
                url: "".to_string(),
                location,
                name: repo.to_owned(),
                category,
            };
            configure_git(&repo, &config)?;
            repos_list.repos.push(repo);
            repos_list.save_config();
        }
        SubCommands::Add { repo } => {
            let config = get_config(&cli);
            let category = get_profile(&cli, &config);
            add_to_slip_repo_list("", repo.into(), category, config)?;
        }
    };
    Ok(())
}

fn add_to_slip_repo_list(
    url: &str,
    directory_to_clone: std::path::PathBuf,
    category: WorkOrPersonal,
    config: Config,
) -> Result<(), Box<dyn Error>> {
    let mut repos_list = RepoList::get_config()?;
    let repo = Repo {
        url: url.to_string(),
        location: directory_to_clone.to_string_lossy().into_owned(),
        name: directory_to_clone
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        category,
    };
    configure_git(&repo, &config)?;
    repos_list.repos.push(repo);
    repos_list.save_config();
    Ok(())
}

fn get_config(cli: &Args) -> Config {
    let config = cli.config();
    let config: Config = fs::read_to_string(config)
        .map(|x| toml::from_str(&x))
        .unwrap_or_else(|_| Ok(Config::new()))
        .unwrap();
    config
}

fn get_profile(cli: &Args, config: &Config) -> WorkOrPersonal {
    let category = if cli.personal {
        WorkOrPersonal::Personal
    } else if cli.work {
        WorkOrPersonal::Work
    } else if config.default.is_some() {
        config.default.unwrap()
    } else {
        WorkOrPersonal::Personal
    };
    category
}

fn configure_git(repo: &Repo, config: &Config) -> Result<(), Box<dyn Error>> {
    println!(
        "configuring git for is {location} with {category}",
        location = repo.location,
        category = repo.category
    );
    let git_config: Option<GitConfig> = config.get_git_config(&repo.category);
    Ok(if let Some(GitConfig { email, name }) = git_config {
        if let Some(email) = email {
            let git_email_command = format!("git config user.email {email}");
            execute(git_email_command, Some(repo.location.as_ref()))?;
        }
        if let Some(name) = name {
            let git_name_command = format!("git config user.name {name}");
            execute(git_name_command, Some(repo.location.as_ref()))?;
        }
    })
}
