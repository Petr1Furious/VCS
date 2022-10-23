use std::path::PathBuf;

use crate::{
    json_files::CommitData,
    repo_file_manager::{get_file_changes, get_repo_dir, FileChange},
    vcs_state_manager::VcsStateManager,
};

pub struct LogResult {
    pub commit_list: Vec<(CommitData, Vec<(FileChange, PathBuf)>)>,
}

/// Log commits history until the current one
pub fn log_in_repo(repo_dir: PathBuf) -> Result<LogResult, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_dir);

    let commit_history = vcs_state_manager.get_commit_history()?;

    let mut log_result = LogResult {
        commit_list: Vec::new(),
    };
    let mut prev_commit: Option<String> = None;
    for commit in commit_history {
        if prev_commit.is_some() {
            let prev_commit_value = prev_commit.unwrap();
            let prev_file_changes = get_file_changes(
                &vcs_state_manager.get_commit_dir(&prev_commit_value),
                &vcs_state_manager.get_commit_contents(&prev_commit_value)?,
                &vcs_state_manager.get_commit_dir(&commit),
                &vcs_state_manager.get_commit_contents(&commit)?,
            )?;
            log_result.commit_list.last_mut().unwrap().1 = prev_file_changes;
        }
        log_result.commit_list.push((
            vcs_state_manager.get_commit_data(&commit)?.unwrap(),
            Vec::new(),
        ));
        prev_commit = Some(commit);
    }

    Ok(log_result)
}

pub fn log() -> Result<LogResult, std::io::Error> {
    let repo_dir = get_repo_dir()?;
    log_in_repo(repo_dir)
}
