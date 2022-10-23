mod command_parser;
pub mod commands;
pub mod json_files;
pub mod repo_file_manager;
pub mod vcs_state_manager;

use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use clap::Parser;
use command_parser::{Arguments, Commands};
use commands::{
    commit::commit,
    init::init,
    jump::{jump_branch, jump_commit, JumpResult},
    log::log,
    merge::{merge, MergeResult},
    new_branch::{new_branch, NewBranchResult},
    status::status,
};

use crate::repo_file_manager::FileChange;

/// Print file changes in git format
fn print_file_changes(file_changes: &Vec<(FileChange, PathBuf)>) {
    for file_change in file_changes {
        let mut string_change = String::new();
        match file_change.0 {
            FileChange::Equal => {}
            FileChange::Added => {
                string_change = String::from("added");
            }
            FileChange::Modified => {
                string_change = String::from("modified");
            }
            FileChange::Removed => {
                string_change = String::from("removed");
            }
        }
        println!(
            "  {}: {}",
            string_change,
            file_change
                .1
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
        );
    }
}

/// Make the word plural if the count is not equal to 1
fn check_for_plural(word: String, count: usize) -> String {
    if count != 1 {
        word + "s"
    } else {
        word.clone()
    }
}

/// Print numbers of modified, added and removed files
fn print_changes_count(file_changes: &Vec<(FileChange, PathBuf)>) {
    let mut modified_count: usize = 0;
    let mut added_count: usize = 0;

    for file_change in file_changes.iter() {
        if file_change.0 == FileChange::Modified {
            modified_count += 1;
        }
        if file_change.0 == FileChange::Added {
            added_count += 1;
        }
    }

    let counts = [
        modified_count,
        added_count,
        file_changes.len() - modified_count - added_count,
    ];
    let names = ["modified", "added", "removed"];

    let mut first = true;
    for (i, count) in counts.iter().enumerate() {
        if *count > 0 {
            if first {
                print!(
                    "  {} {} {}",
                    count,
                    check_for_plural(String::from("file"), *count),
                    names[i]
                );
                first = false;
            } else {
                print!(", {} {}", count, names[i]);
            }
        }
    }
    println!("");
}

pub fn main() {
    let arguments = Arguments::parse();

    match arguments.commands {
        Commands::Init(command) => match init(&Path::new(&command.path).to_path_buf()) {
            Ok(commit_hash) => {
                println!("Initialized VCS repository in {}", command.path);
                println!("Created commit:");
                println!("[master {}] Initial commit", commit_hash);
            }

            Err(error) => {
                println!("Could not initialize VCS repository: {}", error);
            }
        },

        Commands::Status => match status() {
            Ok(status_result) => {
                if !status_result.file_changes.is_empty() {
                    println!("On branch {}", status_result.branch);
                    println!("Changes to be commited:");
                    print_file_changes(&status_result.file_changes);
                } else {
                    println!("No changes to be committed")
                }
            }

            Err(error) => {
                println!("Could not display status: {}", error);
            }
        },

        Commands::Commit(command) => match commit(&command.message) {
            Ok(commit_result) => {
                if commit_result.successful {
                    if !commit_result.file_changes.is_empty() {
                        println!(
                            "[{} {}] Work in progress",
                            commit_result.branch, commit_result.commit
                        );
                        print_changes_count(&commit_result.file_changes);
                        print_file_changes(&commit_result.file_changes);
                    } else {
                        println!("No changes to be committed");
                    }
                } else {
                    println!("You can create a new commit only from last one.\nAborting...");
                }
            }

            Err(error) => {
                println!("Could not create a new commit: {}", error);
            }
        },

        Commands::Jump(commands) => {
            let mut jump_result: JumpResult = JumpResult::NotFound;
            let mut is_error = false;
            if commands.commit.is_some() {
                let commit = commands.commit.as_ref().unwrap();
                match jump_commit(commit) {
                    Ok(result) => {
                        jump_result = result;
                    }
                    Err(error) => {
                        println!("Could not jump to commit {}: {}", commit, error);
                        is_error = true;
                    }
                }
            } else {
                let branch = commands.branch.as_ref().unwrap();
                match jump_branch(&branch) {
                    Ok(result) => {
                        jump_result = result;
                    }
                    Err(error) => {
                        println!("Could not jump to branch {}: {}", branch, error);
                        is_error = true;
                    }
                }
            }

            if !is_error {
                match jump_result {
                    JumpResult::UncommitedChanges { file_changes } => {
                        println!("Error: Your local changes to the following files should be commited or dropped:");
                        for file_change in file_changes {
                            println!(
                                "  {}",
                                file_change.1.into_os_string().into_string().unwrap()
                            );
                        }
                    }

                    JumpResult::Success { commit, branch } => {
                        println!(
                            "Successfully jumped to commit {}. Current branch: {}.",
                            commit, branch
                        );
                    }

                    JumpResult::NotFound => {
                        if commands.commit.is_some() {
                            println!(
                                "No commit with hash {} exists.\nAborting...",
                                commands.commit.unwrap()
                            );
                        } else {
                            println!("No branch {} exists.", commands.branch.unwrap());
                        }
                    }
                }
            }
        }

        Commands::NewBranch(command) => {
            let branch = command.name;
            match new_branch(&branch) {
                Ok(new_branch_result) => match new_branch_result {
                    NewBranchResult::OnlyFromMaster => {
                        println!("Creating a new branch is possible only when you are in the master branch.\nAborting...");
                    }

                    NewBranchResult::Success { commit } => {
                        println!(
                            "Created a new branch branch_name from master's commit {}",
                            commit
                        );
                    }

                    NewBranchResult::AlreadyExists => {
                        println!("Branch branch_name already exists.\nAborting...");
                    }
                },

                Err(error) => {
                    println!("Could not create branch {}: {}", branch, error);
                }
            }
        }

        Commands::Merge(command) => match merge(&command.branch) {
            Ok(merge_result) => match merge_result {
                MergeResult::NotLastCommit => {
                    println!("The merge is possible only when you are in the last commit in master.\nAborting...");
                }

                MergeResult::MergeConflict { path1, path2 } => {
                    println!("Merge confilict: file has been changed both in master and branch\n  {}\n  {}\nAborting...",
                    path1.into_os_string().into_string().unwrap(), path2.into_os_string().into_string().unwrap());
                }

                MergeResult::Success {
                    commit,
                    file_changes,
                } => {
                    println!(
                        "Successfully created merge commit:\n[master {}] Merged branch {}.",
                        commit, command.branch
                    );
                    print_changes_count(&file_changes);
                    print_file_changes(&file_changes);
                    println!("Deleted {}", command.branch);
                }

                MergeResult::MergeWithMaster => {
                    println!("The branch must be different from master");
                }
            },

            Err(error) => {
                println!("Could not merge with branch {}: {}", command.branch, error);
            }
        },

        Commands::Log => match log() {
            Ok(log_result) => {
                for (commit_data, file_changes) in log_result.commit_list {
                    println!("commit {}", commit_data.hash);
                    println!(
                        "Date {}",
                        DateTime::<Local>::to_rfc2822(&commit_data.date.into())
                    );
                    println!("Message {}", commit_data.message);
                    println!("Changes:");
                    if !file_changes.is_empty() {
                        print_file_changes(&file_changes);
                    } else {
                        println!("  No changes");
                    }
                }
            }

            Err(error) => {
                println!("Could not display logs: {}", error);
            }
        },
    }
}
