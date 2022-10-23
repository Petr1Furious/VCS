use std::path::PathBuf;

use crate::{
    json_files::{get_commit_data, CommitData},
    repo_file_manager::{get_file_changes, FileChange, get_repo_dir},
    vcs_state_manager::{get_commit_contents, get_commit_dir, get_commit_history},
};

pub struct LogResult {
    pub commit_list: Vec<(CommitData, Vec<(FileChange, PathBuf)>)>,
}

/// Log commits history until the current one
pub fn log_in_repo(repo_dir: &PathBuf) -> Result<LogResult, std::io::Error> {
    let commit_history = get_commit_history(&repo_dir)?;

    let mut log_result = LogResult {
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
            log_result.commit_list.last_mut().unwrap().1 = prev_file_changes;
        }
        log_result.commit_list
            .push((get_commit_data(&repo_dir, &commit)?.unwrap(), Vec::new()));
        prev_commit = Some(commit);
    }

    Ok(log_result)
}

pub fn log() -> Result<LogResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    log_in_repo(&repo_dir)
}
