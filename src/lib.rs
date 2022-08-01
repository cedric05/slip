pub mod args;
pub mod config;
pub mod repolist;

pub use config::*;
pub mod tui;

use std::process::{Command, Stdio};

pub fn execute(
    clone_command_str: String,
    current_dir: Option<&str>,
) -> Result<std::process::Child, std::io::Error> {
    println!("{}", clone_command_str);
    let mut execute_command;
    if cfg!(target_os = "windows") {
        execute_command = Command::new("cmd");
        execute_command.args(["/C", &clone_command_str]);
    } else {
        execute_command = Command::new("sh");
        execute_command.args(["-c", &clone_command_str]);
    };
    if let Some(dir) = current_dir {
        execute_command.current_dir(dir);
    }
    let spawn = execute_command
        .stdin(Stdio::piped()) // write to terminal
        .stdout(Stdio::piped()) // write to terminal
        .spawn();
    spawn
}
