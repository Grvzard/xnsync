use clap::Parser;
use git2::{FetchOptions, FetchPrune, Repository};
use pathdiff::diff_paths;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Config {
    src_dir: PathBuf,

    dst_dir: PathBuf,
}

fn check_path_is_dir(d: &PathBuf) -> io::Result<()> {
    if !d.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a directory",
        ));
    }
    Ok(())
}

fn contains_git_dir(p: &PathBuf) -> bool {
    p.join(".git").exists() && p.join(".git").is_dir()
}

fn main() -> io::Result<()> {
    let config = Config::parse();
    if !config.src_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "source directory does not exist",
        ));
    }
    check_path_is_dir(&config.src_dir)?;
    if config.dst_dir.exists() {
        check_path_is_dir(&config.dst_dir)?;
    } else {
        fs::create_dir(&config.dst_dir)?;
    }

    let pre_list: Vec<PathBuf> = config
        .src_dir
        .read_dir()?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|en| en.path())
        .filter(contains_git_dir)
        .map(|p| diff_paths(p, &config.src_dir).unwrap())
        .collect();

    let mut dst_repos: Vec<Repository> = vec![];
    for repo_name in pre_list.iter() {
        let src_git_path = config.src_dir.join(repo_name);
        let dst_git_path = config.dst_dir.join(repo_name);
        if dst_git_path.exists() && dst_git_path.is_dir() && contains_git_dir(&dst_git_path) {
            let repo = Repository::open(&dst_git_path).unwrap();
            dst_repos.push(repo);
            continue;
        }
        match Repository::clone(src_git_path.to_str().unwrap(), &dst_git_path) {
            Ok(repo) => dst_repos.push(repo),
            Err(e) => {
                eprintln!(
                    "Error cloning repo({:?}): {}",
                    &repo_name.to_str().unwrap(),
                    e
                );
                continue;
            }
        }
    }

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.prune(FetchPrune::On);
    for repo in dst_repos.iter() {
        let mut remote = repo.find_remote("origin").unwrap();
        remote.fetch(&[""], Some(&mut fetch_opts), None).unwrap();
    }

    Ok(())
}
