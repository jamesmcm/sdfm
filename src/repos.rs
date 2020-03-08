use anyhow::{anyhow, Result};
use dirs;
use git2::{Commit, Cred, ObjectType, PushOptions, RemoteCallbacks, Repository};
use std::env;
use std::fs;
use std::path::PathBuf;

fn ssh_path() -> PathBuf {
    //TODO: Allow specification of SSH key
    dirs::home_dir().unwrap().join(PathBuf::from(".ssh/id_rsa"))
}

// TODO: Make this work with branches
pub fn find_last_commit(repo: &Repository) -> Result<Commit> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    Ok(obj.into_commit().expect("Cannot find commit"))
}

fn ssh_key_has_passphrase() -> bool {
    let sshstring = fs::read_to_string(ssh_path())
        .expect(format!("Cannot read SSH key: {}", ssh_path().to_str().unwrap()).as_str());
    // PEM format
    if sshstring.contains("ENCRYPTED") {
        return true;
    }

    // TODO: Base64 encoded format - https://security.stackexchange.com/questions/129724/how-to-check-if-an-ssh-private-key-has-passphrase-or-not
    false
}

fn get_passphrase() -> Option<String> {
    let mut passphrase: Option<String> = None;
    if ssh_key_has_passphrase() {
        if let Ok(p) = env::var("SSH_PASSPHRASE") {
            passphrase = Some(p);
        } else {
            // TODO: Prompt for passphrase
            unimplemented!()
        }
    }
    passphrase
}

pub fn clone_repo_ssh(url: &str, path: &PathBuf) -> Result<Repository> {
    // Check if SSH key has passphrase
    // If so, prompt for passphrase
    let passphrase = get_passphrase();
    // Prepare callbacks.
    println!("{:?}", ssh_path());
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            &ssh_path(),
            passphrase.as_ref().map(|x| x.as_str()),
        )
    });

    // Prepare fetch options.
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clone the project.
    Ok(builder.clone(url, path)?)
}

pub fn push_repo_ssh(repo: &mut Repository, remote_name: &str, remote_branch: &str) -> Result<()> {
    // TODO: Use branch
    let mut origin = repo.find_remote(remote_name)?;
    let passphrase = get_passphrase();
    let mut push_options = PushOptions::new();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            &ssh_path(),
            passphrase.as_ref().map(|x| x.as_str()),
        )
    });
    callbacks.push_update_reference(|name, status| match status {
        None => {
            println!("ref: {:?} okay!", name);
            Ok(())
        }
        Some(status) => {
            println!("ref: {:?} - error status: {:?}", name, status);
            Err(git2::Error::from_str(
                format! {"ref: {:?} - error status: {:?}", name, status}.as_str(),
            ))
        }
    });

    // TODO: branches
    push_options.remote_callbacks(callbacks);
    origin.push::<&str>(
        &["refs/heads/master:refs/heads/master"],
        Some(&mut push_options),
    )?;
    Ok(())
}
