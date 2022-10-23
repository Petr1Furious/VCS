use std::path::PathBuf;

use crate::{
    json_files::{get_branch, get_commit},
    repo_file_manager::get_repo_dir,
    vcs_state_manager,
};

#[derive(PartialEq, Eq, Debug)]
pub enum NewBranchResult {
    OnlyFromMaster,
    Success { commit: String },
    AlreadyExists,
}

/// Create a new branch from the current commit
pub fn new_branch_in_repo(
    repo_dir: &PathBuf,
    new_branch: &String,
) -> Result<NewBranchResult, std::io::Error> {
    let cur_branch = get_branch(&repo_dir)?;
    if cur_branch != String::from("master") {
        Ok(NewBranchResult::OnlyFromMaster)
    } else {
        if *new_branch != cur_branch {
            vcs_state_manager::new_branch(&repo_dir, new_branch)?;
            Ok(NewBranchResult::Success {
                commit: get_commit(&repo_dir)?,
            })
        } else {
            Ok(NewBranchResult::AlreadyExists)
        }
    }
}

pub fn new_branch(new_branch: &String) -> Result<NewBranchResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    new_branch_in_repo(&repo_dir, new_branch)
}
