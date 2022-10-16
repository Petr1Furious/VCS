use std::path::PathBuf;

use crate::{
    json_files::{get_branch, get_commit, get_commit_data, get_commits},
    repo_file_manager::{get_repo_dir, FileChange},
    vcs_state_manager::{get_file_changes_commit, jump_to_commit},
};

pub enum JumpResult {
    UncommitedChanges {
        file_changes: Vec<(FileChange, PathBuf)>,
    },
    NotFound,
    Success {
        commit: String,
        branch: String,
    },
}

/// Jump to the given commit
pub fn jump_commit(commit: &String) -> Result<JumpResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;

    let file_changes = get_file_changes_commit(&repo_dir, &get_commit(&repo_dir)?)?;
    if !file_changes.is_empty() {
        Ok(JumpResult::UncommitedChanges { file_changes })
    } else {
        match get_commit_data(&repo_dir, commit)? {
            Some(_commit_data) => {
                jump_to_commit(&repo_dir, commit)?;
                Ok(JumpResult::Success {
                    commit: get_commit(&repo_dir)?,
                    branch: get_branch(&repo_dir)?,
                })
            }
            None => Ok(JumpResult::NotFound),
        }
    }
}

/// Jump to the last commit of the given branch
pub fn jump_branch(branch: &String) -> Result<JumpResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;

    match get_commits(&repo_dir, branch)? {
        Some(commits) => jump_commit(commits.last().unwrap()),
        None => Ok(JumpResult::NotFound),
    }
}
