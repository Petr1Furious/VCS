use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use vcs::{
    commands::{
        commit::commit_in_repo,
        init::init,
        jump::{jump_branch_in_repo, jump_commit_in_repo, JumpResult},
        log::log_in_repo,
        merge::{merge_in_repo, MergeResult},
        new_branch::{new_branch_in_repo, NewBranchResult},
        status::{status_in_repo, StatusResult},
    },
    repo_file_manager::FileChange,
};

fn modify_file(path: &PathBuf, content: &String) -> Result<(), std::io::Error> {
    File::create(path)?;
    fs::write(path, content)?;
    Ok(())
}

#[test]
fn it_works() {
    let repo_dir = Path::new(".").join("test_repo");
    if Path::exists(&repo_dir) {
        fs::remove_dir_all(&repo_dir).unwrap();
    }
    fs::create_dir(&repo_dir).unwrap();

    let mut another_branch_commit_history: Vec<String> = vec![init(&repo_dir).unwrap()];

    modify_file(
        &repo_dir.join("test_file"),
        &String::from("some content here"),
    )
    .unwrap();
    assert_eq!(
        status_in_repo(&repo_dir).unwrap(),
        StatusResult {
            branch: String::from("master"),
            file_changes: vec![(FileChange::Added, repo_dir.join("test_file"))]
        }
    );

    let commit_result = commit_in_repo(&repo_dir, &String::from("message1")).unwrap();
    assert!(commit_result.successful);
    assert_eq!(commit_result.branch, String::from("master"));
    assert_eq!(
        commit_result.file_changes,
        vec![(FileChange::Added, repo_dir.join("test_file"))]
    );
    another_branch_commit_history.push(commit_result.commit);

    modify_file(&repo_dir.join("test_file"), &String::from("something else")).unwrap();
    modify_file(
        &repo_dir.join("another_test_file"),
        &String::from("some content here 2"),
    )
    .unwrap();
    assert_eq!(
        status_in_repo(&repo_dir).unwrap(),
        StatusResult {
            branch: String::from("master"),
            file_changes: vec![
                (FileChange::Modified, repo_dir.join("test_file")),
                (FileChange::Added, repo_dir.join("another_test_file"))
            ]
        }
    );

    let commit_result = commit_in_repo(&repo_dir, &String::from("message2")).unwrap();
    assert!(commit_result.successful);
    assert_eq!(commit_result.branch, String::from("master"));
    assert_eq!(
        commit_result.file_changes,
        vec![
            (FileChange::Modified, repo_dir.join("test_file")),
            (FileChange::Added, repo_dir.join("another_test_file"))
        ]
    );
    let fork_commit = commit_result.commit;
    another_branch_commit_history.push(fork_commit.clone());

    modify_file(&repo_dir.join("merge_file"), &String::from("content 1")).unwrap();
    fs::remove_file(&repo_dir.join("test_file")).unwrap();

    let commit_result = commit_in_repo(&repo_dir, &String::from("message3")).unwrap();
    assert!(commit_result.successful);
    assert_eq!(commit_result.branch, String::from("master"));
    assert_eq!(
        commit_result.file_changes,
        vec![
            (FileChange::Added, repo_dir.join("merge_file")),
            (FileChange::Removed, repo_dir.join("test_file"))
        ]
    );
    let last_master_commit = commit_result.commit;

    assert_eq!(
        jump_commit_in_repo(&repo_dir, &fork_commit).unwrap(),
        JumpResult::Success {
            commit: fork_commit.clone(),
            branch: String::from("master")
        }
    );
    assert!(repo_dir.join("test_file").exists());

    assert_eq!(
        new_branch_in_repo(&repo_dir, &String::from("another_branch")).unwrap(),
        NewBranchResult::Success {
            commit: fork_commit.clone()
        }
    );
    assert_eq!(
        jump_branch_in_repo(&repo_dir, &String::from("another_branch")).unwrap(),
        JumpResult::Success {
            commit: fork_commit.clone(),
            branch: String::from("another_branch")
        }
    );

    modify_file(&repo_dir.join("merge_file"), &String::from("content 2")).unwrap();

    let commit_result = commit_in_repo(&repo_dir, &String::from("message4")).unwrap();
    assert!(commit_result.successful);
    assert_eq!(commit_result.branch, String::from("another_branch"));
    assert_eq!(
        commit_result.file_changes,
        vec![(FileChange::Added, repo_dir.join("merge_file")),]
    );
    another_branch_commit_history.push(commit_result.commit);

    let log_result = log_in_repo(&repo_dir).unwrap();
    for (i, commit) in log_result.commit_list.iter().enumerate() {
        assert_eq!(
            commit.0.hash,
            another_branch_commit_history[another_branch_commit_history.len() - 1 - i]
        );
    }
    assert_eq!(
        log_result.commit_list[3].0.message,
        String::from("Initial commit")
    );
    assert_eq!(
        log_result.commit_list[2].0.message,
        String::from("message1")
    );
    assert_eq!(
        log_result.commit_list[1].0.message,
        String::from("message2")
    );
    assert_eq!(
        log_result.commit_list[0].0.message,
        String::from("message4")
    );

    assert_eq!(
        merge_in_repo(&repo_dir, &String::from("another_branch")).unwrap(),
        MergeResult::NotLastCommit
    );

    assert_eq!(
        jump_branch_in_repo(&repo_dir, &String::from("master")).unwrap(),
        JumpResult::Success {
            commit: last_master_commit.clone(),
            branch: String::from("master")
        }
    );

    assert_eq!(
        merge_in_repo(&repo_dir, &String::from("another_branch")).unwrap(),
        MergeResult::MergeConflict {
            path1: repo_dir
                .join(".vcs")
                .join("commits")
                .join(&last_master_commit)
                .join("merge_file"),
            path2: repo_dir
                .join(".vcs")
                .join("commits")
                .join(&another_branch_commit_history.last().unwrap())
                .join("merge_file")
        }
    );

    modify_file(&repo_dir.join("merge_file"), &String::from("content 2")).unwrap();

    let commit_result = commit_in_repo(&repo_dir, &String::from("message5")).unwrap();
    assert!(commit_result.successful);
    assert_eq!(commit_result.branch, String::from("master"));
    assert_eq!(
        commit_result.file_changes,
        vec![(FileChange::Modified, repo_dir.join("merge_file")),]
    );

    let merge_result = merge_in_repo(&repo_dir, &String::from("another_branch")).unwrap();
    match merge_result {
        MergeResult::Success {
            commit,
            file_changes,
        } => {
            assert_eq!(
                file_changes,
                vec![(
                    FileChange::Added,
                    repo_dir
                        .join(".vcs")
                        .join("commits")
                        .join(commit)
                        .join("test_file")
                )]
            )
        }

        _ => {
            panic!("Expected merge to be successful");
        }
    }

    fs::remove_dir_all(&repo_dir).unwrap();
}
