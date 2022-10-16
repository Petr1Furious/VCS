use sha1::{Digest, Sha1};
use std::{
    env,
    fs::{self, File},
    io::{BufReader, Error, Read},
    path::{Component, PathBuf},
};
use walkdir::WalkDir;

#[derive(PartialEq)]
pub enum FileChange {
    Equal,
    Added,
    Modified,
    Removed,
}

fn hash(data_to_hash: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data_to_hash);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn get_repo_dir() -> Result<PathBuf, std::io::Error> {
    let mut cur = env::current_dir().unwrap();
    while !cur.join(".vcs").as_path().exists() {
        if !cur.pop() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not a VCS repository",
            ));
        }
    }
    Ok(cur)
}

pub fn get_contents(path: &PathBuf, ignore_vcs: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut contents: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(path) {
        match entry {
            Ok(dir_entry) => {
                if !ignore_vcs
                    || dir_entry
                        .path()
                        .components()
                        .find(|x| *x == Component::Normal(".vcs".as_ref()))
                        .is_none()
                {
                    contents.push(dir_entry.path().to_path_buf());
                }
            }
            Err(error) => {
                return Err(Error::new(
                    error.io_error().unwrap().kind(),
                    error.to_string(),
                ));
            }
        }
    }

    Ok(contents)
}

fn add_to_hash(cur_hash: &String, to_add: &[u8]) -> String {
    hash([cur_hash.as_bytes(), to_add].concat().as_slice())
}

pub fn get_contents_hash(contents: &Vec<PathBuf>) -> Result<String, std::io::Error> {
    let mut sorted_contents = contents.clone();
    sorted_contents.sort();

    let mut hash: String = String::from("lol");
    for entry in sorted_contents {
        hash = add_to_hash(&hash, format!("{:?}", entry).as_bytes());
        if entry.is_file() {
            hash = add_to_hash(&hash, &fs::read(entry)?);
        }
    }
    Ok(hash)
}

pub fn copy_files_to_commit(
    repo_dir: &PathBuf,
    contents: &Vec<PathBuf>,
    commit: &String,
) -> Result<(), std::io::Error> {
    let commit_dir = (&repo_dir).join(".vcs").join("commits").join(commit);
    for entry in contents {
        let new_path = commit_dir.join(entry.strip_prefix(&repo_dir).unwrap());
        if entry.is_file() {
            fs::copy(&entry, new_path)?;
        } else {
            fs::create_dir(new_path)?;
        }
    }
    Ok(())
}

pub fn remove_repo_files(repo_dir: &PathBuf) -> Result<(), std::io::Error> {
    let mut contents = get_contents(repo_dir, true)?;
    contents = contents[1..].to_vec();
    contents.reverse();
    for entry in contents {
        if entry.strip_prefix(repo_dir).is_err() {
            panic!("Tried to remove file outside of VCS repository");
        }
        if entry.is_file() {
            fs::remove_file(entry)?;
        } else {
            fs::remove_dir(entry)?;
        }
    }
    Ok(())
}

pub fn copy_files_from_commit(repo_dir: &PathBuf, commit: &String) -> Result<(), std::io::Error> {
    let commit_dir = (&repo_dir).join(".vcs").join("commits").join(commit);
    let commit_contents = get_contents(&commit_dir, false)?[1..].to_vec();

    for entry in commit_contents.iter() {
        let new_path = repo_dir.join(entry.strip_prefix(&commit_dir).unwrap());
        if entry.is_file() {
            fs::copy(&entry, new_path)?;
        } else {
            fs::create_dir(new_path)?;
        }
    }
    Ok(())
}

pub fn files_equal(path1: &PathBuf, path2: &PathBuf) -> Result<bool, std::io::Error> {
    let file1 = File::open(path1)?;
    let file2 = File::open(path2)?;
    let mut reader1 = BufReader::new(file1);
    let mut reader2 = BufReader::new(file2);

    let mut buf1 = [0; 10000];
    let mut buf2 = [0; 10000];
    loop {
        if let Result::Ok(n1) = reader1.read(&mut buf1) {
            if n1 > 0 {
                if let Result::Ok(n2) = reader2.read(&mut buf2) {
                    if n1 == n2 {
                        if buf1 == buf2 {
                            continue;
                        }
                    }
                    return Ok(false);
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(true)
}

pub fn get_file_changes(
    dir: &PathBuf,
    contents: &Vec<PathBuf>,
    other_dir: &PathBuf,
    relative_to: &Vec<PathBuf>,
) -> Result<Vec<(FileChange, PathBuf)>, std::io::Error> {
    let mut used_relative_to: Vec<(&PathBuf, bool)> = Vec::new();
    for entry in relative_to.iter() {
        used_relative_to.push((entry, false));
    }

    let mut res: Vec<(FileChange, PathBuf)> = Vec::new();
    for entry in contents.iter() {
        let mut cur_change = FileChange::Added;
        let relative1 = entry.strip_prefix(dir).unwrap();
        for commit_entry in used_relative_to.iter_mut() {
            let relative2 = commit_entry.0.strip_prefix(&other_dir).unwrap();
            if relative1 == relative2 {
                commit_entry.1 = true;
                if files_equal(entry, commit_entry.0)? {
                    cur_change = FileChange::Equal;
                } else {
                    cur_change = FileChange::Modified;
                }
                break;
            }
        }
        if cur_change != FileChange::Equal {
            res.push((cur_change, entry.clone()));
        }
    }

    for commit_entry in used_relative_to.iter() {
        if !commit_entry.1 {
            res.push((
                FileChange::Removed,
                dir.join(commit_entry.0.strip_prefix(&other_dir).unwrap()),
            ));
        }
    }

    Ok(res)
}
