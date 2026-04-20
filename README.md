# git-link
`git link` alias to fetch a link to a remote git repo

## Usage

```bash
# Install:
cargo install git-link

# Print out the git repo's remote URL (even if the remote is via SSH).
git link

# Open the URL.
git link --open
git link -o

# Print a link to a new PR for the current branch.
git link pr

# Open the PR for the current branch.
git link pr --open
git link pr --o
```

## Explanation

Explanation: Any `git-xyz` command/binary available in path is available as a git subcommand like `git xyz`.

`cargo install git-link` adds the command `git-link`, available by alias `git link`.

## Features

* Proven support for GitHub, GitLab, and Codeberg.
* Get a link for the repo's home page.
* Get a link for a new PR for the current branch.
* Open links with the `-o` flag.
