#![feature(drain_filter)]

// i3
// i3status
// mpd
// ssh.conf
// weechat irc.conf
// weechat weechat.conf
// vimrc
// zshrc
// bashrc
// xprofile
// xinitrc
// Xresources
// profile
use dirs;
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Application {
    pub name: String,
    pub dotfiles: Vec<DotFile>,
}

#[derive(Deserialize)]
pub struct DotFile {
    pub name: String,
    possible_paths: Vec<PathBuf>,
    // TODO: Check for diff with template possible_template_paths: Option<Vec<PathBuf>>,
    pub actual_path: Option<PathBuf>,
}

fn default_dotfiles() -> Vec<Application> {
    let i3 = Application {
        name: String::from("i3"),
        dotfiles: vec![DotFile {
            name: String::from("config"),
            possible_paths: vec![
                dirs::config_dir().unwrap().join("i3/config"),
                dirs::home_dir().unwrap().join("/.i3/config"),
                PathBuf::from("/etc/i3/config"),
            ],
            actual_path: None,
        }],
    };

    vec![i3]
}

pub fn get_found_dotfiles() -> Vec<Application> {
    let mut default_apps = default_dotfiles();

    for app in &mut default_apps {
        for dotfile in &mut app.dotfiles {
            for path in &dotfile.possible_paths {
                if path.exists() {
                    dotfile.actual_path = Some(path.clone());
                    break;
                }
            }
        }
    }

    // If actual_path is None then remove dotfile
    for app in &mut default_apps {
        app.dotfiles.drain_filter(|x| x.actual_path.is_none());
    }

    // If dotfiles is empty, remove application
    default_apps.drain_filter(|x| x.dotfiles.is_empty());

    default_apps
}
