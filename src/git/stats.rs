use chrono::{DateTime, Local};
use git2::Repository;
use std::collections::HashMap;
use std::path::PathBuf;

/// Information about a detected commit
#[derive(Debug, Clone)]
pub struct DetectedCommit {
    /// Git commit hash
    pub commit_hash: String,
    /// Project identifier (remote URL or absolute path)
    pub project_id: String,
    /// Project display name (folder name)
    pub project_name: String,
}

/// Statistics about git activity (display purposes)
#[derive(Debug, Clone, Default)]
pub struct GitStats {
    /// Whether we're in a git repository
    pub in_git_repo: bool,
    /// Number of repositories being tracked
    pub repo_count: usize,
    /// Names of all tracked repositories
    pub repo_names: Vec<String>,
}

/// Format a datetime as a human-readable "X ago" string
pub fn format_time_ago(time: Option<DateTime<Local>>) -> String {
    match time {
        Some(time) => {
            let now = Local::now();
            let duration = now.signed_duration_since(time);

            if duration.num_seconds() < 60 {
                "just now".to_string()
            } else if duration.num_minutes() < 60 {
                let mins = duration.num_minutes();
                format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
            } else if duration.num_hours() < 24 {
                let hours = duration.num_hours();
                format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
            } else {
                let days = duration.num_days();
                format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
            }
        }
        None => "never".to_string(),
    }
}

/// Tracks git repository activity across one or more repositories
pub struct GitTracker {
    /// All tracked repositories
    repos: Vec<Repository>,
    /// Last known HEAD commit hash per repository (keyed by repo path)
    last_heads: HashMap<PathBuf, String>,
}

impl GitTracker {
    /// Create a new git tracker for the current directory
    /// If in a git repo, tracks just that repo
    /// If not, scans immediate subdirectories for git repos
    pub fn new() -> Self {
        let repos = Self::discover_repos();
        let mut last_heads = HashMap::new();

        for repo in &repos {
            if let Some(head) = repo
                .head()
                .ok()
                .and_then(|r| r.target())
                .map(|oid| oid.to_string())
            {
                last_heads.insert(repo.path().to_path_buf(), head);
            }
        }

        Self { repos, last_heads }
    }

    /// Discover git repositories
    /// First checks if current directory is a git repo, if so returns just that
    /// Otherwise scans immediate subdirectories for git repos
    fn discover_repos() -> Vec<Repository> {
        // First, check if we're in a git repo
        if let Ok(repo) = Repository::discover(".") {
            return vec![repo];
        }

        // Not in a git repo, scan immediate subdirectories
        let mut repos = Vec::new();

        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    // Try to open as a git repository
                    if let Ok(repo) = Repository::open(&path) {
                        repos.push(repo);
                    }
                }
            }
        }

        // Sort by repo name for consistent ordering
        repos.sort_by(|a, b| {
            let name_a = a.workdir().and_then(|p| p.file_name());
            let name_b = b.workdir().and_then(|p| p.file_name());
            name_a.cmp(&name_b)
        });

        repos
    }

    /// Get the names of all tracked repositories
    pub fn repo_names(&self) -> Vec<String> {
        self.repos
            .iter()
            .filter_map(|r| {
                r.workdir()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .collect()
    }

    /// Check if there's a new commit since last check in any repository
    /// Returns details about the detected commit if found
    pub fn check_for_new_commit(&mut self) -> Option<DetectedCommit> {
        for repo in &self.repos {
            let repo_path = repo.path().to_path_buf();
            let current_head = repo
                .head()
                .ok()
                .and_then(|r| r.target())
                .map(|oid| oid.to_string());

            if let Some(current) = current_head {
                let is_new = match self.last_heads.get(&repo_path) {
                    Some(old) => old != &current,
                    None => false, // Don't count initial HEAD as new commit
                };

                if is_new {
                    self.last_heads.insert(repo_path, current.clone());

                    let project_id = Self::get_project_id(repo);
                    let project_name = Self::get_project_name(repo);

                    return Some(DetectedCommit {
                        commit_hash: current,
                        project_id,
                        project_name,
                    });
                }
            }
        }

        None
    }

    /// Get the project identifier (remote URL or absolute path)
    fn get_project_id(repo: &Repository) -> String {
        // Try to get the origin remote URL
        repo.find_remote("origin")
            .ok()
            .and_then(|remote| remote.url().map(|s| s.to_string()))
            .unwrap_or_else(|| {
                // Fallback to canonical absolute path
                repo.workdir()
                    .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            })
    }

    /// Get the project display name (folder name)
    fn get_project_name(repo: &Repository) -> String {
        repo.workdir()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Get basic git info (repos being tracked)
    pub fn get_stats(&self) -> GitStats {
        if self.repos.is_empty() {
            return GitStats {
                in_git_repo: false,
                ..Default::default()
            };
        }

        GitStats {
            in_git_repo: true,
            repo_count: self.repos.len(),
            repo_names: self.repo_names(),
        }
    }

    /// Get the paths to all .git directories (for file watching)
    pub fn git_dirs(&self) -> Vec<PathBuf> {
        self.repos.iter().map(|r| r.path().to_path_buf()).collect()
    }
}

impl Default for GitTracker {
    fn default() -> Self {
        Self::new()
    }
}
