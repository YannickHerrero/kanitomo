mod persistence;

pub use persistence::{
    calculate_happiness_from_commits, calculate_streak_from_history, get_today_by_project,
    get_today_commit_count, get_week_summary, AppState, StateManager, TrackedCommit,
};
