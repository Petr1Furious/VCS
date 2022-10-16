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
    #[clap(about = "Initialize an empty repository")]
    Init(InitCommand),

    #[clap(about = "Print changes to be committed")]
    Status,

    #[clap(about = "Commit current files in the repository")]
    Commit(CommitCommand),

    #[clap(about = "Jump either to the given commit or to the last commit of the given branch")]
    Jump(JumpCommand),

    #[clap(name = "new_branch")]
    #[clap(about = "Create a new branch from the current commit")]
    NewBranch(NewBranchCommand),

    #[clap(about = "Merge the given branch to master")]
    Merge(MergeCommand),

    #[clap(about = "Log commits history until the current one")]
    Log,
}

#[derive(Debug, Args)]
pub struct InitCommand {
    #[clap(long, short)]
    pub path: String,
}

#[derive(Debug, Args)]
pub struct CommitCommand {
    #[clap(long, short)]
    pub message: String,
}

#[derive(Debug, Args)]
#[clap(group(
    ArgGroup::new("test")
    .required(true)
    .args(&["commit", "branch"])
))]
pub struct JumpCommand {
    #[clap(long, short)]
    pub commit: Option<String>,

    #[clap(long, short)]
    pub branch: Option<String>,
}

#[derive(Debug, Args)]
pub struct NewBranchCommand {
    #[clap(long)]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct MergeCommand {
    #[clap(long, short)]
    pub branch: String,
}
