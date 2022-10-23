use std::path::PathBuf;

use crate::{
    repo_file_manager::{get_repo_dir, FileChange},
    vcs_state_manager::VcsStateManager,
};

#[derive(PartialEq, Eq, Debug)]
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
pub fn jump_commit_in_repo(
    repo_dir: PathBuf,
    commit: &String,
) -> Result<JumpResult, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_dir);

    let cur_commit = vcs_state_manager.get_commit()?;
    let file_changes = vcs_state_manager.get_file_changes_commit(&cur_commit)?;
    if !file_changes.is_empty() {
        Ok(JumpResult::UncommitedChanges { file_changes })
    } else {
        match vcs_state_manager.get_commit_data(commit)? {
            Some(_commit_data) => {
                vcs_state_manager.jump_to_commit(&commit)?;
                Ok(JumpResult::Success {
                    commit: vcs_state_manager.get_commit()?,
                    branch: vcs_state_manager.get_branch()?,
                })
            }
            None => Ok(JumpResult::NotFound),
        }
    }
}

/// Jump to the last commit of the given branch
pub fn jump_branch_in_repo(
    repo_dir: PathBuf,
    branch: &String,
) -> Result<JumpResult, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_dir.clone());

    match vcs_state_manager.get_commits(branch)? {
        Some(commits) => jump_commit_in_repo(repo_dir, commits.last().unwrap()),
        None => Ok(JumpResult::NotFound),
    }
}

pub fn jump_commit(commit: &String) -> Result<JumpResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    jump_commit_in_repo(repo_dir, commit)
}

pub fn jump_branch(commit: &String) -> Result<JumpResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    jump_branch_in_repo(repo_dir, commit)
}
