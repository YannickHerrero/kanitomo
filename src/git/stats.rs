use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use git2::{Repository, Sort};
use std::collections::HashSet;
use std::path::Path;

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
    /// Total commits tracked in this session
    #[allow(dead_code)]
    pub total_commits_tracked: u32,
    /// Whether we're in a git repository
    pub in_git_repo: bool,
    /// The repository name (folder name)
    pub repo_name: Option<String>,
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

/// Tracks git repository activity
pub struct GitTracker {
    repo: Option<Repository>,
    /// Last known HEAD commit hash
    last_head: Option<String>,
}

impl GitTracker {
    /// Create a new git tracker for the current directory
    pub fn new() -> Self {
        let repo = Repository::discover(".").ok();
        let last_head = repo
            .as_ref()
            .and_then(|r| r.head().ok()?.target().map(|oid| oid.to_string()));

        Self { repo, last_head }
    }

    /// Create a tracker for a specific path
    #[allow(dead_code)]
    pub fn from_path(path: &Path) -> Self {
        let repo = Repository::discover(path).ok();
        let last_head = repo
            .as_ref()
            .and_then(|r| r.head().ok()?.target().map(|oid| oid.to_string()));

        Self { repo, last_head }
    }

    /// Check if we're in a git repository
    #[allow(dead_code)]
    pub fn is_in_repo(&self) -> bool {
        self.repo.is_some()
    }

    /// Get the repository name
    pub fn repo_name(&self) -> Option<String> {
        self.repo.as_ref().and_then(|r| {
            r.workdir()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
    }

    /// Check if there's a new commit since last check
    pub fn check_for_new_commit(&mut self) -> bool {
        let Some(repo) = &self.repo else {
            return false;
        };

        let current_head = repo
            .head()
            .ok()
            .and_then(|r| r.target())
            .map(|oid| oid.to_string());

        let has_new = match (&self.last_head, &current_head) {
            (Some(old), Some(new)) => old != new,
            (None, Some(_)) => true,
            _ => false,
        };

        if has_new {
            self.last_head = current_head;
        }

        has_new
    }

    /// Get current git statistics
    pub fn get_stats(&self) -> GitStats {
        let Some(repo) = &self.repo else {
            return GitStats {
                in_git_repo: false,
                ..Default::default()
            };
        };

        let mut stats = GitStats {
            in_git_repo: true,
            repo_name: self.repo_name(),
            ..Default::default()
        };

        // Get commits
        if let Ok(mut revwalk) = repo.revwalk() {
            revwalk.set_sorting(Sort::TIME).ok();

            if revwalk.push_head().is_ok() {
                let today = Local::now().date_naive();
                let mut commit_dates: HashSet<NaiveDate> = HashSet::new();
                let mut last_commit_time: Option<DateTime<Local>> = None;

                for oid in revwalk.filter_map(|r| r.ok()) {
                    if let Ok(commit) = repo.find_commit(oid) {
                        let time = commit.time();
                        let datetime = Utc
                            .timestamp_opt(time.seconds(), 0)
                            .single()
                            .map(|dt| dt.with_timezone(&Local));

                        if let Some(dt) = datetime {
                            let date = dt.date_naive();

                            // Track last commit
                            if last_commit_time.is_none() {
                                last_commit_time = Some(dt);
                            }

                            // Count commits today
                            if date == today {
                                stats.commits_today += 1;
                            }

                            // Track all dates for streak calculation
                            commit_dates.insert(date);
                        }
                    }
                }

                stats.last_commit = last_commit_time;
                stats.current_streak = calculate_streak(&commit_dates, today);
            }
        }

        stats
    }

    /// Get the path to the .git directory (for file watching)
    pub fn git_dir(&self) -> Option<std::path::PathBuf> {
        self.repo.as_ref().map(|r| r.path().to_path_buf())
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
