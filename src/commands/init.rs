use std::path::PathBuf;

use crate::vcs_state_manager::init_repository;

/// Initialize an empty repository
pub fn init(path: &PathBuf) -> Result<String, std::io::Error> {
    init_repository(path)
}
