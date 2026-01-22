use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, Local, Weekday};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// Last time the app was opened
    pub last_seen: DateTime<Local>,
    /// Current happiness level (0-100)
    pub happiness: u8,
    /// Best streak ever achieved
    pub best_streak: u32,
    /// Total commits tracked across all sessions
    pub total_commits_tracked: u32,
    /// Version for future migrations
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_seen: Local::now(),
            happiness: 50, // Start at neutral
            best_streak: 0,
            total_commits_tracked: 0,
            version: 1,
        }
    }
}

/// Manages saving and loading application state
pub struct StateManager {
    state_path: PathBuf,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Result<Self> {
        let state_dir = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .context("Could not find home directory")?
            .join(".crabagochi");

        // Create directory if it doesn't exist
        fs::create_dir_all(&state_dir).context("Failed to create crabagochi data directory")?;

        let state_path = state_dir.join("state.json");

        Ok(Self { state_path })
    }

    /// Load state from disk, applying time-based decay
    pub fn load(&self) -> Result<AppState> {
        if !self.state_path.exists() {
            return Ok(AppState::default());
        }

        let contents = fs::read_to_string(&self.state_path).context("Failed to read state file")?;

        let mut state: AppState =
            serde_json::from_str(&contents).context("Failed to parse state file")?;

        // Apply decay based on time passed
        let decay = calculate_decay(state.last_seen, Local::now());
        state.happiness = state.happiness.saturating_sub(decay);

        // Update last seen
        state.last_seen = Local::now();

        Ok(state)
    }

    /// Save state to disk
    pub fn save(&self, state: &AppState) -> Result<()> {
        let mut state = state.clone();
        state.last_seen = Local::now();

        let contents = serde_json::to_string_pretty(&state).context("Failed to serialize state")?;

        fs::write(&self.state_path, contents).context("Failed to write state file")?;

        Ok(())
    }

    /// Get the path to the state file
    #[allow(dead_code)]
    pub fn state_path(&self) -> &PathBuf {
        &self.state_path
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create state manager")
    }
}

/// Calculate happiness decay based on time elapsed
/// Weekends don't count towards decay
fn calculate_decay(last_seen: DateTime<Local>, now: DateTime<Local>) -> u8 {
    // Count weekday hours that have passed
    let weekday_hours = count_weekday_hours(last_seen, now);

    // Decay rate: lose ~5 happiness per hour of weekday time
    // This means after 10 hours of not committing, you'd lose 50 happiness
    // But weekends are free!
    let decay_per_hour = 5.0;
    let total_decay = (weekday_hours as f32 * decay_per_hour) as u8;

    // Cap decay at a maximum so the crab doesn't instantly die
    total_decay.min(80)
}

/// Count the number of weekday hours between two times
fn count_weekday_hours(start: DateTime<Local>, end: DateTime<Local>) -> u64 {
    if end <= start {
        return 0;
    }

    let mut hours = 0u64;
    let mut current = start;

    // Iterate through each hour
    while current < end {
        let weekday = current.weekday();

        // Only count weekday hours (Monday = 0 through Friday = 4)
        if !is_weekend(weekday) {
            hours += 1;
        }

        current = current + Duration::hours(1);

        // Safety: prevent infinite loops on very large time spans
        if hours > 10000 {
            break;
        }
    }

    hours
}

/// Check if a weekday is a weekend day
fn is_weekend(weekday: Weekday) -> bool {
    matches!(weekday, Weekday::Sat | Weekday::Sun)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_weekend_no_decay() {
        // Saturday 10am to Sunday 10pm = 0 weekday hours
        let start = Local.with_ymd_and_hms(2026, 1, 24, 10, 0, 0).unwrap(); // Saturday
        let end = Local.with_ymd_and_hms(2026, 1, 25, 22, 0, 0).unwrap(); // Sunday

        let hours = count_weekday_hours(start, end);
        assert_eq!(hours, 0);
    }

    #[test]
    fn test_weekday_decay() {
        // Monday 9am to Monday 5pm = 8 weekday hours
        let start = Local.with_ymd_and_hms(2026, 1, 19, 9, 0, 0).unwrap(); // Monday
        let end = Local.with_ymd_and_hms(2026, 1, 19, 17, 0, 0).unwrap(); // Monday

        let hours = count_weekday_hours(start, end);
        assert_eq!(hours, 8);
    }

    #[test]
    fn test_decay_calculation() {
        // 10 weekday hours = 50 decay
        let start = Local.with_ymd_and_hms(2026, 1, 19, 8, 0, 0).unwrap();
        let end = Local.with_ymd_and_hms(2026, 1, 19, 18, 0, 0).unwrap();

        let decay = calculate_decay(start, end);
        assert_eq!(decay, 50);
    }
}
