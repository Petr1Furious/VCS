use std::{fs, path::PathBuf, time::SystemTime};

use crate::{
    json_files::{
        add_branch_commit, add_commit_data, get_branch, get_branch_list, get_commit,
        get_commit_data, get_commits, remove_branch, set_branch, set_branch_list, set_commit,
        BranchList, CommitData,
    },
    repo_file_manager::{
        copy_files_from_commit, copy_files_to_commit, get_contents, get_contents_hash,
        get_file_changes, remove_repo_files, FileChange,
    },
};

pub struct VcsStateManager {
    repo_dir: PathBuf,
    cur_commit: Option<String>,
    cur_branch: Option<String>,
}

impl VcsStateManager {
    /// Initialize VCS state manager
    pub fn init(repo_dir: PathBuf) -> Self {
        Self {
            repo_dir,
            cur_commit: None,
            cur_branch: None,
        }
    }

    pub fn get_commit(self: &mut Self) -> Result<String, std::io::Error> {
        match self.cur_commit.clone() {
            Some(value) => Ok(value),

            None => {
                self.cur_commit = Some(get_commit(&self.repo_dir)?);
                Ok(self.cur_commit.clone().unwrap())
            }
        }
    }

    pub fn get_branch(self: &mut Self) -> Result<String, std::io::Error> {
        match self.cur_branch.clone() {
            Some(value) => Ok(value),

            None => {
                self.cur_branch = Some(get_branch(&self.repo_dir)?);
                Ok(self.cur_branch.clone().unwrap())
            }
        }
    }

    pub fn set_commit(self: &mut Self, commit: &String) -> Result<(), std::io::Error> {
        self.cur_commit = Some(commit.clone());
        set_commit(&self.repo_dir, commit)
    }

    pub fn set_branch(self: &mut Self, branch: &String) -> Result<(), std::io::Error> {
        self.cur_branch = Some(branch.clone());
        set_branch(&self.repo_dir, branch)
    }

    pub fn add_commit_data(self: &mut Self, commit_data: CommitData) -> Result<(), std::io::Error> {
        add_commit_data(&self.repo_dir, commit_data)
    }

    pub fn get_branch_list(self: &mut Self) -> Result<BranchList, std::io::Error> {
        get_branch_list(&self.repo_dir)
    }

    pub fn set_branch_list(self: &mut Self, branch_list: BranchList) -> Result<(), std::io::Error> {
        set_branch_list(&self.repo_dir, branch_list)
    }

    pub fn get_commits(
        self: &mut Self,
        branch: &String,
    ) -> Result<Option<Vec<String>>, std::io::Error> {
        get_commits(&self.repo_dir, branch)
    }

    pub fn get_commit_data(
        self: &mut Self,
        commit: &String,
    ) -> Result<Option<CommitData>, std::io::Error> {
        get_commit_data(&self.repo_dir, commit)
    }

    pub fn add_branch_commit(
        self: &mut Self,
        branch: &String,
        commit: &String,
    ) -> Result<(), std::io::Error> {
        add_branch_commit(&self.repo_dir, branch, commit)
    }

    pub fn remove_branch(self: &mut Self, branch: &String) -> Result<(), std::io::Error> {
        remove_branch(&self.repo_dir, branch)
    }

    /// Initialize repository in the given path
    pub fn init_repository(self: &mut Self) -> Result<String, std::io::Error> {
        let working_dir = self.repo_dir.join(".vcs");
        fs::create_dir(&working_dir)?;
        fs::create_dir(&working_dir.join("commits"))?;

        let commit = self.commit(&String::from("Initial commit"), &String::from("master"))?;
        Ok(commit)
    }

    /// Get files and folders of the given commit
    pub fn get_commit_contents(
        self: &mut Self,
        commit: &String,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        get_contents(
            &self.repo_dir.join(".vcs").join("commits").join(commit),
            false,
        )
    }

    /// Get commit folder
    pub fn get_commit_dir(self: &mut Self, commit: &String) -> PathBuf {
        self.repo_dir.join(".vcs").join("commits").join(commit)
    }

    /// Get file changes in the repository relative to the given commit
    pub fn get_file_changes_commit(
        self: &mut Self,
        commit: &String,
    ) -> Result<Vec<(FileChange, PathBuf)>, std::io::Error> {
        let contents = get_contents(&self.repo_dir, true)?;
        let commit_contents = self.get_commit_contents(commit)?;
        let commit_dir = self.repo_dir.join(".vcs").join("commits").join(commit);
        get_file_changes(&self.repo_dir, &contents, &commit_dir, &commit_contents)
    }

    /// Commit the given files and folders
    pub fn commit_contents(
        self: &mut Self,
        message: &String,
        branch: &String,
        contents: &Vec<PathBuf>,
    ) -> Result<String, std::io::Error> {
        let commit = get_contents_hash(&contents)?;

        copy_files_to_commit(&self.repo_dir, &contents, &commit)?;
        self.set_commit(&commit)?;
        self.set_branch(&branch)?;
        self.add_commit_data(CommitData::from(
            commit.clone(),
            message.clone(),
            SystemTime::now(),
        ))?;
        self.add_branch_commit(&branch, &commit)?;
        Ok(commit)
    }

    /// Commit from the repo folder
    pub fn commit(
        self: &mut Self,
        message: &String,
        branch: &String,
    ) -> Result<String, std::io::Error> {
        let contents = get_contents(&self.repo_dir, true)?;
        self.commit_contents(message, branch, &contents)
    }

    /// Get a branch containing the commit
    pub fn get_branch_with_commit(
        self: &mut Self,
        commit: &String,
    ) -> Result<Option<String>, std::io::Error> {
        let branch_list = self.get_branch_list()?;

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
    pub fn jump_to_commit(self: &mut Self, commit: &String) -> Result<(), std::io::Error> {
        remove_repo_files(&self.repo_dir)?;

        let branch_with_commit = self.get_branch_with_commit(commit)?.unwrap();
        self.set_commit(commit)?;
        self.set_branch(&branch_with_commit)?;
        copy_files_from_commit(&self.repo_dir, commit)
    }

    /// Create a new branch from the given commit
    pub fn new_branch(self: &mut Self, new_branch: &String) -> Result<(), std::io::Error> {
        let cur_commit = self.get_commit()?;
        self.add_branch_commit(new_branch, &cur_commit)
    }

    /// Get commits history
    pub fn get_commit_history(self: &mut Self) -> Result<Vec<String>, std::io::Error> {
        let cur_commit = self.get_commit()?;
        let cur_branch = self.get_branch()?;

        let mut commit_history: Vec<String> = Vec::new();
        if cur_branch != String::from("master") {
            let cur_branch_commits = self.get_commits(&cur_branch)?.unwrap();
            for commit in self.get_commits(&String::from("master"))?.unwrap() {
                if commit == cur_branch_commits[0] {
                    break;
                }
                commit_history.push(commit);
            }
        }

        for commit in self.get_commits(&cur_branch)?.unwrap() {
            commit_history.push(commit.clone());
            if commit == cur_commit {
                break;
            }
        }

        commit_history.reverse();
        Ok(commit_history)
    }
}
