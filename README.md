# git-link
`git link` alias to fetch a link to a remote git repo

## Usage

1. `cargo install git-link`
    * Explanation: Any `git-xxx123` command/binary available in path is available as a git subcommand like `git xxx123`.
    * This cargo install adds command `git-link`, available by alias `git link`.

2. Use any of the following commands inside a git repo:

```bash
# Print out the git repo's remote URL (even if the remote is via SSH)
git link

# Open the URL
git link --open
git link -o

# Print a link to a new PR for the current branch
git link pr

# Open the PR for the current branch
git link pr --open
git link pr --o
```
