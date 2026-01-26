use std::process::{Command, exit};
use clap::{Parser, Subcommand};

/// Generate links for the current Git repository
#[derive(Parser)]
#[command(name = "git-link")]
#[command(author, version, about)]
struct Cli {
    /// Open the URL in the browser
    #[arg(short = 'o', long, global = true)]
    open: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Link to a pull request for the current branch
    Pr,
}

fn run(cmd: &str, args: &[&str]) -> String {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .unwrap_or_else(|_| {
            eprintln!("Failed to run {}", cmd);
            exit(1);
        });

    if !output.status.success() {
        eprintln!("Command failed: {} {:?}", cmd, args);
        exit(1);
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn normalize_remote(remote: &str) -> String {
    // HTTPS: https://host/org/repo(.git)
    if let Some(rest) = remote.strip_prefix("https://") {
        return format!(
            "https://{}",
            rest.strip_suffix(".git").unwrap_or(rest)
        );
    }

    // SSH: git@host:org/repo(.git)
    if let Some(rest) = remote.strip_prefix("git@") {
        let rest = rest.strip_suffix(".git").unwrap_or(rest);
        let mut parts = rest.splitn(2, ':');

        let host = parts.next().unwrap();
        let path = parts.next().unwrap_or("");

        return format!("https://{}/{}", host, path);
    }

    panic!("Unrecognized remote URL format: {}", remote);
}

pub fn pr_url(repo_url: &str, branch: &str) -> String {
    format!("{}/pull/new/{}", repo_url, branch)
}

fn open_in_browser(url: &str) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    let _ = Command::new(opener).arg(url).status();
}

fn main() {
    let cli = Cli::parse();

    let remote = run("git", &["config", "--get", "remote.origin.url"]);
    if remote.is_empty() {
        eprintln!("No origin remote found");
        exit(1);
    }

    let repo_url = normalize_remote(&remote);

    let final_url = match cli.command {
        Some(Commands::Pr) => {
            let branch = run("git", &["symbolic-ref", "--short", "HEAD"]);
            if branch.is_empty() {
                eprintln!("Not on a branch");
                exit(1);
            }
            pr_url(&repo_url, &branch)
        }
        None => repo_url,
    };

    // Always print
    println!("{}", final_url);

    // Optionally open
    if cli.open {
        open_in_browser(&final_url);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_remote_with_git_suffix() {
        let input = "git@example.com:org/project.git";
        let expected = "https://example.com/org/project";
        assert_eq!(normalize_remote(input), expected);
    }

    #[test]
    fn ssh_remote_without_git_suffix() {
        let input = "git@example.com:org/project";
        let expected = "https://example.com/org/project";
        assert_eq!(normalize_remote(input), expected);
    }

    #[test]
    fn https_remote_with_git_suffix() {
        let input = "https://example.com/org/project.git";
        let expected = "https://example.com/org/project";
        assert_eq!(normalize_remote(input), expected);
    }

    #[test]
    fn https_remote_without_git_suffix() {
        let input = "https://example.com/org/project";
        let expected = "https://example.com/org/project";
        assert_eq!(normalize_remote(input), expected);
    }

    #[test]
    fn pr_url_is_constructed_correctly() {
        let repo_url = "https://example.com/org/project";
        let branch = "feature-branch";
        let expected = "https://example.com/org/project/pull/new/feature-branch";
        assert_eq!(pr_url(repo_url, branch), expected);
    }

    #[test]
    #[should_panic(expected = "Unrecognized remote URL format")]
    fn invalid_remote_panics() {
        normalize_remote("ssh://example.com/org/project.git");
    }
}

