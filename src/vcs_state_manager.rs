use std::{
    fs::{self},
    path::PathBuf,
    time::SystemTime,
};

use crate::{
    json_files::{
        add_branch_commit, add_commit_data, get_branch, get_branch_list, get_commit, get_commits,
        set_branch, set_commit, CommitData,
    },
    repo_file_manager::{
        copy_files_from_commit, copy_files_to_commit, get_contents, get_contents_hash,
        get_file_changes, remove_repo_files, FileChange,
    },
};

/// Initialize repository in the given path
pub fn init_repository(path: &PathBuf) -> Result<String, std::io::Error> {
    let dir = path.join(".vcs");
    fs::create_dir(&dir)?;
    fs::create_dir(&dir.join("commits"))?;

    let commit = commit(
        path,
        &String::from("Initial commit"),
        &String::from("master"),
    )?;
    Ok(commit)
}

/// Get files and folders of the given commit
pub fn get_commit_contents(
    repo_dir: &PathBuf,
    commit: &String,
) -> Result<Vec<PathBuf>, std::io::Error> {
    get_contents(&repo_dir.join(".vcs").join("commits").join(commit), false)
}

/// Get commit folder
pub fn get_commit_dir(repo_dir: &PathBuf, commit: &String) -> PathBuf {
    repo_dir.join(".vcs").join("commits").join(commit)
}

/// Get file changes in the repository relative to the given commit
pub fn get_file_changes_commit(
    repo_dir: &PathBuf,
    commit: &String,
) -> Result<Vec<(FileChange, PathBuf)>, std::io::Error> {
    let contents = get_contents(repo_dir, true)?;
    let commit_contents = get_commit_contents(repo_dir, commit)?;
    let commit_dir = repo_dir.join(".vcs").join("commits").join(commit);
    get_file_changes(repo_dir, &contents, &commit_dir, &commit_contents)
}

/// Commit the given files and folders
pub fn commit_contents(
    repo_dir: &PathBuf,
    message: &String,
    branch: &String,
    contents: &Vec<PathBuf>,
) -> Result<String, std::io::Error> {
    let commit = get_contents_hash(&contents)?;

    copy_files_to_commit(repo_dir, &contents, &commit)?;
    set_commit(repo_dir, &commit)?;
    set_branch(repo_dir, &branch)?;
    add_commit_data(
        repo_dir,
        CommitData::from(commit.clone(), message.clone(), SystemTime::now()),
    )?;
    add_branch_commit(&repo_dir, &branch, &commit)?;
    Ok(commit)
}

/// Commit from the repo folder
pub fn commit(
    repo_dir: &PathBuf,
    message: &String,
    branch: &String,
) -> Result<String, std::io::Error> {
    let contents = get_contents(&repo_dir, true)?;
    commit_contents(repo_dir, message, branch, &contents)
}

/// Get a branch containing the commit
pub fn get_branch_with_commit(
    repo_dir: &PathBuf,
    commit: &String,
) -> Result<Option<String>, std::io::Error> {
    let branch_list = get_branch_list(repo_dir)?;

    let mut res_branch: Option<String> = None;
    for branch in branch_list.branches {
        if branch.commits.iter().find(|x| *x == commit).is_some() {
            if branch.name != String::from("master") {
                return Ok(Some(branch.name));
            }
            res_branch = Some(branch.name);
        }
    }

    Ok(res_branch)
}

// Replace repo contents with the contents of the given commit
pub fn jump_to_commit(repo_dir: &PathBuf, commit: &String) -> Result<(), std::io::Error> {
    remove_repo_files(repo_dir)?;
    set_commit(repo_dir, commit)?;
    set_branch(
        repo_dir,
        &get_branch_with_commit(repo_dir, commit)?.unwrap(),
    )?;
    copy_files_from_commit(repo_dir, commit)
}

/// Create a new branch from the given commit
pub fn new_branch(repo_dir: &PathBuf, new_branch: &String) -> Result<(), std::io::Error> {
    add_branch_commit(repo_dir, new_branch, &get_commit(repo_dir)?)
}

/// Get commits history
pub fn get_commit_history(repo_dir: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let cur_commit = get_commit(repo_dir)?;
    let cur_branch = get_branch(repo_dir)?;

    let mut res: Vec<String> = Vec::new();
    if cur_branch != String::from("master") {
        let cur_branch_commits = get_commits(repo_dir, &cur_branch)?.unwrap();
        for commit in get_commits(repo_dir, &String::from("master"))?.unwrap() {
            if commit == cur_branch_commits[0] {
                break;
            }
            res.push(commit);
        }
    }

    for commit in get_commits(repo_dir, &cur_branch)?.unwrap() {
        res.push(commit.clone());
        if commit == cur_commit {
            break;
        }
    }

    res.reverse();
    Ok(res)
}
