use std::path::PathBuf;

use crate::{
    json_files::{get_commit, get_commits, remove_branch},
    repo_file_manager::{files_equal, get_file_changes, get_repo_dir, FileChange},
    vcs_state_manager::{commit_contents, get_commit_contents, get_commit_dir},
};

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
pub fn merge(branch: &String) -> Result<MergeResult, std::io::Error> {
    if branch.clone() == String::from("master") {
        return Ok(MergeResult::MergeWithMaster);
    }

    let repo_dir = get_repo_dir()?;

    let last_master_commit = get_commits(&repo_dir, &String::from("master"))?
        .unwrap()
        .pop()
        .unwrap();
    let last_branch_commit = get_commits(&repo_dir, &branch)?.unwrap().pop().unwrap();
    if last_master_commit != get_commit(&repo_dir)? {
        return Ok(MergeResult::NotLastCommit);
    }
    let common_commit = get_commits(&repo_dir, &branch)?.unwrap().remove(0);

    let master_contents = get_commit_contents(&repo_dir, &last_master_commit)?;
    let branch_contents = get_commit_contents(&repo_dir, &last_branch_commit)?;
    let common_contents = get_commit_contents(&repo_dir, &common_commit)?;

    let master_dir = get_commit_dir(&repo_dir, &last_master_commit);
    let branch_dir = get_commit_dir(&repo_dir, &last_branch_commit);
    let common_dir = get_commit_dir(&repo_dir, &common_commit);

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
            files_to_merge.push(entry.1);
        }
    }

    commit_contents(
        &repo_dir,
        &String::from(format!("Merged branch {}", branch)),
        &String::from("master"),
        &files_to_merge,
    )?;
    let new_commit = get_commit(&repo_dir)?;
    let file_changes = get_file_changes(
        &get_commit_dir(&repo_dir, &new_commit),
        &get_commit_contents(&repo_dir, &new_commit)?,
        &master_dir,
        &master_contents,
    )?;

    remove_branch(&repo_dir, branch)?;

    return Ok(MergeResult::Success {
        commit: new_commit,
        file_changes,
    });
}
