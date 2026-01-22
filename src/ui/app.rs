use crate::crab::Crab;
use crate::git::{GitStats, GitTracker};
use crate::state::{AppState, StateManager};
use crate::ui::widgets;
use anyhow::Result;
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
    /// Current git stats
    pub git_stats: GitStats,
    /// State manager for persistence
    pub state_manager: StateManager,
    /// Current app state
    pub app_state: AppState,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Debug mode enables feed/punish controls
    pub debug_mode: bool,
    /// Whether to show the repo list overlay
    pub show_repo_list: bool,
    /// File watcher for git changes (kept alive to maintain watching)
    _watcher: Option<RecommendedWatcher>,
    /// Channel for receiving file change events
    watcher_rx: Option<Receiver<notify::Result<notify::Event>>>,
    /// Last time we saved state
    last_save: Instant,
}

impl App {
    /// Create a new app instance
    pub fn new(debug_mode: bool) -> Result<Self> {
        let state_manager = StateManager::new()?;
        let app_state = state_manager.load()?;

        let git_tracker = GitTracker::new();
        let mut git_stats = git_tracker.get_stats();

        // Update best streak from persistence
        git_stats.best_streak = app_state.best_streak;

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

        Ok(Self {
            crab,
            git_tracker,
            git_stats,
            state_manager,
            app_state,
            should_quit: false,
            debug_mode,
            show_repo_list: false,
            _watcher: watcher,
            watcher_rx,
            last_save: Instant::now(),
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
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('a') => {
                // Toggle repo list view (only if tracking multiple repos)
                if self.git_stats.repo_count > 1 {
                    self.show_repo_list = !self.show_repo_list;
                }
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
        if has_events && self.git_tracker.check_for_new_commit() {
            self.on_new_commit();
        }
    }

    /// Called when a new commit is detected
    fn on_new_commit(&mut self) {
        // Boost happiness significantly
        self.crab.boost_happiness(25);
        self.crab.celebrate();

        // Update stats
        self.refresh_stats();

        // Update app state
        self.app_state.happiness = self.crab.happiness;
        self.app_state.total_commits_tracked += 1;

        // Update best streak if needed
        if self.git_stats.current_streak > self.app_state.best_streak {
            self.app_state.best_streak = self.git_stats.current_streak;
            self.git_stats.best_streak = self.app_state.best_streak;
        }
    }

    /// Refresh git statistics (display only, no happiness changes)
    fn refresh_stats(&mut self) {
        self.git_stats = self.git_tracker.get_stats();
        self.git_stats.best_streak = self.app_state.best_streak;
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
        widgets::render_title(frame, chunks[0]);
        widgets::render_crab(frame, &self.crab, chunks[1]);
        widgets::render_stats(frame, &self.git_stats, self.crab.happiness, chunks[2]);
        widgets::render_help(
            frame,
            chunks[3],
            self.debug_mode,
            self.git_stats.repo_count > 1,
        );

        // Render repo list overlay if active
        if self.show_repo_list {
            widgets::render_repo_list(frame, &self.git_stats.repo_names, area);
        }
    }
}
