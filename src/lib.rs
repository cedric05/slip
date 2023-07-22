pub mod args;
pub mod config;
pub mod repolist;

pub use config::*;
pub mod tui;

use std::process::{Command, ExitStatus};

pub fn execute(
    command_to_launch: String,
    current_dir: Option<&str>,
) -> Result<ExitStatus, std::io::Error> {
    println!("{}", command_to_launch);
    let mut execute_command;
    if cfg!(target_os = "windows") {
        execute_command = Command::new("cmd");
        execute_command.args(["/C", &command_to_launch]);
    } else {
        execute_command = Command::new("sh");
        execute_command.args(["-c", &command_to_launch]);
    };
    if let Some(dir) = current_dir {
        execute_command.current_dir(dir);
    }
    let spawn = execute_command.spawn()?.wait()?;
    Ok(spawn)
}
