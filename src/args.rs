use anyhow::{anyhow, Result};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "sdfm", about = "Simple Dotfile Manager")]
pub struct App {
    /// Verbose output
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(name = "init", about = "Initialise sdfm config for given repo")]
    Init(InitCommand),

    #[structopt(name = "sync", about = "Sync current dotfiles with dotfile repo")]
    Merge(MergeCommand), // Named Merge to avoid confusion with Sync trait

    #[structopt(
        name = "pull",
        about = "Pull from a dotfile repo to this device and set target branch"
    )]
    Pull(PullCommand),
}

pub enum RepoURL {
    SSH(String),
    HTTPS(String),
}

impl RepoURL {
    fn parse(url: &str) -> Result<Self, anyhow::Error> {
        match url {
            // TODO: Use &str instead of String since we never mutate this
            url if url.starts_with("git@") => Ok(Self::SSH(String::from(url))),
            url if url.starts_with("https://") => Ok(Self::HTTPS(String::from(url))),
            _ => Err(anyhow!(
                "Invalid repo URL format (neither SSH nor HTTPS): {}",
                url
            )),
        }
    }
}

/// Used to get URL string for serialisation in config
impl ToString for RepoURL {
    fn to_string(&self) -> String {
        match self {
            Self::SSH(url) => url.clone(),
            Self::HTTPS(url) => url.clone(),
        }
    }
}

#[derive(StructOpt)]
pub struct InitCommand {
    /// URL of repo - either HTTPS or SSH
    #[structopt(parse(try_from_str = RepoURL::parse))]
    pub url: RepoURL,

    /// Target branch for repo, should be master/default branch for new repos
    #[structopt(short = "t", long = "target-branch")]
    pub target_branch: String,
    // TODO: Allow specification of SSH keys directly for SSH
    // TODO: Allow to pass HTTPS creds as env var for HTTPS
    // TODO: Allow to pass list of dotfiles to track directly instead of interactive scan
    // TODO: Allow specification of config dir
}

#[derive(StructOpt)]
pub struct MergeCommand {
    /// Skip diff checking
    #[structopt(short = "s", long = "skip-diff")]
    pub skipdiff: bool,

    /// Force push (ignore merge conflicts with target branch)
    #[structopt(short = "f", long = "force")]
    pub force: bool,

    /// Target branch (default: this device branch)
    #[structopt(default_value, short = "t", long = "target-branch")]
    pub target_branch: String,
}

#[derive(StructOpt)]
pub struct PullCommand {
    /// Skip diff checking
    #[structopt(short = "s", long = "skip-diff")]
    pub skipdiff: bool,

    /// Force push (ignore merge conflicts with target branch)
    #[structopt(short = "f", long = "force")]
    pub force: bool,

    /// Target branch (default: this device branch)
    #[structopt(default_value, short = "t", long = "target-branch")]
    pub target_branch: String,
}
