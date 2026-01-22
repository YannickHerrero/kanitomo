mod persistence;

pub use persistence::{
    calculate_streak_from_history, get_today_by_project, get_week_summary, AppState, StateManager,
    TrackedCommit,
};
