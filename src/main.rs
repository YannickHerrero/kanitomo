mod crab;
mod environment;
mod git;
mod state;
mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::env;
use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use state::StateManager;
use ui::minigames::{
    BreakoutGame, DashGame, Game2048, Game2048Move, SnakeGame, TetrisGame, TetrisMode, VsrgGame,
};
use ui::{widgets, App, CrabCatchGame};

fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let debug_mode = args.iter().any(|arg| arg == "--debug" || arg == "-d");
    let reset_mode = args.iter().any(|arg| arg == "--reset");

    // Check for --game or -g flag
    let game_flag_index = args
        .iter()
        .position(|arg| arg == "--game" || arg == "-g" || arg.starts_with("--game="));

    if let Some(idx) = game_flag_index {
        let game_name = if args[idx].starts_with("--game=") {
            Some(args[idx].strip_prefix("--game=").unwrap())
        } else if idx + 1 < args.len() && !args[idx + 1].starts_with("-") {
            Some(args[idx + 1].as_str())
        } else {
            None // No game name provided, show menu
        };

        return handle_game_mode(game_name, debug_mode);
    }

    // Handle reset before setting up TUI
    if reset_mode {
        return handle_reset();
    }

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run app
    let result = run_game_selection_menu(&mut terminal, debug_mode);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Handle any errors
    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }

    Ok(())
}

/// Handle the --reset flag
fn handle_reset() -> Result<()> {
    println!("This will reset all Kanitomo stats:");
    println!("  - Happiness -> 50%");
    println!("  - Streak -> 0 days");
    println!("  - Best streak -> 0 days");
    println!("  - Commit history -> cleared");
    println!();
    print!("Are you sure? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        let state_manager = StateManager::new()?;
        state_manager.reset()?;
        println!("Stats reset successfully!");
    } else {
        println!("Reset cancelled.");
    }

    Ok(())
}

fn run_tamagotchi(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    debug_mode: bool,
) -> Result<()> {
    let mut app = App::new(debug_mode)?;
    app.run(terminal)?;
    Ok(())
}

/// Handle the --game flag
fn handle_game_mode(game_name: Option<&str>, debug_mode: bool) -> Result<()> {
    match game_name {
        None => run_game_selection_menu_with_setup(debug_mode),
        Some("kanitomo") => run_standalone_game("kanitomo", debug_mode),
        Some("crabcatch") => run_standalone_game("crabcatch", debug_mode),
        Some("snake") => run_standalone_game("snake", debug_mode),
        Some("breakout") => run_standalone_game("breakout", debug_mode),
        Some("tetris") => run_standalone_game("tetris", debug_mode),
        Some("dash") => run_standalone_game("dash", debug_mode),
        Some("2048") => run_standalone_game("2048", debug_mode),
        Some("vsrg") => run_standalone_game("vsrg", debug_mode),
        Some(invalid) => {
            eprintln!(
                "Unknown game '{}'. Available games: kanitomo, crabcatch, snake, breakout, tetris, dash, 2048, vsrg",
                invalid
            );
            std::process::exit(1);
        }
    }
}

enum StandaloneState {
    GameMenu,
    TetrisModeMenu,
    PlayingCrabCatch(CrabCatchGame),
    PlayingSnake(SnakeGame),
    PlayingBreakout(BreakoutGame),
    PlayingTetris(TetrisGame),
    PlayingDash(DashGame),
    Playing2048(Game2048),
    PlayingVsrg(VsrgGame),
    ShowCrabCatchResults(u32),
    ShowSnakeResults(u32),
    ShowBreakoutResults(u32, bool),
    ShowTetrisResults(TetrisMode, u32, f32),
    ShowDashResults(u32),
    Show2048Results(u32, u32),
    ShowVsrgResults(u32, f32, u32),
}

/// Run the game selection menu
fn run_game_selection_menu_with_setup(debug_mode: bool) -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_game_selection_menu(&mut terminal, debug_mode);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Run the game selection menu
fn run_game_selection_menu(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    debug_mode: bool,
) -> Result<()> {
    run_standalone_game_loop(terminal, StandaloneState::GameMenu, debug_mode)
}

/// Run a specific game directly
fn run_standalone_game(game_name: &str, debug_mode: bool) -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get initial terminal size
    let size = terminal.size()?;
    let bounds = (size.width, size.height);

    let result = if game_name == "kanitomo" {
        run_tamagotchi(&mut terminal, debug_mode)?;
        run_standalone_game_loop(&mut terminal, StandaloneState::GameMenu, debug_mode)
    } else {
        let initial_state = match game_name {
            "crabcatch" => StandaloneState::PlayingCrabCatch(CrabCatchGame::new(bounds)),
            "snake" => StandaloneState::PlayingSnake(SnakeGame::new(bounds)),
            "breakout" => StandaloneState::PlayingBreakout(BreakoutGame::new(bounds)),
            "tetris" => StandaloneState::TetrisModeMenu,
            "dash" => StandaloneState::PlayingDash(DashGame::new(bounds)),
            "2048" => StandaloneState::Playing2048(Game2048::new()),
            "vsrg" => StandaloneState::PlayingVsrg(VsrgGame::new(bounds)),
            _ => unreachable!(),
        };

        run_standalone_game_loop(&mut terminal, initial_state, debug_mode)
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Main game loop for standalone mode
fn run_standalone_game_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    initial_state: StandaloneState,
    debug_mode: bool,
) -> Result<()> {
    let state_manager = StateManager::new()?;
    let mut app_state = state_manager.load()?;
    let mut current_state = initial_state;
    let mut last_update = Instant::now();
    let mut last_size = terminal.size()?;

    loop {
        // Draw current state
        terminal.draw(|frame| {
            let area = frame.area();
            match &current_state {
                StandaloneState::GameMenu => {
                    widgets::render_minigame_menu(frame, area, &app_state);
                }
                StandaloneState::TetrisModeMenu => {
                    widgets::render_tetris_mode_menu(frame, area);
                }
                StandaloneState::PlayingCrabCatch(game) => {
                    widgets::render_crab_catch(frame, game, area);
                }
                StandaloneState::PlayingSnake(game) => {
                    widgets::render_snake_game(frame, game, area);
                }
                StandaloneState::PlayingBreakout(game) => {
                    widgets::render_breakout_game(frame, game, area);
                }
                StandaloneState::PlayingTetris(game) => {
                    widgets::render_tetris_game(frame, game, area);
                }
                StandaloneState::PlayingDash(game) => {
                    widgets::render_dash_game(frame, game, area);
                }
                StandaloneState::Playing2048(game) => {
                    widgets::render_2048_game(frame, game, area);
                }
                StandaloneState::PlayingVsrg(game) => {
                    widgets::render_vsrg_game(frame, game, area);
                }
                StandaloneState::ShowCrabCatchResults(score) => {
                    widgets::render_minigame_results(frame, area, *score, &app_state);
                }
                StandaloneState::ShowSnakeResults(score) => {
                    widgets::render_snake_results(frame, area, *score, &app_state);
                }
                StandaloneState::ShowBreakoutResults(score, victory) => {
                    widgets::render_breakout_results(frame, area, *score, *victory, &app_state);
                }
                StandaloneState::ShowTetrisResults(mode, score, time) => {
                    widgets::render_tetris_results(frame, area, *mode, *score, *time, &app_state);
                }
                StandaloneState::ShowDashResults(score) => {
                    widgets::render_dash_results(frame, area, *score, &app_state);
                }
                StandaloneState::Show2048Results(score, max_tile) => {
                    widgets::render_2048_results(frame, area, *score, *max_tile, &app_state);
                }
                StandaloneState::ShowVsrgResults(score, accuracy, max_combo) => {
                    widgets::render_vsrg_results(
                        frame, area, *score, *accuracy, *max_combo, &app_state,
                    );
                }
            }
        })?;

        // Handle input with timeout for animations
        let timeout = Duration::from_millis(16); // ~60 FPS
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match &mut current_state {
                    StandaloneState::GameMenu => match key.code {
                        KeyCode::Char('1') => {
                            run_tamagotchi(terminal, debug_mode)?;
                            app_state = state_manager.load()?;
                            last_update = Instant::now();
                            last_size = terminal.size()?;
                            current_state = StandaloneState::GameMenu;
                        }
                        KeyCode::Char('2') => {
                            let size = terminal.size()?;
                            current_state = StandaloneState::PlayingCrabCatch(CrabCatchGame::new((
                                size.width,
                                size.height,
                            )));
                        }
                        KeyCode::Char('3') => {
                            let size = terminal.size()?;
                            current_state = StandaloneState::PlayingSnake(SnakeGame::new((
                                size.width,
                                size.height,
                            )));
                        }
                        KeyCode::Char('4') => {
                            let size = terminal.size()?;
                            current_state = StandaloneState::PlayingBreakout(BreakoutGame::new((
                                size.width,
                                size.height,
                            )));
                        }
                        KeyCode::Char('5') => {
                            current_state = StandaloneState::TetrisModeMenu;
                        }
                        KeyCode::Char('6') => {
                            let size = terminal.size()?;
                            current_state = StandaloneState::PlayingDash(DashGame::new((
                                size.width,
                                size.height,
                            )));
                        }
                        KeyCode::Char('7') => {
                            current_state = StandaloneState::Playing2048(Game2048::new());
                        }
                        KeyCode::Char('8') => {
                            let size = terminal.size()?;
                            current_state = StandaloneState::PlayingVsrg(VsrgGame::new((
                                size.width,
                                size.height,
                            )));
                        }
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char(' ') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    StandaloneState::TetrisModeMenu => match key.code {
                        KeyCode::Char('1') => {
                            current_state =
                                StandaloneState::PlayingTetris(TetrisGame::new(TetrisMode::Normal));
                        }
                        KeyCode::Char('2') => {
                            current_state =
                                StandaloneState::PlayingTetris(TetrisGame::new(TetrisMode::Sprint));
                        }
                        KeyCode::Char('3') => {
                            current_state =
                                StandaloneState::PlayingTetris(TetrisGame::new(TetrisMode::Zen));
                        }
                        KeyCode::Char('4') => {
                            current_state =
                                StandaloneState::PlayingTetris(TetrisGame::new(TetrisMode::Dig));
                        }
                        KeyCode::Char('5') => {
                            current_state = StandaloneState::PlayingTetris(TetrisGame::new(
                                TetrisMode::Survival,
                            ));
                        }
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char(' ') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::PlayingCrabCatch(game) => match key.code {
                        KeyCode::Left | KeyCode::Char('h') => {
                            game.move_crab(-1);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            game.move_crab(1);
                        }
                        KeyCode::Char('q') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::PlayingSnake(game) => match key.code {
                        KeyCode::Left | KeyCode::Char('h') => {
                            game.set_direction(ui::minigames::Direction::Left);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            game.set_direction(ui::minigames::Direction::Right);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            game.set_direction(ui::minigames::Direction::Up);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            game.set_direction(ui::minigames::Direction::Down);
                        }
                        KeyCode::Char('q') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::PlayingBreakout(game) => match key.code {
                        KeyCode::Left | KeyCode::Char('h') => {
                            game.move_paddle(-1.0);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            game.move_paddle(1.0);
                        }
                        KeyCode::Char(' ') => {
                            game.launch_ball();
                        }
                        KeyCode::Char('q') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::PlayingTetris(game) => match key.code {
                        KeyCode::Left | KeyCode::Char('h') => {
                            game.move_piece(-1, 0);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            game.move_piece(1, 0);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            game.soft_drop();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            game.rotate_piece_cw();
                        }
                        KeyCode::Char('z') | KeyCode::Char('i') => {
                            game.rotate_piece_ccw();
                        }
                        KeyCode::Char('c') => {
                            game.hold();
                        }
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            game.hard_drop();
                        }
                        KeyCode::Char('q') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::PlayingDash(game) => match key.code {
                        KeyCode::Char(' ') | KeyCode::Up | KeyCode::Char('k') => {
                            game.jump();
                        }
                        KeyCode::Char('q') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::Playing2048(game) => match key.code {
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                            game.make_move(Game2048Move::Up);
                        }
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                            game.make_move(Game2048Move::Down);
                        }
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => {
                            game.make_move(Game2048Move::Left);
                        }
                        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => {
                            game.make_move(Game2048Move::Right);
                        }
                        KeyCode::Char('r') => {
                            game.reset();
                        }
                        KeyCode::Char('q') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::PlayingVsrg(game) => match key.code {
                        KeyCode::Char('d') => {
                            game.hit(0);
                        }
                        KeyCode::Char('f') => {
                            game.hit(1);
                        }
                        KeyCode::Char('j') => {
                            game.hit(2);
                        }
                        KeyCode::Char('k') => {
                            game.hit(3);
                        }
                        KeyCode::Char('q') => {
                            current_state = StandaloneState::GameMenu;
                        }
                        _ => {}
                    },
                    StandaloneState::ShowCrabCatchResults(_)
                    | StandaloneState::ShowSnakeResults(_)
                    | StandaloneState::ShowBreakoutResults(_, _)
                    | StandaloneState::ShowTetrisResults(_, _, _)
                    | StandaloneState::ShowDashResults(_)
                    | StandaloneState::Show2048Results(_, _)
                    | StandaloneState::ShowVsrgResults(_, _, _) => {
                        // Any key exits from results screen
                        current_state = StandaloneState::GameMenu;
                    }
                }
            }
        }

        // Update game state
        let now = Instant::now();
        let dt = now.duration_since(last_update).as_secs_f32();
        last_update = now;

        // Check for terminal resize
        let current_size = terminal.size()?;
        if current_size != last_size {
            last_size = current_size;
            let bounds = (current_size.width, current_size.height);

            match &mut current_state {
                StandaloneState::PlayingCrabCatch(game) => {
                    game.update_bounds(bounds);
                }
                StandaloneState::PlayingSnake(game) => {
                    game.update_bounds(bounds);
                }
                StandaloneState::PlayingBreakout(game) => {
                    game.update_bounds(bounds);
                }
                StandaloneState::PlayingDash(game) => {
                    game.update_bounds(bounds);
                }
                StandaloneState::PlayingVsrg(game) => {
                    game.update_bounds(bounds);
                }
                _ => {}
            }
        }

        match &mut current_state {
            StandaloneState::PlayingCrabCatch(game) => {
                game.update(dt);
                if game.is_finished() {
                    let score = game.score;
                    // Record score
                    app_state.minigame_best_scores.push(score);
                    app_state.minigame_best_scores.sort_by(|a, b| b.cmp(a));
                    if app_state.minigame_best_scores.len() > 100 {
                        app_state.minigame_best_scores.truncate(100);
                    }
                    state_manager.save(&app_state)?;
                    current_state = StandaloneState::ShowCrabCatchResults(score);
                }
            }
            StandaloneState::PlayingSnake(game) => {
                game.update(dt);
                if game.is_finished() {
                    let score = game.score;
                    // Record score
                    app_state.snake_best_scores.push(score);
                    app_state.snake_best_scores.sort_by(|a, b| b.cmp(a));
                    if app_state.snake_best_scores.len() > 100 {
                        app_state.snake_best_scores.truncate(100);
                    }
                    state_manager.save(&app_state)?;
                    current_state = StandaloneState::ShowSnakeResults(score);
                }
            }
            StandaloneState::PlayingBreakout(game) => {
                game.update(dt);
                if game.is_finished() {
                    let score = game.score;
                    let victory = game.victory;
                    // Record score
                    app_state.breakout_best_scores.push(score);
                    app_state.breakout_best_scores.sort_by(|a, b| b.cmp(a));
                    if app_state.breakout_best_scores.len() > 100 {
                        app_state.breakout_best_scores.truncate(100);
                    }
                    state_manager.save(&app_state)?;
                    current_state = StandaloneState::ShowBreakoutResults(score, victory);
                }
            }
            StandaloneState::PlayingTetris(game) => {
                game.update(dt);
                if game.is_finished() {
                    let mode = game.mode;
                    let score = game.score;
                    let time = game.elapsed_time;

                    // Record score based on mode
                    match mode {
                        TetrisMode::Normal => {
                            app_state.tetris_normal_scores.push(score);
                            app_state.tetris_normal_scores.sort_by(|a, b| b.cmp(a));
                            if app_state.tetris_normal_scores.len() > 100 {
                                app_state.tetris_normal_scores.truncate(100);
                            }
                        }
                        TetrisMode::Sprint => {
                            app_state.tetris_sprint_times.push(time);
                            app_state
                                .tetris_sprint_times
                                .sort_by(|a, b| a.partial_cmp(b).unwrap());
                            if app_state.tetris_sprint_times.len() > 100 {
                                app_state.tetris_sprint_times.truncate(100);
                            }
                        }
                        TetrisMode::Zen => {
                            app_state.tetris_zen_scores.push(score);
                            app_state.tetris_zen_scores.sort_by(|a, b| b.cmp(a));
                            if app_state.tetris_zen_scores.len() > 100 {
                                app_state.tetris_zen_scores.truncate(100);
                            }
                        }
                        TetrisMode::Dig => {
                            app_state.tetris_dig_scores.push(score);
                            app_state.tetris_dig_scores.sort_by(|a, b| b.cmp(a));
                            if app_state.tetris_dig_scores.len() > 100 {
                                app_state.tetris_dig_scores.truncate(100);
                            }
                        }
                        TetrisMode::Survival => {
                            app_state.tetris_survival_scores.push(score);
                            app_state.tetris_survival_scores.sort_by(|a, b| b.cmp(a));
                            if app_state.tetris_survival_scores.len() > 100 {
                                app_state.tetris_survival_scores.truncate(100);
                            }
                        }
                    }

                    state_manager.save(&app_state)?;
                    current_state = StandaloneState::ShowTetrisResults(mode, score, time);
                }
            }
            StandaloneState::PlayingDash(game) => {
                game.update(dt);
                if game.is_finished() {
                    let score = game.score;
                    app_state.dash_best_scores.push(score);
                    app_state.dash_best_scores.sort_by(|a, b| b.cmp(a));
                    if app_state.dash_best_scores.len() > 100 {
                        app_state.dash_best_scores.truncate(100);
                    }
                    state_manager.save(&app_state)?;
                    current_state = StandaloneState::ShowDashResults(score);
                }
            }
            StandaloneState::Playing2048(game) => {
                if game.is_finished() {
                    let score = game.score;
                    let max_tile = game.max_tile();
                    app_state.game_2048_best_scores.push(score);
                    app_state.game_2048_best_scores.sort_by(|a, b| b.cmp(a));
                    if app_state.game_2048_best_scores.len() > 100 {
                        app_state.game_2048_best_scores.truncate(100);
                    }
                    state_manager.save(&app_state)?;
                    current_state = StandaloneState::Show2048Results(score, max_tile);
                }
            }
            StandaloneState::PlayingVsrg(game) => {
                game.update(dt);
                if game.is_finished() {
                    let score = game.score;
                    let accuracy = game.accuracy();
                    let max_combo = game.max_combo;
                    app_state.vsrg_best_scores.push(score);
                    app_state.vsrg_best_scores.sort_by(|a, b| b.cmp(a));
                    if app_state.vsrg_best_scores.len() > 100 {
                        app_state.vsrg_best_scores.truncate(100);
                    }
                    state_manager.save(&app_state)?;
                    current_state = StandaloneState::ShowVsrgResults(score, accuracy, max_combo);
                }
            }
            _ => {}
        }
    }
}
