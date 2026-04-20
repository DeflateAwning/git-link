use clap::{Parser, Subcommand};
use std::process::{Command, exit};

/// Generate links for the current Git repository.
#[derive(Parser)]
#[command(name = "git-link")]
#[command(author, version, about)]
struct Cli {
    /// Open the URL in the browser.
    #[arg(short = 'o', long, global = true)]
    open: bool,

    /// Show verbose output.
    #[arg(short = 'v', long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Link to a pull request / merge request for the current branch
    Pr,
    /// Link to a merge request / pull request for the current branch
    Mr,
}

#[derive(Debug)]
pub enum RemoteFlavor {
    Github,
    Gitlab,
    Codeberg,
}

fn run_shell_cmd(cmd: &str, args: &[&str], verbose: bool) -> String {
    if verbose {
        println!("Running shell command: {} {:?}", cmd, args);
    }

    let output = Command::new(cmd).args(args).output().unwrap_or_else(|_| {
        eprintln!("Failed to run {}", cmd);
        exit(1);
    });

    if !output.status.success() {
        eprintln!(
            "Command failed (exit code {}): {} {:?}",
            output.status, cmd, args
        );
        exit(1);
    }

    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if verbose {
        println!(
            "Command output (exit code {}): {}",
            output.status, output_str
        );
    }

    output_str
}

/// Normalize a remote URL (SSH, HTTPS) to a standard HTTPS format.
pub fn normalize_remote(remote: &str) -> String {
    let url: String = {
        if let Some(rest) =
            (remote.strip_prefix("https://")).or_else(|| remote.strip_prefix("http://"))
        {
            // HTTPS: https://host/org/repo(.git).
            // Note: Always upgrade http to https for security.
            format!("https://{rest}")
        } else if let Some(rest) = (remote.strip_prefix("ssh://git@"))
            .or_else(|| remote.strip_prefix("git@"))
            .or_else(|| remote.strip_prefix("git+ssh://"))
            .or_else(|| remote.strip_prefix("ssh+git://"))
        {
            // SSH: git@host:org/repo(.git)
            let mut parts = rest.splitn(2, ':');

            let host = parts.next().unwrap();
            let path = parts.next().unwrap_or("");

            format!("https://{}/{}", host, path)
        } else if let Some(rest) = remote.strip_prefix("git://") {
            format!("https://{}", rest)
        } else {
            panic!("Unrecognized remote URL format: {}", remote);
        }
    };

    let url = url.strip_suffix(".git").unwrap_or(&url).to_string();
    let url = url.strip_suffix("/").unwrap_or(&url).to_string();
    let url = url.strip_suffix(".git").unwrap_or(&url).to_string();
    url.strip_suffix("/").unwrap_or(&url).to_string()
}

/// Extract the domain from an HTTP(S) repository URL.
///
/// On invalid input, returns the original string.
pub fn extract_repo_domain(repo_url: &str) -> String {
    let url = url::Url::parse(repo_url).unwrap();
    url.host_str().unwrap_or(repo_url).to_string()
}

pub fn detect_remote_flavor(repo_url: &str) -> Option<RemoteFlavor> {
    let repo_url_domain = extract_repo_domain(repo_url).to_lowercase();

    if repo_url_domain.contains("github") {
        Some(RemoteFlavor::Github)
    } else if repo_url_domain.contains("gitlab") {
        // TODO: There may be a better way to detect self-hosted GitLab repos.
        Some(RemoteFlavor::Gitlab)
    } else if repo_url_domain.contains("codeberg") {
        Some(RemoteFlavor::Codeberg)
    } else {
        // Unknown remote flavor.
        None
    }
}

/// Create a Pull Request url, assuming the remote is GitHub, or a URL-compatible site.
pub fn github_pr_url(repo_url: &str, branch: &str) -> String {
    format!("{}/pull/new/{}", repo_url, branch)
}

/// Create a Merge Request url, assuming the remote is GitLab, or a URL-compatible site.
pub fn gitlab_mr_url(repo_url: &str, branch: &str) -> String {
    format!(
        "{}/-/merge_requests/new?merge_request[source_branch]={}",
        repo_url, branch
    )
}

/// Create a Pull Request url, assuming the remote is Codeberg, or a URL-compatible site.
///
/// This is the closest to "new PR" which exists on Codeberg.
pub fn codeberg_compare_url(repo_url: &str, branch: &str, default_branch: &str) -> String {
    format!("{repo_url}/compare/{default_branch}...{branch}")
}

/// Create a Pull Request or Merge Request url, depending on the remote type.
///
/// If the remote type is not recognized, the repo URL is returned as-is.
pub fn link_for_pr_or_mr(repo_url: &str, branch: &str, verbose: bool) -> String {
    let flavor = detect_remote_flavor(repo_url);

    if verbose {
        println!("Detected remote flavor: {:?}", flavor);
    }

    match flavor {
        Some(RemoteFlavor::Github) => github_pr_url(repo_url, branch),
        Some(RemoteFlavor::Gitlab) => gitlab_mr_url(repo_url, branch),
        Some(RemoteFlavor::Codeberg) => {
            // TODO: Detect default branch better.
            codeberg_compare_url(repo_url, branch, "main")
        }
        None => repo_url.to_string(),
    }
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

    let remote = run_shell_cmd(
        "git",
        &["config", "--get", "remote.origin.url"],
        cli.verbose,
    );
    if remote.is_empty() {
        eprintln!("No origin remote found");
        exit(1);
    }

    let repo_url = normalize_remote(&remote);

    if cli.verbose {
        println!("Repo URL: {}", repo_url);
    }

    let final_url = match cli.command {
        Some(Commands::Pr | Commands::Mr) => {
            let branch = run_shell_cmd("git", &["symbolic-ref", "--short", "HEAD"], cli.verbose);
            if branch.is_empty() {
                eprintln!("Not on a branch");
                exit(1);
            }

            if cli.verbose {
                println!("Branch: {}", branch);
            }

            link_for_pr_or_mr(&repo_url, &branch, cli.verbose)
        }
        None => repo_url,
    };

    // Always print (whether or not opening in a browser).
    println!("{}", final_url);

    // Optionally open in web browser.
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
    fn ssh_remote_with_git_suffix_with_ssh_prefix() {
        // This is the Codeberg style.
        let input = "ssh://git@example.com:org/project.git";
        let expected = "https://example.com/org/project";
        assert_eq!(normalize_remote(input), expected);
    }

    #[test]
    fn ssh_remote_without_git_suffix_with_ssh_prefix() {
        // This is similar to the Codeberg style.
        let input = "ssh://git@example.com:org/project";
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
        assert_eq!(github_pr_url(repo_url, branch), expected);
    }

    #[test]
    #[should_panic(expected = "Unrecognized remote URL format")]
    fn invalid_remote_panics() {
        normalize_remote("ssh://example.com/org/project.git");
    }

    #[test]
    fn test_github_pr_url() {
        let repo = "https://github.com/org/project";
        let branch = "feature-x";
        let expected = "https://github.com/org/project/pull/new/feature-x";
        assert_eq!(link_for_pr_or_mr(repo, branch, true), expected);
    }

    #[test]
    fn test_codeberg_pr_url() {
        let repo = "https://codeberg.org/org/project";
        let branch = "feature-x";
        let expected = "https://codeberg.org/org/project/compare/main...feature-x";
        assert_eq!(link_for_pr_or_mr(repo, branch, true), expected);
    }

    #[test]
    fn test_gitlab_mr_url() {
        let repo = "https://gitlab.com/org/project";
        let branch = "feature-x";
        let expected = "https://gitlab.com/org/project/-/merge_requests/new?merge_request[source_branch]=feature-x";
        assert_eq!(link_for_pr_or_mr(repo, branch, true), expected);
    }

    #[test]
    fn test_self_hosted_gitlab_mr_url() {
        let repo = "https://gitlab.example.com/org/project";
        let branch = "dev";
        let expected = "https://gitlab.example.com/org/project/-/merge_requests/new?merge_request[source_branch]=dev";
        assert_eq!(link_for_pr_or_mr(repo, branch, true), expected);
    }
}
