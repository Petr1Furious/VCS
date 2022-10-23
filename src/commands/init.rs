use std::path::PathBuf;

use crate::vcs_state_manager::VcsStateManager;

/// Initialize an empty repository
pub fn init(repo_path: PathBuf) -> Result<String, std::io::Error> {
    let mut vcs_state_manager = VcsStateManager::init(repo_path);
    vcs_state_manager.init_repository()
}
