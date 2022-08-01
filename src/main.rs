use std::error::Error;
use std::fs;
use std::path::Path;

use clap::StructOpt;
use slip_git::args::{Args, Commands};
use slip_git::config::{Config, WorkOrPersonal};
use slip_git::execute;
use slip_git::GitConfig;
use url::Url;

mod repolist;
mod tui;

use repolist::*;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    match &cli.command {
        Commands::Ui => {
            let category_filter = if cli.personal {
                Some(WorkOrPersonal::Personal)
            } else if cli.work {
                Some(WorkOrPersonal::Work)
            } else {
                None
            };
            tui::main(category_filter)?;
        }
        Commands::List => {
            let repos_list = RepoList::get_config()?;
            repos_list
                .repos
                .iter()
                .for_each(|repo| println!("{}", repo))
        }
        Commands::Clone { url, dir } => {
            let config = cli.config();
            let config: Config = fs::read_to_string(config)
                .map(|x| toml::from_str(&x))
                .unwrap_or_else(|_| Ok(Config::new()))
                .unwrap();
            let (reporoot, pattern) = if cli.personal {
                config.personal()
            } else if cli.work {
                config.work()
            } else {
                config.default()
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
            let spawn = execute(clone_command_str, None);
            if let Ok(mut child) = spawn {
                let status = child.wait().expect("child command ran into error");
                if status.success() {
                    let mut repos_list = RepoList::get_config()?;
                    let repo = Repo {
                        url: url.to_string(),
                        location: directory_to_clone.to_string_lossy().into_owned(),
                        name: directory_to_clone
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        category: {
                            if cli.work {
                                WorkOrPersonal::Work
                            } else {
                                WorkOrPersonal::Personal
                            }
                        },
                    };
                    configure_git(&repo, &config)?;
                    repos_list.repos.push(repo);
                    repos_list.save_config();
                }
            } else {
                println!("child command ran into error");
            };
        }
        Commands::Reconfig => {
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
    };
    Ok(())
}

fn configure_git(repo: &Repo, config: &Config) -> Result<(), Box<dyn Error>> {
    println!("configuring git for is {location} with {category}", location=repo.location, category=repo.category);
    let git_config: Option<GitConfig> = config.get_git_config(&repo.category);
    Ok(if let Some(GitConfig { email, name }) = git_config {
        if let Some(email) = email {
            let git_email_command = format!("git config user.email {email}");
            execute(git_email_command, Some(repo.location.as_ref()))?.wait()?;
        }
        if let Some(name) = name {
            let git_name_command = format!("git config user.name {name}");
            execute(git_name_command, Some(repo.location.as_ref()))?.wait()?;
        }
    })
}
