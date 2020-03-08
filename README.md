# sdfm - Simple Dotfile Manager
sdfm is a Simple Dotfile Manager, written in Rust.

It uses a git repo to sync your chosen dotfiles, with different devices using different branches but able to sync with the master branch (i.e. to push and pull some changes upstream whilst maintaining device-specific differences for resolution settings, etc.)


## Current status

`sdfm init` works to upload the current state of your dotfiles to a repo with SSH authentication. The SSH passphrase should be passed in the `SSH_PASSPHRASE` environment variable:

```
SSH_PASSPHRASE="password" ./sdfm init https://gitlab.com/username/sdfm-dotfiles.git -t master
```

Currently the only default dotfiles scanned for is the i3 config.

## TODO:

* Make `sdfm init` work for both empty (no prior commits) and non-empty repos.
* Implement `sdfm pull` to set up sdfm using an existing sdfm dotfiles repo, on a new machine (i.e. to copy your dotfiles from another machine).
* Make `sdfm init` and `sdfm pull` correctly obey base and target branch arguments.
* Implement `sdfm sync` to sync locally modified dotfiles with target branch (and merge with upstream branch if requested).
* Do not directly write to dotfiles in case of merge conflicts, but force resolution by user first.
* Add option to `sdfm sync` to only merge in changes without writing to remote branches.
* Implement `sdfm edit` to edit list of dotfiles being tracked in current branch.
* Add logging, enabled with verbose flag.
* Add headless argument to `sdfm init`, to not trigger interactive edits with `$EDITOR`. This should be mandatory if not running in TTY.
* Add argument to `sdfm init` to provide dotfiles list to be tracked, for headless mode.
* Add context to error messages.
* Add cleanup when in error state.
* Add `sdfm clean` command to remove current sdfm configuration.


