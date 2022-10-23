use std::path::PathBuf;

use crate::{repo_file_manager::get_repo_dir, vcs_state_manager::VcsStateManager};

#[derive(PartialEq, Eq, Debug)]
pub enum NewBranchResult {
    OnlyFromMaster,
    Success { commit: String },
    AlreadyExists,
}

/// Create a new branch from the current commit
pub fn new_branch_in_repo(
    repo_dir: PathBuf,
    new_branch: &String,
) -> Result<NewBranchResult, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_dir);

    let cur_branch = vcs_state_manager.get_branch()?;
    if cur_branch != String::from("master") {
        Ok(NewBranchResult::OnlyFromMaster)
    } else {
        if *new_branch != cur_branch {
            vcs_state_manager.new_branch(new_branch)?;
            Ok(NewBranchResult::Success {
                commit: vcs_state_manager.get_commit()?,
            })
        } else {
            Ok(NewBranchResult::AlreadyExists)
        }
    }
}

pub fn new_branch(new_branch: &String) -> Result<NewBranchResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    new_branch_in_repo(repo_dir, new_branch)
}
