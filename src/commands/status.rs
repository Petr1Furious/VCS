use std::path::PathBuf;

use crate::{
    json_files::{get_branch, get_commit},
    repo_file_manager::{FileChange, get_repo_dir},
    vcs_state_manager::get_file_changes_commit,
};

#[derive(PartialEq, Eq, Debug)]
pub struct StatusResult {
    pub branch: String,
    pub file_changes: Vec<(FileChange, PathBuf)>,
}

/// Print changes to be committed
pub fn status_in_repo(repo_dir: &PathBuf) -> Result<StatusResult, std::io::Error> {
    let file_changes = get_file_changes_commit(&repo_dir, &get_commit(&repo_dir)?)?;
    Ok(StatusResult {
        branch: get_branch(&repo_dir)?,
        file_changes,
    })
}

pub fn status() -> Result<StatusResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    status_in_repo(&repo_dir)
}
