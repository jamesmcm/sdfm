use crate::args::InitCommand;
use crate::args::RepoURL;
use crate::dotfiles::get_found_dotfiles;
use crate::repos;
use anyhow::Result;
use serde_derive::Serialize;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use dirs;
use git2::{IndexAddOption, PushOptions, Signature};
use xdg;

#[derive(Serialize)]
struct Config {
    repo: String,
    target_branch: String,
}

pub fn init(command: InitCommand) -> Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("sdfm").unwrap();

    // Clone repo to $XDG_CONFIG_HOME/sdfm/repo
    let mut repo_path = xdg_dirs.get_config_home();
    repo_path.push("repo");

    // TODO: Fix HTTPS repos with creds
    let mut repo = match &command.url {
        RepoURL::HTTPS(url) => unimplemented!(),
        RepoURL::SSH(url) => repos::clone_repo_ssh(&url, &repo_path)?,
    };

    // TODO: Check whether repo is empty
    // TODO: Check that repo is valid sdfm repo

    // Write config TOML
    let config_path = xdg_dirs.place_config_file("config.toml")?;
    {
        let mut config_file = File::create(config_path)?;

        let config = Config {
            repo: command.url.to_string(),
            target_branch: command.target_branch,
        };
        let toml = toml::to_string(&config)?;
        config_file.write_all(toml.as_bytes())?;
    }
    // Check for default dotfiles
    let dotfiles_path = xdg_dirs.place_config_file("dotfiles")?;
    {
        let found_dotfiles = get_found_dotfiles();
        let mut dotfiles_list = File::create(&dotfiles_path)?;
        for application in found_dotfiles {
            dotfiles_list.write_all(format!("# {}\n", application.name).as_bytes())?;
            for dotfile in application.dotfiles {
                dotfiles_list.write_all(format!("## {}\n", dotfile.name).as_bytes())?;
                dotfiles_list.write_all(
                    format!("{}\n", dotfile.actual_path.unwrap().to_str().unwrap()).as_bytes(),
                )?;
            }
        }
    }

    // Open $EDITOR so user can edit tracked dotfiles
    if let Ok(editor) = env::var("EDITOR") {
        Command::new(editor)
            .arg(dotfiles_path.to_str().unwrap())
            .status()
            .unwrap();
    } else {
        // TODO: Check for installed editor here
        Command::new("nano")
            .arg(dotfiles_path.to_str().unwrap())
            .status()
            .unwrap();
    }

    // Create hard links on system in repo
    let dotfiles_list = File::open(dotfiles_path)?;
    for line in BufReader::new(dotfiles_list).lines() {
        // TODO: Remove unwrap here
        let line = line.unwrap();
        if line.starts_with('#') {
            // Comment
            continue;
        }

        // TODO: Refactor this - DRY
        if line.starts_with(dirs::config_dir().unwrap().to_str().unwrap()) {
            // in $XDG_CONFIG_HOME - write to repo/xdg_config/
            let outpath = repo_path.join("xdg_config").join(
                PathBuf::from(&line)
                    .strip_prefix(dirs::config_dir().unwrap())
                    .unwrap(),
            );
            let mut dirpath = outpath.clone();
            dirpath.pop();
            let mut mkdir = Command::new("mkdir").arg("-p").arg(&dirpath).spawn()?;
            mkdir.wait()?;

            let mut ln = Command::new("ln").arg(&line).arg(&outpath).spawn()?;
            ln.wait()?;
            break;
        }

        if line.starts_with(dirs::home_dir().unwrap().to_str().unwrap()) {
            // in homedir - write to repo/homedir/
            let outpath = repo_path.join("homedir").join(
                PathBuf::from(&line)
                    .strip_prefix(dirs::home_dir().unwrap())
                    .unwrap(),
            );
            let mut dirpath = outpath.clone();
            dirpath.pop();
            let mut mkdir = Command::new("mkdir").arg("-p").arg(&dirpath).spawn()?;
            mkdir.wait()?;

            let mut ln = Command::new("ln").arg(&line).arg(&outpath).spawn()?;
            ln.wait()?;
            break;
        } else {
            // Absolute path i.e. /etc/i3/config - copy to repo/root/...
            let outpath = repo_path.join("root").join(PathBuf::from(&line));
            let mut dirpath = outpath.clone();
            dirpath.pop();

            let mut mkdir = Command::new("mkdir").arg("-p").arg(&dirpath).spawn()?;
            mkdir.wait()?;

            let mut ln = Command::new("ln").arg(&line).arg(&outpath).spawn()?;
            ln.wait()?;
        }
    }

    // Add files to git repo
    let mut index = repo.index().expect("cannot get the Index file");
    env::set_current_dir(&repo_path)?;
    println!("Repo path: {:?}", repo_path);

    index.add_all(
        ["*"].iter(),
        IndexAddOption::DEFAULT,
        Some(&mut |file, _name| {
            println!("Adding file: {:?}", file);
            0
        }),
    )?;
    index.write()?;
    // Commit changes
    let oid = index.write_tree()?;
    let signature = Signature::now("sdfm bot", "sdfm")?;
    {
        // TODO: Check if no prior commits
        let parent_commit = repos::find_last_commit(&repo)?;
        let tree = repo.find_tree(oid)?;
        repo.commit(
            Some("HEAD"),      //  point HEAD to our new commit
            &signature,        // author
            &signature,        // committer
            "sdfm commit",     // TODO: Improve commit message
            &tree,             // tree
            &[&parent_commit], // TODO: Handle empty repo
        )?; // parents
    }
    // TODO: Push repo
    repos::push_repo_ssh(&mut repo, "origin", "master")?;
    Ok(())
}
