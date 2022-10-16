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

pub fn get_commit(repo_dir: &PathBuf) -> Result<String, std::io::Error> {
    Ok(get_commit_and_branch(repo_dir)?.commit)
}

pub fn get_branch(repo_dir: &PathBuf) -> Result<String, std::io::Error> {
    Ok(get_commit_and_branch(repo_dir)?.branch)
}

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

pub fn set_commit(repo_dir: &PathBuf, commit: &String) -> Result<(), std::io::Error> {
    set_commit_and_branch(
        repo_dir,
        CommitAndBranch::from(commit.clone(), get_branch(repo_dir)?),
    )
}

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

#[derive(Serialize, Deserialize)]
pub struct CommitList {
    commits: Vec<CommitData>,
}

impl CommitList {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
        }
    }
}

pub fn get_commit_list(repo_dir: &PathBuf) -> Result<CommitList, std::io::Error> {
    if !repo_dir.join(".vcs").join("commit_list.json").exists() {
        save_commit_list(repo_dir, CommitList::new())?;
    }
    Ok(serde_json::from_str::<CommitList>(
        fs::read_to_string(repo_dir.join(".vcs").join("commit_list.json"))?.as_str(),
    )
    .unwrap())
}

pub fn save_commit_list(
    repo_dir: &PathBuf,
    commits_data: CommitList,
) -> Result<(), std::io::Error> {
    fs::write(
        repo_dir.join(".vcs").join("commit_list.json"),
        serde_json::to_string(&commits_data).unwrap().as_bytes(),
    )
}

pub fn add_commit_data(repo_dir: &PathBuf, commit_data: CommitData) -> Result<(), std::io::Error> {
    let mut commits_data = get_commit_list(&repo_dir)?;
    commits_data.commits.push(commit_data);
    save_commit_list(&repo_dir, commits_data)
}

pub fn get_commit_data(
    repo_dir: &PathBuf,
    commit: &String,
) -> Result<Option<CommitData>, std::io::Error> {
    let commit_list = get_commit_list(repo_dir)?;

    let temp = commit_list
        .commits
        .iter()
        .find(|x| x.hash == commit.clone());
    if temp.is_none() {
        return Ok(None);
    } else {
        Ok(Some(temp.unwrap().clone()))
    }
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

pub fn save_branch_list(repo_dir: &PathBuf, branch_list: BranchList) -> Result<(), std::io::Error> {
    fs::write(
        repo_dir.join(".vcs").join("branch_list.json"),
        serde_json::to_string(&branch_list).unwrap().as_bytes(),
    )
}

pub fn get_branch_list(repo_dir: &PathBuf) -> Result<BranchList, std::io::Error> {
    if !repo_dir.join(".vcs").join("branch_list.json").exists() {
        save_branch_list(repo_dir, BranchList::new())?;
    }
    Ok(serde_json::from_str::<BranchList>(
        fs::read_to_string(repo_dir.join(".vcs").join("branch_list.json"))?.as_str(),
    )
    .unwrap())
}

pub fn add_branch_commit(
    repo_dir: &PathBuf,
    branch: &String,
    commit: &String,
) -> Result<(), std::io::Error> {
    let mut branch_list = get_branch_list(repo_dir)?;
    let found = branch_list.branches.iter_mut().find(|x| x.name == *branch);
    match found {
        Some(branch_data) => {
            branch_data.commits.push(commit.clone());
        }
        None => {
            branch_list
                .branches
                .push(BranchData::from(branch.clone(), commit.clone()));
        }
    }

    save_branch_list(repo_dir, branch_list)
}

pub fn get_commits(
    repo_dir: &PathBuf,
    branch: &String,
) -> Result<Option<Vec<String>>, std::io::Error> {
    let branch_list = get_branch_list(repo_dir)?;
    let branch = branch_list.branches.into_iter().find(|x| x.name == *branch);
    match branch {
        Some(branch_data) => Ok(Some(branch_data.commits)),
        None => Ok(None),
    }
}

pub fn remove_branch(repo_dir: &PathBuf, branch: &String) -> Result<(), std::io::Error> {
    let mut branch_list = get_branch_list(repo_dir)?;
    let found = branch_list
        .branches
        .iter_mut()
        .position(|x| x.name == *branch);
    if found.is_some() {
        branch_list.branches.remove(found.unwrap());
        save_branch_list(repo_dir, branch_list)?;
    }
    Ok(())
}
