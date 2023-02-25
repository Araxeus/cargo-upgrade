# Cargo Upgrade

This tool is designed to help you upgrade your globally installed cargo binaries to the latest version.

## Installation

```bash
cargo install cargo-upgrade-command
```

## Available Commands

### `cargo upgrade`

This command will upgrade all of your globally installed cargo binaries to the latest version.

![upgrade.gif](https://github.com/Araxeus/cargo-upgrade/raw/master/assets/upgrade.gif "Upgrade command")

### `cargo upgrade --outdated`

This command will show you if any of your globally installed cargo binaries are out of date.

alias: `cargo upgrade -o` or `cargo upgrade o` or `cargo upgrade list`

![outdated.gif](https://github.com/Araxeus/cargo-upgrade/raw/master/assets/outdated.gif "--outdated command")

### How it works

To get outdated binaries, it will parse `cargo install --list` and then compare the version of each crate with the version in `cargo search <crate> --limit=1`.

Then if upgrading, it will run `cargo install --locked <crate>`.
