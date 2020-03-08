#![feature(drain_filter)]

mod args;
mod dotfiles;
mod init;
mod merge;
mod pull;
mod repos;

use structopt::StructOpt;

fn main() {
    let app = args::App::from_args();

    let result = match app.cmd {
        args::Command::Init(command) => init::init(command),
        args::Command::Merge(command) => merge::merge(command),
        args::Command::Pull(command) => pull::pull(command),
    };

    println!("{:?}", result);
}
