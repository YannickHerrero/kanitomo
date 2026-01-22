use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use git2::{Repository, Sort};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Statistics about git activity
#[derive(Debug, Clone, Default)]
pub struct GitStats {
    /// Number of commits made today
    pub commits_today: u32,
    /// Current streak of consecutive days with commits
    pub current_streak: u32,
    /// Best streak ever (from persistence)
    pub best_streak: u32,
    /// Time of the last commit
    pub last_commit: Option<DateTime<Local>>,
    /// Whether we're in a git repository
    pub in_git_repo: bool,
    /// Number of repositories being tracked
    pub repo_count: usize,
    /// Names of all tracked repositories
    pub repo_names: Vec<String>,
}

impl GitStats {
    /// Format the last commit time as a human-readable string
    pub fn last_commit_ago(&self) -> String {
        match self.last_commit {
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
    pub fn check_for_new_commit(&mut self) -> bool {
        let mut has_new = false;

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
                    None => true,
                };

                if is_new {
                    self.last_heads.insert(repo_path, current);
                    has_new = true;
                }
            }
        }

        has_new
    }

    /// Get current git statistics aggregated across all repositories
    pub fn get_stats(&self) -> GitStats {
        if self.repos.is_empty() {
            return GitStats {
                in_git_repo: false,
                ..Default::default()
            };
        }

        let today = Local::now().date_naive();
        let cutoff_date = today - Duration::days(30);

        let mut commits_today: u32 = 0;
        let mut last_commit_time: Option<DateTime<Local>> = None;
        let mut all_commit_dates: HashSet<NaiveDate> = HashSet::new();

        for repo in &self.repos {
            if let Ok(mut revwalk) = repo.revwalk() {
                revwalk.set_sorting(Sort::TIME).ok();

                if revwalk.push_head().is_ok() {
                    for oid in revwalk.filter_map(|r| r.ok()) {
                        if let Ok(commit) = repo.find_commit(oid) {
                            let time = commit.time();
                            let datetime = Utc
                                .timestamp_opt(time.seconds(), 0)
                                .single()
                                .map(|dt| dt.with_timezone(&Local));

                            if let Some(dt) = datetime {
                                let date = dt.date_naive();

                                // Stop if older than 30 days
                                if date < cutoff_date {
                                    break;
                                }

                                // Track most recent commit across all repos
                                if last_commit_time.is_none() || Some(dt) > last_commit_time {
                                    last_commit_time = Some(dt);
                                }

                                // Count commits today
                                if date == today {
                                    commits_today += 1;
                                }

                                // Track all dates for streak calculation
                                all_commit_dates.insert(date);
                            }
                        }
                    }
                }
            }
        }

        GitStats {
            in_git_repo: true,
            repo_count: self.repos.len(),
            repo_names: self.repo_names(),
            commits_today,
            last_commit: last_commit_time,
            current_streak: calculate_streak(&all_commit_dates, today),
            ..Default::default()
        }
    }

    /// Get the paths to all .git directories (for file watching)
    pub fn git_dirs(&self) -> Vec<PathBuf> {
        self.repos.iter().map(|r| r.path().to_path_buf()).collect()
    }
}

/// Calculate the current streak of consecutive days with commits
fn calculate_streak(commit_dates: &HashSet<NaiveDate>, today: NaiveDate) -> u32 {
    let mut streak = 0;
    let mut check_date = today;

    // If no commit today, start from yesterday
    if !commit_dates.contains(&check_date) {
        check_date = check_date.pred_opt().unwrap_or(check_date);
    }

    // Count consecutive days
    while commit_dates.contains(&check_date) {
        streak += 1;
        check_date = match check_date.pred_opt() {
            Some(d) => d,
            None => break,
        };
    }

    streak
}

impl Default for GitTracker {
    fn default() -> Self {
        Self::new()
    }
}
