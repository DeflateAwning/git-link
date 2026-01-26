use std::process::{Command, exit};

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
    // HTTPS form: https://host/org/repo(.git)
    if let Some(rest) = remote.strip_prefix("https://") {
        return format!(
            "https://{}",
            rest.strip_suffix(".git").unwrap_or(rest)
        );
    }

    // SSH form: git@host:org/repo(.git)
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


fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut open = false;
    let mut pr = false;

    for arg in &args {
        match arg.as_str() {
            "--open" => open = true,
            "pr" => pr = true,
            _ => {
                eprintln!("Unknown argument: {}", arg);
                exit(1);
            }
        }
    }

    let remote = run("git", &["config", "--get", "remote.origin.url"]);
    if remote.is_empty() {
        eprintln!("No origin remote found");
        exit(1);
    }

    let repo_url = normalize_remote(&remote);

    let final_url = if pr {
        let branch = run("git", &["symbolic-ref", "--short", "HEAD"]);
        if branch.is_empty() {
            eprintln!("Not on a branch");
            exit(1);
        }
        format!("{}/pull/new/{}", repo_url, branch)
    } else {
        repo_url
    };

    // Always print
    println!("{}", final_url);

    // Optionally open
    if open {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };

        let _ = Command::new(opener).arg(&final_url).status();
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

