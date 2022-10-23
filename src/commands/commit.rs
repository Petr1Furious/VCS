use std::path::PathBuf;

use crate::{
    repo_file_manager::{get_repo_dir, FileChange},
    vcs_state_manager::VcsStateManager,
};

pub struct CommitResult {
    pub successful: bool,
    pub branch: String,
    pub commit: String,
    pub file_changes: Vec<(FileChange, PathBuf)>,
}

/// Commit current files in the repository
pub fn commit_in_repo(repo_dir: PathBuf, message: &String) -> Result<CommitResult, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_dir);

    let commit = vcs_state_manager.get_commit()?;
    let branch = vcs_state_manager.get_branch()?;
    if *vcs_state_manager
        .get_commits(&branch)?
        .unwrap()
        .last()
        .unwrap()
        != commit
    {
        Ok(CommitResult {
            successful: false,
            branch,
            commit,
            file_changes: Vec::new(),
        })
    } else {
        let file_changes = vcs_state_manager.get_file_changes_commit(&commit)?;
        if !file_changes.is_empty() {
            let new_commit = vcs_state_manager.commit(&message, &branch)?;
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
    commit_in_repo(repo_dir, message)
}
