use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use clap::StructOpt;
use slip_git::args::{Args, Commands};
use slip_git::config::{Config, WorkOrPersonal};
use url::Url;

mod repolist;
mod tui;

use repolist::*;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    match &cli.command {
        Commands::Ui => {
            tui::main()?;
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
                } else {
                    // git@github.com/gitignore/gitignore
                    url[(url.rfind(':').unwrap() + 1)..(url.len())].to_string()
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
            println!("{}", clone_command_str);
            let mut execute_command;
            if cfg!(target_os = "windows") {
                execute_command = Command::new("cmd");
                execute_command.args(["/C", &clone_command_str]);
            } else {
                execute_command = Command::new("sh");
                execute_command.args(["-c", &clone_command_str]);
            };
            let spawn = execute_command
                .stdin(Stdio::piped()) // write to terminal
                .stdout(Stdio::piped()) // write to terminal
                .spawn();
            if let Ok(mut child) = spawn {
                let status = child.wait().expect("child command ran into error");
                if status.success() {
                    let mut repos_list = RepoList::get_config()?;
                    repos_list.repos.push(Repo {
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
                    });
                    repos_list.save_config();
                }
            } else {
                println!("child command ran into error");
            };
        }
    };
    Ok(())
}
