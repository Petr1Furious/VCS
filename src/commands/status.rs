use std::path::PathBuf;

use crate::{
    json_files::{get_branch, get_commit},
    repo_file_manager::{get_repo_dir, FileChange},
    vcs_state_manager::get_file_changes_commit,
};

pub struct LogData {
    pub branch: String,
    pub file_changes: Vec<(FileChange, PathBuf)>,
}

/// Print changes to be committed
pub fn status() -> Result<LogData, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    let file_changes = get_file_changes_commit(&repo_dir, &get_commit(&repo_dir)?)?;
    Ok(LogData {
        branch: get_branch(&repo_dir)?,
        file_changes,
    })
}
