use std::path::PathBuf;

use crate::{
    repo_file_manager::{get_repo_dir, FileChange},
    vcs_state_manager::VcsStateManager,
};

#[derive(PartialEq, Eq, Debug)]
pub struct StatusResult {
    pub branch: String,
    pub file_changes: Vec<(FileChange, PathBuf)>,
}

/// Print changes to be committed
pub fn status_in_repo(repo_dir: PathBuf) -> Result<StatusResult, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_dir);

    let cur_commit = vcs_state_manager.get_commit()?;
    let file_changes = vcs_state_manager.get_file_changes_commit(&cur_commit)?;
    Ok(StatusResult {
        branch: vcs_state_manager.get_branch()?,
        file_changes,
    })
}

pub fn status() -> Result<StatusResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    status_in_repo(repo_dir)
}
