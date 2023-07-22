use clap::ArgGroup;
use clap::{Parser, Subcommand};

/// Simple slip command to better organize github repositories
#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(group(
    ArgGroup::new("category")
        // .required(true)
        .args(&["work", "personal"]),
))]
pub struct Args {
    #[clap(short, long, default_value = "~/.slip.toml")]
    pub config: String,

    #[clap(global = true, short, long)]
    pub work: bool,

    #[clap(global = true, short, long)]
    pub personal: bool,

    #[clap(subcommand)]
    pub command: Option<SubCommands>,
}

impl Args {
    pub fn config(&self) -> String {
        String::from(shellexpand::tilde(&self.config))
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum SubCommands {
    /// clone repository
    Clone {
        url: String,
        dir: Option<String>,
    },

    Reconfig,

    List {
        // filter repos
        #[clap(index = 1)]
        filter: Option<String>,
    },
    Ui,

    New {
        // repo name
        #[clap(index = 1)]
        repo: String,
    },

    Add {
        #[clap(index = 1)]
        repo: String,
    },
}
