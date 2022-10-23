use std::path::PathBuf;

use crate::{
    json_files::{get_branch, get_commit, get_commits},
    repo_file_manager::{get_repo_dir, FileChange},
    vcs_state_manager::{self, get_file_changes_commit},
};

pub struct CommitResult {
    pub successful: bool,
    pub branch: String,
    pub commit: String,
    pub file_changes: Vec<(FileChange, PathBuf)>,
}

/// Commit current files in the repository
pub fn commit_in_repo(
    repo_dir: &PathBuf,
    message: &String,
) -> Result<CommitResult, std::io::Error> {
    let commit = get_commit(&repo_dir)?;
    let branch = get_branch(&repo_dir)?;
    if *get_commits(&repo_dir, &branch)?.unwrap().last().unwrap() != commit {
        Ok(CommitResult {
            successful: false,
            branch,
            commit,
            file_changes: Vec::new(),
        })
    } else {
        let file_changes = get_file_changes_commit(&repo_dir, &commit)?;
        if !file_changes.is_empty() {
            let new_commit = vcs_state_manager::commit(&repo_dir, &message, &branch)?;
            Ok(CommitResult {
                successful: true,
                branch,
                commit: new_commit,
                file_changes,
            })
        } else {
            Ok(CommitResult {
                successful: true,
                branch,
                commit,
                file_changes,
            })
        }
    }
}

pub fn commit(message: &String) -> Result<CommitResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    commit_in_repo(&repo_dir, message)
}
