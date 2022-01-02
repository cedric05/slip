use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use clap::StructOpt;
use slip_git::args::{Args, Commands};
use slip_git::config::Config;
use url::Url;

fn main() {
    let cli = Args::parse();

    match &cli.command {
        Commands::Clone { url, dir } => {
            let config = cli.config();
            let config: Config = fs::read_to_string(config)
                .map(|x| toml::from_str(&x))
                .unwrap_or_else(|_| Ok(Config::new()))
                .unwrap();
            let reporoot = if cli.personal {
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
            let directory_to_clone =
                Path::new(reporoot).join(dir.as_ref().map(|x| x.to_owned()).unwrap_or_else(|| {
                    /*
                     *
                     * Its created for my case. may not work for all
                     * where git clone github.com/adsfdsafa
                     * here, username is username of user running
                     */
                    // https://github.com/gitignore/gitignore
                    if url.starts_with("https://") {
                        Url::parse(url).unwrap().path()[1..].to_string()
                    } else {
                        // git@github.com/gitignore/gitignore
                        let at = url.rfind(':').unwrap() + 1;
                        url[at..(url.len())].to_owned()
                    }
                }));
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
                child.wait().expect("child command ran into error");
            } else {
                println!("child command ran into error");
            };
        }
    };
}
