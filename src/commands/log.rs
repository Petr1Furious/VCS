use std::path::PathBuf;

use crate::{
    json_files::{get_commit_data, CommitData},
    repo_file_manager::{get_file_changes, get_repo_dir, FileChange},
    vcs_state_manager::{get_commit_contents, get_commit_dir, get_commit_history},
};

pub struct LogResult {
    pub commit_list: Vec<(CommitData, Vec<(FileChange, PathBuf)>)>,
}

/// Log commits history until the current one
pub fn log() -> Result<LogResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    let commit_history = get_commit_history(&repo_dir)?;

    let mut res = LogResult {
        commit_list: Vec::new(),
    };
    let mut prev_commit: Option<String> = None;
    for commit in commit_history {
        if prev_commit.is_some() {
            let prev_commit_value = prev_commit.unwrap();
            let prev_file_changes = get_file_changes(
                &get_commit_dir(&repo_dir, &prev_commit_value),
                &get_commit_contents(&repo_dir, &prev_commit_value)?,
                &get_commit_dir(&repo_dir, &commit),
                &get_commit_contents(&repo_dir, &commit)?,
            )?;
            res.commit_list.last_mut().unwrap().1 = prev_file_changes;
        }
        res.commit_list
            .push((get_commit_data(&repo_dir, &commit)?.unwrap(), Vec::new()));
        prev_commit = Some(commit);
    }

    Ok(res)
}
