use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_with::formats::Flexible;
use serde_with::TimestampMilliSeconds;

#[derive(Serialize, Deserialize)]
struct CommitAndBranch {
    commit: String,
    branch: String,
}

impl CommitAndBranch {
    fn from(commit: String, branch: String) -> Self {
        Self { commit, branch }
    }
}

/// Get commit and branch from state.json
fn get_commit_and_branch(repo_dir: &PathBuf) -> Result<CommitAndBranch, std::io::Error> {
    if !repo_dir.join(".vcs").join("state.json").exists() {
        set_commit_and_branch(
            repo_dir,
            CommitAndBranch::from(String::new(), String::new()),
        )?;
    }
    Ok(serde_json::from_str::<CommitAndBranch>(
        fs::read_to_string(repo_dir.join(".vcs").join("state.json"))?.as_str(),
    )
    .unwrap())
}

/// Get the commit from state.json
pub fn get_commit(repo_dir: &PathBuf) -> Result<String, std::io::Error> {
    Ok(get_commit_and_branch(repo_dir)?.commit)
}

/// Get the branch from state.json
pub fn get_branch(repo_dir: &PathBuf) -> Result<String, std::io::Error> {
    Ok(get_commit_and_branch(repo_dir)?.branch)
}

/// Set commit and branch to state.json
fn set_commit_and_branch(
    repo_dir: &PathBuf,
    commit_and_branch: CommitAndBranch,
) -> Result<(), std::io::Error> {
    fs::write(
        repo_dir.join(".vcs").join("state.json"),
        serde_json::to_string(&commit_and_branch)
            .unwrap()
            .as_bytes(),
    )
}

/// Set the commit to state.json
pub fn set_commit(repo_dir: &PathBuf, commit: &String) -> Result<(), std::io::Error> {
    set_commit_and_branch(
        repo_dir,
        CommitAndBranch::from(commit.clone(), get_branch(repo_dir)?),
    )
}

/// Set the branch to state.json
pub fn set_branch(repo_dir: &PathBuf, branch: &String) -> Result<(), std::io::Error> {
    set_commit_and_branch(
        repo_dir,
        CommitAndBranch::from(get_commit(repo_dir)?, branch.clone()),
    )
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct CommitData {
    pub hash: String,
    pub message: String,
    #[serde_as(as = "TimestampMilliSeconds<String, Flexible>")]
    pub date: SystemTime,
}

impl CommitData {
    pub fn from(hash: String, message: String, time: SystemTime) -> Self {
        Self {
            hash,
            message,
            date: time,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommitList {
    pub commits: Vec<CommitData>,
}

impl CommitList {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
        }
    }
}

/// Get commit list from commit_list.json
pub fn get_commit_list(repo_dir: &PathBuf) -> Result<CommitList, std::io::Error> {
    if !repo_dir.join(".vcs").join("commit_list.json").exists() {
        set_commit_list(repo_dir, CommitList::new())?;
    }
    Ok(serde_json::from_str::<CommitList>(
        fs::read_to_string(repo_dir.join(".vcs").join("commit_list.json"))?.as_str(),
    )
    .unwrap())
}

/// Set commit list to commit_list.json
pub fn set_commit_list(repo_dir: &PathBuf, commits_data: CommitList) -> Result<(), std::io::Error> {
    fs::write(
        repo_dir.join(".vcs").join("commit_list.json"),
        serde_json::to_string(&commits_data).unwrap().as_bytes(),
    )
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BranchData {
    pub name: String,
    pub commits: Vec<String>,
}

impl BranchData {
    pub fn from(name: String, commit: String) -> Self {
        Self {
            name,
            commits: vec![commit],
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BranchList {
    pub branches: Vec<BranchData>,
}

impl BranchList {
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
        }
    }
}

/// Set the branch list to branch_list.json
pub fn set_branch_list(repo_dir: &PathBuf, branch_list: BranchList) -> Result<(), std::io::Error> {
    fs::write(
        repo_dir.join(".vcs").join("branch_list.json"),
        serde_json::to_string(&branch_list).unwrap().as_bytes(),
    )
}

/// Get the branch list from branch_list.json
pub fn get_branch_list(repo_dir: &PathBuf) -> Result<BranchList, std::io::Error> {
    if !repo_dir.join(".vcs").join("branch_list.json").exists() {
        set_branch_list(repo_dir, BranchList::new())?;
    }
    Ok(serde_json::from_str::<BranchList>(
        fs::read_to_string(repo_dir.join(".vcs").join("branch_list.json"))?.as_str(),
    )
    .unwrap())
}
