use crate::crab::{Crab, Mood};
use crate::git::{DetectedCommit, GitStats, GitTracker};
use crate::state::{calculate_streak_from_history, AppState, StateManager, TrackedCommit};
use crate::ui::{messages, widgets};
use anyhow::Result;
use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

/// Main application state
pub struct App {
    /// The crab entity
    pub crab: Crab,
    /// Git tracker
    pub git_tracker: GitTracker,
    /// Current git stats (basic repo info)
    pub git_stats: GitStats,
    /// State manager for persistence
    pub state_manager: StateManager,
    /// Current app state (includes commit history)
    pub app_state: AppState,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Debug mode enables feed/punish controls
    pub debug_mode: bool,
    /// Whether to show the repo list overlay
    pub show_repo_list: bool,
    /// Whether to show the details overlay
    pub show_details: bool,
    /// File watcher for git changes (kept alive to maintain watching)
    _watcher: Option<RecommendedWatcher>,
    /// Channel for receiving file change events
    watcher_rx: Option<Receiver<notify::Result<notify::Event>>>,
    /// Last time we saved state
    last_save: Instant,
    /// Current message from Kani
    current_message: String,
    /// Temporary message (for reactions to events)
    temp_message: Option<String>,
    /// When the temp message should expire
    temp_message_until: Option<Instant>,
    /// Last time we changed the idle message
    last_message_change: Instant,
    /// Last known mood (to detect mood changes)
    last_mood: Mood,
}

impl App {
    /// Create a new app instance
    pub fn new(debug_mode: bool) -> Result<Self> {
        let state_manager = StateManager::new()?;
        let app_state = state_manager.load()?;

        let git_tracker = GitTracker::new();
        let git_stats = git_tracker.get_stats();

        // Create the crab with loaded happiness
        let crab = Crab::new((10.0, 2.0), app_state.happiness);

        // Set up file watcher for all git repos
        let git_dirs = git_tracker.git_dirs();
        let (watcher, watcher_rx) = if !git_dirs.is_empty() {
            let (tx, rx) = channel();

            // Use event-based watching (no polling) for better performance
            let mut watcher = RecommendedWatcher::new(
                move |res| {
                    let _ = tx.send(res);
                },
                Config::default(),
            )?;

            // Watch HEAD and refs for each repository
            for git_dir in &git_dirs {
                let head_path = git_dir.join("HEAD");
                let refs_path = git_dir.join("refs");

                if head_path.exists() {
                    watcher.watch(&head_path, RecursiveMode::NonRecursive).ok();
                }
                if refs_path.exists() {
                    watcher.watch(&refs_path, RecursiveMode::Recursive).ok();
                }
            }

            (Some(watcher), Some(rx))
        } else {
            (None, None)
        };

        let current_mood = Mood::from_happiness(app_state.happiness);
        let initial_message = messages::get_mood_message(current_mood).to_string();

        Ok(Self {
            crab,
            git_tracker,
            git_stats,
            state_manager,
            app_state,
            should_quit: false,
            debug_mode,
            show_repo_list: false,
            show_details: false,
            _watcher: watcher,
            watcher_rx,
            last_save: Instant::now(),
            current_message: initial_message,
            temp_message: None,
            temp_message_until: None,
            last_message_change: Instant::now(),
            last_mood: current_mood,
        })
    }

    /// Run the main event loop
    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    ) -> Result<()> {
        let tick_rate = Duration::from_millis(50); // 20 FPS
        let mut last_tick = Instant::now();

        while !self.should_quit {
            // Draw
            terminal.draw(|frame| self.draw(frame))?;

            // Handle input with timeout
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code);
                    }
                }
            }

            // Update on tick
            if last_tick.elapsed() >= tick_rate {
                self.update();
                last_tick = Instant::now();
            }
        }

        // Save state on exit
        self.save_state()?;

        Ok(())
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.show_repo_list {
                    self.show_repo_list = false;
                } else if self.show_details {
                    self.show_details = false;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('a') => {
                // Toggle repo list view (only if tracking multiple repos)
                if self.git_stats.repo_count > 1 {
                    self.show_repo_list = !self.show_repo_list;
                    self.show_details = false; // Close other overlay
                }
            }
            KeyCode::Char('d') => {
                // Toggle details view
                self.show_details = !self.show_details;
                self.show_repo_list = false; // Close other overlay
            }
            KeyCode::Char('r') => {
                // Manual refresh
                self.refresh_stats();
            }
            KeyCode::Char('f') if self.debug_mode => {
                // Manual feed (debug only)
                self.crab.boost_happiness(5);
                self.crab.celebrate();
                self.app_state.happiness = self.crab.happiness;
            }
            KeyCode::Char('p') if self.debug_mode => {
                // Punish (debug only)
                self.crab.decay_happiness(5);
                self.app_state.happiness = self.crab.happiness;
            }
            KeyCode::Char('s') if self.debug_mode => {
                // Toggle movement freeze (debug only)
                self.crab.movement_frozen = !self.crab.movement_frozen;
            }
            _ => {}
        }
    }

    /// Update game state
    fn update(&mut self) {
        // Get terminal size for bounds
        let bounds = (80.0, 15.0); // Default, will be updated in draw

        // Update crab animation
        self.crab.update(0.05, bounds);

        // Check for file system events (new commits)
        self.check_for_changes();

        // Check for mood changes
        self.check_mood_change();

        // Update messages
        self.update_messages();

        // Periodic save (every 60 seconds)
        if self.last_save.elapsed() > Duration::from_secs(60) {
            let _ = self.save_state();
            self.last_save = Instant::now();
        }
    }

    /// Check for git changes via file watcher
    fn check_for_changes(&mut self) {
        // First, check if we have any pending events
        let has_events = if let Some(ref rx) = self.watcher_rx {
            // Drain all pending events and check if any exist
            let mut found = false;
            while rx.try_recv().is_ok() {
                found = true;
            }
            found
        } else {
            false
        };

        // If we had events, check for new commits
        if has_events {
            if let Some(detected) = self.git_tracker.check_for_new_commit() {
                self.on_new_commit(detected);
            }
        }
    }

    /// Called when a new commit is detected
    fn on_new_commit(&mut self, detected: DetectedCommit) {
        // Check for duplicate (same commit hash)
        if self
            .app_state
            .commit_history
            .iter()
            .any(|c| c.commit_hash == detected.commit_hash)
        {
            return; // Already tracked this commit
        }

        // Add to commit history
        let tracked = TrackedCommit {
            timestamp: Local::now(),
            commit_hash: detected.commit_hash,
            project_id: detected.project_id,
            project_name: detected.project_name,
        };
        self.app_state.commit_history.push(tracked);

        // Update last commit time
        self.app_state.last_commit_time = Some(Local::now());

        // Recalculate streak
        self.app_state.current_streak =
            calculate_streak_from_history(&self.app_state.commit_history);

        // Update best streak if needed
        if self.app_state.current_streak > self.app_state.best_streak {
            self.app_state.best_streak = self.app_state.current_streak;
        }

        // Boost happiness significantly
        self.crab.boost_happiness(25);
        self.crab.celebrate();

        // Update app state
        self.app_state.happiness = self.crab.happiness;
        self.app_state.total_commits_tracked += 1;

        // Show a commit reaction message for 30 seconds
        self.set_temp_message(messages::get_commit_message());
    }

    /// Check for mood changes and react with messages
    fn check_mood_change(&mut self) {
        let current_mood = Mood::from_happiness(self.crab.happiness);

        if current_mood != self.last_mood {
            // Mood changed - show a reaction message
            let message = if current_mood as u8 > self.last_mood as u8 {
                // Mood went down (higher enum value = worse mood)
                messages::get_mood_down_message()
            } else {
                // Mood went up
                messages::get_mood_up_message()
            };

            self.set_temp_message(message);
            self.last_mood = current_mood;
        }
    }

    /// Update message rotation
    fn update_messages(&mut self) {
        // Check if temp message has expired
        if let Some(until) = self.temp_message_until {
            if Instant::now() >= until {
                self.temp_message = None;
                self.temp_message_until = None;
            }
        }

        // Rotate idle message every 2 minutes (only if no temp message)
        if self.temp_message.is_none()
            && self.last_message_change.elapsed() > Duration::from_secs(120)
        {
            let mood = Mood::from_happiness(self.crab.happiness);
            self.current_message = messages::get_mood_message(mood).to_string();
            self.last_message_change = Instant::now();
        }
    }

    /// Set a temporary message that shows for 30 seconds
    fn set_temp_message(&mut self, message: &str) {
        self.temp_message = Some(message.to_string());
        self.temp_message_until = Some(Instant::now() + Duration::from_secs(30));
    }

    /// Get the current message to display
    pub fn get_display_message(&self) -> &str {
        self.temp_message
            .as_deref()
            .unwrap_or(&self.current_message)
    }

    /// Refresh git statistics (basic repo info)
    fn refresh_stats(&mut self) {
        self.git_stats = self.git_tracker.get_stats();
    }

    /// Save application state
    fn save_state(&mut self) -> Result<()> {
        self.app_state.happiness = self.crab.happiness;
        self.state_manager.save(&self.app_state)?;
        Ok(())
    }

    /// Draw the UI
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Update crab bounds based on actual terminal size
        let crab_bounds = (area.width as f32 - 4.0, (area.height as f32 * 0.5) - 2.0);
        self.crab.update(0.0, crab_bounds); // Update bounds without time delta

        // Layout: Title | Crab Area | Stats | Help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Title
                Constraint::Min(8),     // Crab area
                Constraint::Length(12), // Stats
                Constraint::Length(1),  // Help
            ])
            .split(area);

        // Render components
        widgets::render_title(frame, chunks[0], self.get_display_message());
        widgets::render_crab(frame, &self.crab, chunks[1]);
        widgets::render_stats(
            frame,
            &self.git_stats,
            &self.app_state,
            self.crab.happiness,
            chunks[2],
        );
        widgets::render_help(
            frame,
            chunks[3],
            self.debug_mode,
            self.git_stats.repo_count > 1,
        );

        // Render overlays
        if self.show_repo_list {
            widgets::render_repo_list(frame, &self.git_stats.repo_names, area);
        }

        if self.show_details {
            widgets::render_details_overlay(frame, &self.app_state, area);
        }
    }
}
