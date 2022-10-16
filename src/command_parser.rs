extern crate clap;

use clap::{ArgGroup, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about, long_about = None)]
pub struct Arguments {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitCommand),

    Status,

    Commit(CommitCommand),

    Jump(JumpCommand),

    NewBranch(NewBranchCommand),

    Merge(MergeCommand),

    Log,
}

#[derive(Debug, Args)]
pub struct InitCommand {
    #[clap(long)]
    pub path: String,
}

#[derive(Debug, Args)]
pub struct CommitCommand {
    #[clap(long)]
    pub message: String,
}

#[derive(Debug, Args)]
#[clap(group(
    ArgGroup::new("test")
    .required(true)
    .args(&["commit", "branch"])
))]
pub struct JumpCommand {
    #[clap(long)]
    pub commit: Option<String>,

    #[clap(long)]
    pub branch: Option<String>,
}

#[derive(Debug, Args)]
pub struct NewBranchCommand {
    #[clap(long)]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct MergeCommand {
    #[clap(long)]
    pub branch: String,
}
