use std::{path::{Path, PathBuf}, fs::{self, File}};

use vcs::commands::{init::init, commit::commit_in_repo, jump::{jump_commit_in_repo, jump_branch_in_repo, JumpResult}, new_branch::new_branch_in_repo, merge::merge_in_repo};

#[test]
fn test_commit_jump() {
    let repo_dir = Path::new(".").join("test_repo2");
    if Path::exists(&repo_dir) {
        fs::remove_dir_all(&repo_dir).unwrap();
    }
    fs::create_dir(&repo_dir).unwrap();

    init(repo_dir.clone()).unwrap();

    const FILES_COUNT: usize = 20;
    let mut commits: Vec<String> = Vec::new();
    for file_num in 0..FILES_COUNT {
        File::create(repo_dir.join(file_num.to_string())).unwrap();

        let result = commit_in_repo(repo_dir.clone(), &file_num.to_string()).unwrap();
        assert!(result.successful);

        commits.push(result.commit);
    }

    for i in 0..FILES_COUNT {
        jump_commit_in_repo(repo_dir.clone(), &commits[i], None).unwrap();
        assert_eq!(fs::read_dir(&repo_dir).unwrap().count(), i + 2);
    }

    fs::remove_dir_all(&repo_dir).unwrap();
}

fn create_branches(repo_dir: &PathBuf, branches_count: usize) -> Vec<String> {
    let first_commit = init(repo_dir.clone()).unwrap();

    let mut commits: Vec<String> = Vec::new();
    for branch_num in 0..branches_count {
        jump_commit_in_repo(repo_dir.clone(), &first_commit, Some(&String::from("master"))).unwrap();
        new_branch_in_repo(repo_dir.clone(), &String::from(branch_num.to_string())).unwrap();
        jump_branch_in_repo(repo_dir.clone(), &String::from(branch_num.to_string())).unwrap();

        File::create(repo_dir.join(branch_num.to_string())).unwrap();

        let result = commit_in_repo(repo_dir.clone(), &branch_num.to_string()).unwrap();
        assert!(result.successful);

        commits.push(result.commit);
    }

    commits
}

#[test]
fn test_new_branch() {
    let repo_dir = Path::new(".").join("test_repo3");
    if Path::exists(&repo_dir) {
        fs::remove_dir_all(&repo_dir).unwrap();
    }
    fs::create_dir(&repo_dir).unwrap();

    const BRANCHES_COUNT: usize = 20;

    let commits = create_branches(&repo_dir, BRANCHES_COUNT);

    for i in 0..BRANCHES_COUNT {
        jump_commit_in_repo(repo_dir.clone(), &commits[i], None).unwrap();
        
        assert_eq!(fs::read_dir(&repo_dir).unwrap().count(), 2);
        for entry in fs::read_dir(&repo_dir).unwrap() {
            let cur_path = entry.unwrap().path();
            if cur_path != repo_dir.join(".vcs") {
                assert_eq!(repo_dir.join(i.to_string()), cur_path)
            }
        }
    }

    fs::remove_dir_all(&repo_dir).unwrap();
}

#[test]
fn test_merge() {
    let repo_dir = Path::new(".").join("test_repo4");
    if Path::exists(&repo_dir) {
        fs::remove_dir_all(&repo_dir).unwrap();
    }
    fs::create_dir(&repo_dir).unwrap();

    const BRANCHES_COUNT: usize = 20;

    create_branches(&repo_dir, BRANCHES_COUNT);
    for i in 0..BRANCHES_COUNT {
        match jump_branch_in_repo(repo_dir.clone(), &String::from("master")).unwrap() {
            JumpResult::Success { commit: _commit, branch: _branch } => {}
            _ => {
                panic!("")
            }
        }
        merge_in_repo(repo_dir.clone(), &i.to_string()).unwrap();
    }
    jump_branch_in_repo(repo_dir.clone(), &String::from("master")).unwrap();
    assert_eq!(fs::read_dir(&repo_dir).unwrap().count(), BRANCHES_COUNT + 1);

    fs::remove_dir_all(&repo_dir).unwrap();
}
