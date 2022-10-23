use std::path::PathBuf;

use crate::{
    repo_file_manager::{files_equal, get_file_changes, get_repo_dir, FileChange},
    vcs_state_manager::VcsStateManager,
};

#[derive(PartialEq, Eq, Debug)]
pub enum MergeResult {
    NotLastCommit,
    MergeConflict {
        path1: PathBuf,
        path2: PathBuf,
    },
    Success {
        commit: String,
        file_changes: Vec<(FileChange, PathBuf)>,
    },
    MergeWithMaster,
}

/// Merge the given branch to master
pub fn merge_in_repo(repo_dir: PathBuf, branch: &String) -> Result<MergeResult, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_dir);

    if branch.clone() == String::from("master") {
        return Ok(MergeResult::MergeWithMaster);
    }

    let last_master_commit = vcs_state_manager
        .get_commits(&String::from("master"))?
        .unwrap()
        .pop()
        .unwrap();
    let last_branch_commit = vcs_state_manager
        .get_commits(&branch)?
        .unwrap()
        .pop()
        .unwrap();
    if last_master_commit != vcs_state_manager.get_commit()? {
        return Ok(MergeResult::NotLastCommit);
    }
    let common_commit = vcs_state_manager.get_commits(&branch)?.unwrap().remove(0);

    let master_contents = vcs_state_manager.get_commit_contents(&last_master_commit)?;
    let branch_contents = vcs_state_manager.get_commit_contents(&last_branch_commit)?;
    let common_contents = vcs_state_manager.get_commit_contents(&common_commit)?;

    let master_dir = vcs_state_manager.get_commit_dir(&last_master_commit);
    let branch_dir = vcs_state_manager.get_commit_dir(&last_branch_commit);
    let common_dir = vcs_state_manager.get_commit_dir(&common_commit);

    let file_changes_master =
        get_file_changes(&master_dir, &master_contents, &common_dir, &common_contents)?;
    let file_changes_branch =
        get_file_changes(&branch_dir, &branch_contents, &common_dir, &common_contents)?;

    let mut files_to_merge: Vec<PathBuf> = Vec::new();
    for entry in branch_contents.iter() {
        files_to_merge.push(entry.clone());
    }

    for entry in file_changes_master {
        let same = file_changes_branch.iter().find(|x| {
            x.1.strip_prefix(&branch_dir).unwrap() == entry.1.strip_prefix(&master_dir).unwrap()
        });
        if same.is_some() {
            if !files_equal(&entry.1, &same.unwrap().1)? {
                return Ok(MergeResult::MergeConflict {
                    path1: entry.1,
                    path2: same.unwrap().1.clone(),
                });
            }
        } else {
            if !files_to_merge
                .contains(&(&branch_dir).join(entry.1.strip_prefix(&master_dir).unwrap()))
            {
                files_to_merge.push(entry.1);
            }
        }
    }

    vcs_state_manager.commit_contents(
        &String::from(format!("Merged branch {}", branch)),
        &String::from("master"),
        &files_to_merge,
    )?;
    let new_commit = vcs_state_manager.get_commit()?;
    let file_changes = get_file_changes(
        &vcs_state_manager.get_commit_dir(&new_commit),
        &vcs_state_manager.get_commit_contents(&new_commit)?,
        &master_dir,
        &master_contents,
    )?;

    vcs_state_manager.remove_branch(branch)?;

    return Ok(MergeResult::Success {
        commit: new_commit,
        file_changes,
    });
}

pub fn merge(branch: &String) -> Result<MergeResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    merge_in_repo(repo_dir, branch)
}
