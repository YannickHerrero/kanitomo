mod crab;
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

use state::StateManager;
use ui::App;

fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let debug_mode = args.iter().any(|arg| arg == "--debug" || arg == "-d");
    let reset_mode = args.iter().any(|arg| arg == "--reset");

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
    let result = run_app(&mut terminal, debug_mode);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, debug_mode: bool) -> Result<()> {
    let mut app = App::new(debug_mode)?;
    app.run(terminal)?;
    Ok(())
}
