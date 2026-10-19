# gscan

A small Rust command-line tool for surveying local Git repositories.

## First milestone

Scan immediate subdirectories of a supplied directory and show each repository's:

- Name and current branch, including detached HEAD.
- Number of changed files, including untracked files.
- Commits ahead of and behind its locally known upstream, when configured.

Use the installed Git command and Rust standard library. Report inspection errors
clearly and handle repositories without commits or an upstream. Remote status
uses locally cached refs; scanning does not automatically fetch.

Planned usage:

```sh
gscan /srv/dev/projects
```

Possible later additions: `--dirty`, JSON output, and explicit `--fetch`.

## Development

Requires stable Rust and Git. This repository currently contains the Cargo starter
program; repository scanning is not implemented yet.

```sh
cargo run
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
```
