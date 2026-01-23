use crate::crab::{Crab, Mood};
use crate::environment::Environment;
use crate::git::{DetectedCommit, GitStats, GitTracker};
use crate::state::{
    calculate_happiness_from_commits, calculate_streak_from_history, get_today_commit_count,
    AppState, StateManager, TrackedCommit,
};
use crate::ui::{messages, widgets};
use anyhow::Result;
use chrono::{Datelike, Local};
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
    /// Whether to show the stats panel
    pub show_stats: bool,
    /// Whether to show the help bar
    pub show_help: bool,
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
    /// The environment (ground, background, objects)
    pub environment: Environment,
    /// Last known terminal size (for resize detection)
    last_terminal_size: (u16, u16),
    /// Debug: run an accelerated day/night cycle
    fast_cycle: bool,
}

impl App {
    /// Create a new app instance
    pub fn new(debug_mode: bool) -> Result<Self> {
        let state_manager = StateManager::new()?;
        let app_state = state_manager.load()?;

        let git_tracker = GitTracker::new();
        let git_stats = git_tracker.get_stats();

        // Create the crab with loaded happiness
        // Start at a high y position so it falls to ground on first update
        let crab = Crab::new((10.0, 100.0), app_state.happiness);

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

        // Create initial environment with default size (will be resized on first draw)
        let environment = Environment::generate(80, 15, app_state.ground_style);

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
            show_stats: true,
            show_help: false,
            _watcher: watcher,
            watcher_rx,
            last_save: Instant::now(),
            current_message: initial_message,
            temp_message: None,
            temp_message_until: None,
            last_message_change: Instant::now(),
            last_mood: current_mood,
            environment,
            last_terminal_size: (0, 0), // Will trigger regeneration on first draw
            fast_cycle: false,
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
                if self.show_help {
                    self.show_help = false;
                } else if self.show_repo_list {
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
                    self.show_help = false;
                }
            }
            KeyCode::Char('d') => {
                // Toggle details view
                self.show_details = !self.show_details;
                self.show_repo_list = false; // Close other overlay
                self.show_help = false;
            }
            KeyCode::Char('r') => {
                // Manual refresh
                self.refresh_stats();
            }
            KeyCode::Char('?') => {
                // Toggle help window
                self.show_help = !self.show_help;
                if self.show_help {
                    self.show_repo_list = false;
                    self.show_details = false;
                }
            }
            KeyCode::Char('f') if self.debug_mode => {
                // Manual feed (debug only)
                let timestamp = Local::now();
                let tracked = TrackedCommit {
                    timestamp,
                    commit_hash: format!("debug-{}", timestamp.timestamp()),
                    project_id: "debug".to_string(),
                    project_name: "debug".to_string(),
                };
                self.app_state.commit_history.push(tracked);
                self.app_state.last_commit_time = Some(timestamp);
                self.sync_happiness_from_commits();
                self.crab.celebrate();
                self.set_temp_message("Debug feed: +1 commit");
            }
            KeyCode::Char('p') if self.debug_mode => {
                // Punish (debug only)
                let today = Local::now().date_naive();
                if let Some(index) = self
                    .app_state
                    .commit_history
                    .iter()
                    .rposition(|commit| commit.timestamp.date_naive() == today)
                {
                    self.app_state.commit_history.remove(index);
                    self.sync_last_commit_time();
                    self.sync_happiness_from_commits();
                    self.set_temp_message("Debug punish: -1 commit");
                }
            }
            KeyCode::Char('s') => {
                // Toggle stats panel
                self.show_stats = !self.show_stats;
            }
            KeyCode::Char('x') if self.debug_mode => {
                // Toggle movement freeze (debug only)
                self.crab.movement_frozen = !self.crab.movement_frozen;
            }
            KeyCode::Char('c') if self.debug_mode => {
                // Toggle fast day/night cycle (debug only)
                self.fast_cycle = !self.fast_cycle;
                let status = if self.fast_cycle { "on" } else { "off" };
                self.set_temp_message(&format!("Fast cycle: {status}"));
            }
            KeyCode::Char('g') if self.debug_mode => {
                // Cycle ground styles (debug only)
                self.app_state.ground_style = self.app_state.ground_style.next();
                self.app_state.ground_style_week = Local::now().iso_week().week();
                let (width, height) =
                    if self.last_terminal_size.0 > 0 && self.last_terminal_size.1 > 0 {
                        self.last_terminal_size
                    } else {
                        (self.environment.width, self.environment.height)
                    };
                self.environment =
                    Environment::generate(width, height, self.app_state.ground_style);
                self.set_temp_message(&format!(
                    "Ground style: {}",
                    self.app_state.ground_style.display_name()
                ));
            }
            _ => {}
        }
    }

    /// Update game state
    fn update(&mut self) {
        let dt = 0.05;
        // Use last known terminal size for bounds (updated in draw)
        let bounds = (
            self.last_terminal_size.0 as f32 - 2.0,
            self.last_terminal_size.1 as f32,
        );

        // Update crab animation (only if we have valid bounds)
        if bounds.0 > 0.0 && bounds.1 > 0.0 {
            self.crab.update(dt, bounds);
        }

        // Check for file system events (new commits)
        self.check_for_changes();

        // Sync happiness based on today's commit count
        self.sync_happiness_from_commits();

        // Check for mood changes
        self.check_mood_change();

        // Update messages
        self.update_messages();

        // Update day/night cycle and environment movement
        let cycle_speed = if self.fast_cycle {
            self.environment.cycle_duration.as_secs_f32() / 10.0
        } else {
            1.0
        };
        let cloud_speed = if self.fast_cycle { 3.0 } else { 1.0 };
        self.environment.update_cycle(dt, cycle_speed, cloud_speed);

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

        self.crab.celebrate();

        // Update app state
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

    /// Sync happiness based on today's commit count
    fn sync_happiness_from_commits(&mut self) {
        let commits_today = get_today_commit_count(&self.app_state.commit_history);
        let happiness = calculate_happiness_from_commits(commits_today);
        self.crab.happiness = happiness;
        self.app_state.happiness = happiness;
    }

    fn sync_last_commit_time(&mut self) {
        self.app_state.last_commit_time = self
            .app_state
            .commit_history
            .iter()
            .map(|commit| commit.timestamp)
            .max();
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

        // Layout: Title | Crab Area | Stats (optional)
        let mut constraints = vec![
            Constraint::Length(1), // Title
            Constraint::Min(8),    // Crab area
        ];

        if self.show_stats {
            constraints.push(Constraint::Length(12));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let crab_area = chunks[1];

        // Check for terminal resize and regenerate environment
        let current_size = (crab_area.width, crab_area.height);
        if current_size != self.last_terminal_size {
            self.environment = Environment::generate(
                crab_area.width,
                crab_area.height,
                self.app_state.ground_style,
            );
            self.last_terminal_size = current_size;
        }

        // Update crab bounds based on actual crab area size
        let crab_bounds = (crab_area.width as f32 - 2.0, crab_area.height as f32);
        self.crab.update(0.0, crab_bounds); // Update bounds without time delta

        // Render components in correct order:
        // 1. Title
        widgets::render_title(frame, chunks[0], self.get_display_message());

        // 2. Environment background (sky, sun/moon, clouds, stars)
        widgets::render_environment_background(frame, &self.environment, crab_area);

        // 3. Kani (the crab)
        widgets::render_crab(frame, &self.crab, crab_area);

        // 4. Ground line (at bottom of crab area)
        widgets::render_ground(frame, &self.environment, crab_area);

        if self.show_stats {
            // 5. Stats panel
            widgets::render_stats(
                frame,
                &self.git_stats,
                &self.app_state,
                self.crab.happiness,
                chunks[2],
            );
        }

        // Render overlays
        if self.show_repo_list {
            widgets::render_repo_list(frame, &self.git_stats.repo_names, area);
        }

        if self.show_details {
            widgets::render_details_overlay(frame, &self.app_state, area);
        }

        if self.show_help {
            widgets::render_help_overlay(
                frame,
                area,
                self.debug_mode,
                self.git_stats.repo_count > 1,
                self.show_stats,
            );
        }
    }
}
