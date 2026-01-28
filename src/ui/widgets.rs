use crate::crab::Crab;
use crate::environment::{Environment, TimeOfDay};
use crate::git::{format_time_ago, CommitInfo, GitStats};
use crate::state::{get_today_by_project, get_week_summary, AppState};
use crate::ui::minigames::{BreakoutGame, DashGame, PieceType, SnakeGame, TetrisGame, TetrisMode};
use crate::ui::CrabCatchGame;
use chrono::Datelike;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the crab in the given area
pub fn render_crab(frame: &mut Frame, crab: &Crab, area: Rect) {
    let crab_frame = crab.get_frame();
    let color = crab.color();

    // Calculate position within the area
    let x_offset = crab.position.0 as u16;
    let y_offset = crab.position.1 as u16;

    // Create styled text for the crab
    let lines: Vec<Line> = crab_frame
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let crab_text = Text::from(lines);

    // Position the crab within the area
    let crab_area = Rect {
        x: area.x + x_offset.min(area.width.saturating_sub(22)),
        y: area.y + y_offset.min(area.height.saturating_sub(5)),
        width: 22.min(area.width),
        height: 4.min(area.height),
    };

    let paragraph = Paragraph::new(crab_text);
    frame.render_widget(paragraph, crab_area);
}

/// Render the environment background (sky, sun/moon, clouds, stars)
/// This should be rendered BEFORE the crab
pub fn render_environment_background(frame: &mut Frame, env: &Environment, area: Rect) {
    // Get celestial color based on time of day
    let celestial_color = match env.time_of_day {
        TimeOfDay::Morning => Color::Yellow,
        TimeOfDay::Day => Color::Yellow,
        TimeOfDay::Evening => Color::Rgb(255, 200, 100),
        TimeOfDay::Night => Color::White,
    };

    // Render stars at night
    if env.time_of_day == TimeOfDay::Night {
        for star in &env.stars {
            if star.x < area.width && star.y < area.height {
                let star_area = Rect {
                    x: area.x + star.x,
                    y: area.y + star.y,
                    width: 1,
                    height: 1,
                };
                let star_char =
                    Paragraph::new(star.char.to_string()).style(Style::default().fg(Color::White));
                frame.render_widget(star_char, star_area);
            }
        }
    }

    // Render sun
    if let Some((x, y)) = env.sun_position() {
        render_element(
            frame,
            &crate::environment::elements::SUN
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            x,
            y,
            celestial_color,
            area,
        );
    }

    // Render moon
    if let Some((x, y)) = env.moon_position() {
        render_element(
            frame,
            &crate::environment::elements::MOON_SMALL
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            x,
            y,
            Color::Rgb(200, 200, 220),
            area,
        );
    }

    // Render clouds
    let cloud_color = if env.time_of_day == TimeOfDay::Night {
        Color::DarkGray
    } else {
        Color::Gray
    };
    for cloud in &env.clouds {
        if env.time_of_day == TimeOfDay::Night && !cloud.night_visible {
            continue;
        }
        let cloud_x = cloud.x.round() as i32;
        if cloud_x >= area.width as i32 || cloud_x + cloud.width as i32 <= 0 {
            continue;
        }

        render_element(
            frame,
            &cloud.content,
            cloud_x,
            cloud.y as i32,
            cloud_color,
            area,
        );
    }
}

fn render_element(frame: &mut Frame, content: &[String], x: i32, y: i32, color: Color, area: Rect) {
    for (i, line) in content.iter().enumerate() {
        let y_pos = y + i as i32;
        if y_pos < 0 || y_pos >= area.height as i32 {
            continue;
        }

        if x >= area.width as i32 {
            continue;
        }

        let x_start = x.max(0) as u16;
        let max_width = area.width.saturating_sub(x_start) as usize;
        if max_width == 0 {
            continue;
        }

        let line_start = if x < 0 { (-x) as usize } else { 0 };
        if line_start >= line.len() {
            continue;
        }

        let visible = &line[line_start..];
        let width = visible.len().min(max_width) as u16;
        let visible = &visible[..width as usize];

        let line_area = Rect {
            x: area.x + x_start,
            y: area.y + y_pos as u16,
            width,
            height: 1,
        };

        let line_widget = Paragraph::new(visible.to_string()).style(Style::default().fg(color));
        frame.render_widget(line_widget, line_area);
    }
}

/// Render the ground line at the bottom of the crab area
/// This should be rendered AFTER the crab
pub fn render_ground(frame: &mut Frame, env: &Environment, area: Rect) {
    if area.height == 0 {
        return;
    }

    // Ground is at the very bottom of the crab area
    let ground_y = area.y + area.height.saturating_sub(1);

    // Get ground color based on style and time
    let ground_color = match env.time_of_day {
        TimeOfDay::Night => match env.ground_style {
            crate::environment::GroundStyle::Beach => Color::Rgb(82, 72, 52),
            crate::environment::GroundStyle::Garden => Color::Rgb(28, 64, 40),
            crate::environment::GroundStyle::Rocky => Color::Rgb(70, 74, 78),
            crate::environment::GroundStyle::Minimal => Color::Rgb(48, 72, 44),
        },
        _ => match env.ground_style {
            crate::environment::GroundStyle::Beach => Color::Rgb(200, 183, 132),
            crate::environment::GroundStyle::Garden => Color::Rgb(46, 128, 72),
            crate::environment::GroundStyle::Rocky => Color::Rgb(126, 132, 138),
            crate::environment::GroundStyle::Minimal => Color::Rgb(96, 146, 78),
        },
    };

    // Truncate ground line to fit area width
    let ground_display: String = env.ground_line.chars().take(area.width as usize).collect();

    let ground_area = Rect {
        x: area.x,
        y: ground_y,
        width: area.width,
        height: 1,
    };

    let ground_widget = Paragraph::new(ground_display).style(Style::default().fg(ground_color));
    frame.render_widget(ground_widget, ground_area);
}

/// Render the stats panel
pub fn render_stats(
    frame: &mut Frame,
    stats: &GitStats,
    app_state: &AppState,
    happiness: u8,
    area: Rect,
) {
    let mood = crate::crab::Mood::from_happiness(happiness);

    // Get commits today from tracked history
    let commits_today = get_today_by_project(&app_state.commit_history)
        .iter()
        .map(|(_, _, count)| count)
        .sum::<u32>();

    let mut lines = vec![
        Line::from(vec![
            Span::raw("  Mood: "),
            Span::styled(
                mood.display_name(),
                Style::default()
                    .fg(mood.color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if stats.in_git_repo {
        // Show repo info - single name or count for multiple
        if stats.repo_count == 1 {
            if let Some(name) = stats.repo_names.first() {
                lines.push(Line::from(vec![
                    Span::styled("  Repo: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(name.clone(), Style::default().fg(Color::Cyan)),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("  Watching: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} repos", stats.repo_count),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(" [a]", Style::default().fg(Color::DarkGray)),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("  Commits today: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                commits_today.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" [d]", Style::default().fg(Color::DarkGray)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Streak: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{} day{}",
                    app_state.current_streak,
                    if app_state.current_streak == 1 {
                        ""
                    } else {
                        "s"
                    }
                ),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        if app_state.best_streak > 0 {
            lines.push(Line::from(vec![
                Span::styled("  Best streak: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} days", app_state.best_streak),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("  Last commit: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_time_ago(app_state.last_commit_time),
                Style::default().fg(Color::White),
            ),
        ]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            "  No git repositories found",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  Run in a git folder or",
            Style::default().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  a folder with git projects",
            Style::default().fg(Color::DarkGray),
        )]));
    }

    // Add happiness bar
    lines.push(Line::from(""));
    lines.push(render_happiness_bar(happiness));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Stats ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Render the mini-game selection menu
pub fn render_minigame_menu(frame: &mut Frame, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  MINI GAMES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [1] ", Style::default().fg(Color::Yellow)),
        Span::styled("Crab Catch", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [2] ", Style::default().fg(Color::Yellow)),
        Span::styled("Snake", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [3] ", Style::default().fg(Color::Yellow)),
        Span::styled("Breakout", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [4] ", Style::default().fg(Color::Yellow)),
        Span::styled("Tetris", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [5] ", Style::default().fg(Color::Yellow)),
        Span::styled("Dash", Style::default().fg(Color::White)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [1], [2], [3], [4], or [5] to start",
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 42.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Mini Games ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Calculate the rank of a score in a sorted (descending) score list
/// Ties get the best (lowest) rank number
fn calculate_rank(scores: &[u32], current_score: u32) -> usize {
    scores
        .iter()
        .position(|&s| s <= current_score)
        .map(|p| p + 1)
        .unwrap_or(scores.len() + 1)
}

/// Render the mini-game results screen
pub fn render_minigame_results(frame: &mut Frame, area: Rect, score: u32, app_state: &AppState) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  CRAB CATCH",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    // Top 3 scores
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  TOP SCORES",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    if app_state.minigame_best_scores.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No scores yet",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for (index, best) in app_state.minigame_best_scores.iter().take(3).enumerate() {
            lines.push(Line::from(vec![Span::styled(
                format!("  #{} - {} pts", index + 1, best),
                Style::default().fg(Color::White),
            )]));
        }
    }

    // Current score with rank
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  YOUR SCORE",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    let rank = calculate_rank(&app_state.minigame_best_scores, score);
    let rank_color = match rank {
        1 => Color::Rgb(255, 215, 0),   // Gold
        2 => Color::Rgb(192, 192, 192), // Silver
        3 => Color::Rgb(205, 127, 50),  // Bronze
        _ => Color::Green,
    };
    lines.push(Line::from(vec![Span::styled(
        format!("  #{} - {} pts", rank, score),
        Style::default().fg(rank_color).add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 36.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Results ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the crab catch mini-game
pub fn render_crab_catch(frame: &mut Frame, game: &CrabCatchGame, area: Rect) {
    if area.width == 0 || area.height < 2 {
        return;
    }

    let inner_width = game.bounds.0.min(area.width.saturating_sub(2)).max(1);
    let play_width = inner_width.saturating_add(2);
    let play_x = area.x + (area.width.saturating_sub(play_width) / 2);
    let play_area = Rect {
        x: play_x,
        y: area.y,
        width: play_width,
        height: area.height,
    };

    let crab_y = play_area.y + play_area.height.saturating_sub(1);
    let crab_x = play_area.x + 1 + game.crab_x.max(0) as u16;
    let crab_area = Rect {
        x: crab_x.min(play_area.x + play_area.width.saturating_sub(2)),
        y: crab_y,
        width: game.crab_width.min(play_area.width.saturating_sub(2)),
        height: 1,
    };

    let crab_widget = Paragraph::new(game.crab_sprite()).style(
        Style::default()
            .fg(Color::Rgb(255, 120, 80))
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(crab_widget, crab_area);

    for food in &game.foods {
        let x = play_area.x + 1 + (food.x.round().max(0.0) as u16);
        let y = play_area.y + food.y.round().max(0.0) as u16;
        if x >= play_area.x + play_area.width.saturating_sub(2)
            || y >= play_area.y + play_area.height.saturating_sub(1)
        {
            continue;
        }

        // Use different colors for visual variety
        let food_color = match food.glyph {
            '*' => Color::Yellow,
            '+' => Color::Green,
            'o' => Color::Red,
            '@' => Color::Magenta,
            _ => Color::Cyan,
        };
        render_block_cell(frame, x, y, food_color, Modifier::BOLD);
    }

    let remaining = game.remaining_time().as_secs();
    let hud_lines = vec![
        Line::from(vec![Span::styled(
            "  Crab Catch",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.score.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Time: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}s", remaining),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let hud_width = 20.min(area.width.saturating_sub(2));
    let hud_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: hud_width.max(1),
        height: 5.min(area.height.saturating_sub(2)),
    };

    let hud_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let hud_widget = Paragraph::new(hud_lines).block(hud_block);
    frame.render_widget(hud_widget, hud_area);

    let wall_style = Style::default().fg(Color::DarkGray);
    if play_area.width >= 2 {
        for y in play_area.y..play_area.y + play_area.height {
            let left_wall = Rect {
                x: play_area.x,
                y,
                width: 1,
                height: 1,
            };
            let right_wall = Rect {
                x: play_area.x + play_area.width.saturating_sub(1),
                y,
                width: 1,
                height: 1,
            };
            frame.render_widget(Paragraph::new("|").style(wall_style), left_wall);
            frame.render_widget(Paragraph::new("|").style(wall_style), right_wall);
        }
    }
}

/// Render the snake mini-game
pub fn render_snake_game(frame: &mut Frame, game: &SnakeGame, area: Rect) {
    if area.width == 0 || area.height < 2 {
        return;
    }

    // Calculate play area with 2-char wide blocks
    let inner_width = (game.bounds.0 * 2).min(area.width.saturating_sub(2)).max(2);
    let inner_height = game.bounds.1.min(area.height.saturating_sub(2)).max(1);
    let play_width = inner_width.saturating_add(2);
    let play_height = inner_height.saturating_add(2);
    let play_x = area.x + (area.width.saturating_sub(play_width) / 2);
    let play_y = area.y + (area.height.saturating_sub(play_height) / 2);

    let play_area = Rect {
        x: play_x,
        y: play_y,
        width: play_width,
        height: play_height,
    };

    // Draw border
    let border_style = Style::default().fg(Color::DarkGray);

    // Top and bottom borders
    let horizontal_border = "-".repeat(play_width as usize);
    let top_border_area = Rect {
        x: play_area.x,
        y: play_area.y,
        width: play_width,
        height: 1,
    };
    let bottom_border_area = Rect {
        x: play_area.x,
        y: play_area.y + play_height.saturating_sub(1),
        width: play_width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(horizontal_border.clone()).style(border_style),
        top_border_area,
    );
    frame.render_widget(
        Paragraph::new(horizontal_border).style(border_style),
        bottom_border_area,
    );

    // Left and right borders
    for dy in 1..play_height.saturating_sub(1) {
        let left_wall = Rect {
            x: play_area.x,
            y: play_area.y + dy,
            width: 1,
            height: 1,
        };
        let right_wall = Rect {
            x: play_area.x + play_width.saturating_sub(1),
            y: play_area.y + dy,
            width: 1,
            height: 1,
        };
        frame.render_widget(Paragraph::new("|").style(border_style), left_wall);
        frame.render_widget(Paragraph::new("|").style(border_style), right_wall);
    }

    // Draw food as block
    let food_x = play_area.x + 1 + (game.food.0 as u16 * 2);
    let food_y = play_area.y + 1 + game.food.1 as u16;
    if food_x + 2 <= play_area.x + play_width.saturating_sub(1)
        && food_y < play_area.y + play_height.saturating_sub(1)
    {
        render_block_cell(frame, food_x, food_y, Color::Green, Modifier::BOLD);
    }

    // Draw snake as blocks
    for (i, segment) in game.snake.iter().enumerate() {
        let seg_x = play_area.x + 1 + (segment.0 as u16 * 2);
        let seg_y = play_area.y + 1 + segment.1 as u16;

        if seg_x + 2 <= play_area.x + play_width.saturating_sub(1)
            && seg_y < play_area.y + play_height.saturating_sub(1)
        {
            let (color, modifier) = if i == 0 {
                // Head - brighter
                (Color::Rgb(255, 120, 80), Modifier::BOLD)
            } else {
                // Body - darker
                (Color::Rgb(200, 100, 60), Modifier::empty())
            };

            render_block_cell(frame, seg_x, seg_y, color, modifier);
        }
    }

    // Draw HUD
    let hud_lines = vec![
        Line::from(vec![Span::styled(
            "  Snake",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.score.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Length: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.snake.len().to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let hud_width = 20.min(area.width.saturating_sub(2));
    let hud_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: hud_width.max(1),
        height: 5.min(area.height.saturating_sub(2)),
    };

    let hud_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let hud_widget = Paragraph::new(hud_lines).block(hud_block);
    frame.render_widget(hud_widget, hud_area);
}

/// Render the snake game results screen
pub fn render_snake_results(frame: &mut Frame, area: Rect, score: u32, app_state: &AppState) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  SNAKE",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    // Top 3 scores
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  TOP SCORES",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    if app_state.snake_best_scores.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No scores yet",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for (index, best) in app_state.snake_best_scores.iter().take(3).enumerate() {
            lines.push(Line::from(vec![Span::styled(
                format!("  #{} - {} pts", index + 1, best),
                Style::default().fg(Color::White),
            )]));
        }
    }

    // Current score with rank
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  YOUR SCORE",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    let rank = calculate_rank(&app_state.snake_best_scores, score);
    let rank_color = match rank {
        1 => Color::Rgb(255, 215, 0),   // Gold
        2 => Color::Rgb(192, 192, 192), // Silver
        3 => Color::Rgb(205, 127, 50),  // Bronze
        _ => Color::Green,
    };
    lines.push(Line::from(vec![Span::styled(
        format!("  #{} - {} pts", rank, score),
        Style::default().fg(rank_color).add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 36.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Results ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the breakout mini-game
pub fn render_breakout_game(frame: &mut Frame, game: &BreakoutGame, area: Rect) {
    if area.width == 0 || area.height < 4 {
        return;
    }

    let inner_width = game.bounds.0.min(area.width.saturating_sub(2)).max(1);
    let inner_height = game.bounds.1.min(area.height.saturating_sub(2)).max(1);
    let play_width = inner_width.saturating_add(2);
    let play_height = inner_height.saturating_add(2);
    let play_x = area.x + (area.width.saturating_sub(play_width) / 2);
    let play_y = area.y + (area.height.saturating_sub(play_height) / 2);

    let play_area = Rect {
        x: play_x,
        y: play_y,
        width: play_width,
        height: play_height,
    };

    // Draw border
    let border_style = Style::default().fg(Color::DarkGray);

    // Top and bottom borders
    let horizontal_border = "-".repeat(play_width as usize);
    let top_border_area = Rect {
        x: play_area.x,
        y: play_area.y,
        width: play_width,
        height: 1,
    };
    let bottom_border_area = Rect {
        x: play_area.x,
        y: play_area.y + play_height.saturating_sub(1),
        width: play_width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(horizontal_border.clone()).style(border_style),
        top_border_area,
    );
    frame.render_widget(
        Paragraph::new(horizontal_border).style(border_style),
        bottom_border_area,
    );

    // Left and right borders
    for dy in 1..play_height.saturating_sub(1) {
        let left_wall = Rect {
            x: play_area.x,
            y: play_area.y + dy,
            width: 1,
            height: 1,
        };
        let right_wall = Rect {
            x: play_area.x + play_width.saturating_sub(1),
            y: play_area.y + dy,
            width: 1,
            height: 1,
        };
        frame.render_widget(Paragraph::new("|").style(border_style), left_wall);
        frame.render_widget(Paragraph::new("|").style(border_style), right_wall);
    }

    // Draw bricks as blocks
    for brick in &game.bricks {
        let brick_x = play_area.x + 1 + brick.x;
        let brick_y = play_area.y + 1 + brick.y;

        if brick_x + brick.width > play_area.x + play_width.saturating_sub(1)
            || brick_y >= play_area.y + play_height.saturating_sub(1)
        {
            continue;
        }

        // Color based on point value
        let brick_color = match brick.points {
            50 => Color::Red,
            40 => Color::LightRed,
            30 => Color::Yellow,
            20 => Color::Green,
            _ => Color::Cyan,
        };

        // Render brick as solid blocks
        for bx in 0..brick.width {
            if brick_x + bx < play_area.x + play_width.saturating_sub(1) {
                render_block_cell(frame, brick_x + bx, brick_y, brick_color, Modifier::BOLD);
            }
        }
    }

    // Draw ball as block
    let ball_x = play_area.x + 1 + game.ball_pos.0.round().max(0.0) as u16;
    let ball_y = play_area.y + 1 + game.ball_pos.1.round().max(0.0) as u16;
    if ball_x + 2 <= play_area.x + play_width.saturating_sub(1)
        && ball_y < play_area.y + play_height.saturating_sub(1)
    {
        render_block_cell(frame, ball_x, ball_y, Color::White, Modifier::BOLD);
    }

    // Draw paddle as blocks
    let paddle_x = play_area.x + 1 + game.paddle_x.round().max(0.0) as u16;
    let paddle_y = play_area.y + play_height.saturating_sub(2);
    if paddle_x + game.paddle_width <= play_area.x + play_width.saturating_sub(1) {
        for px in 0..game.paddle_width {
            if paddle_x + px + 2 <= play_area.x + play_width.saturating_sub(1) {
                render_block_cell(
                    frame,
                    paddle_x + px,
                    paddle_y,
                    Color::Rgb(255, 120, 80),
                    Modifier::BOLD,
                );
            }
        }
    }

    // Draw HUD
    let lives_str: String = (0..game.lives).map(|_| "♥").collect();
    let hud_lines = vec![
        Line::from(vec![Span::styled(
            "  Breakout",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.score.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Lives: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                lives_str,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let hud_width = 20.min(area.width.saturating_sub(2));
    let hud_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: hud_width.max(1),
        height: 5.min(area.height.saturating_sub(2)),
    };

    let hud_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let hud_widget = Paragraph::new(hud_lines).block(hud_block);
    frame.render_widget(hud_widget, hud_area);

    // Show launch prompt if ball not launched
    if !game.ball_launched {
        let prompt = "Press SPACE to launch";
        let prompt_x = play_area.x + (play_width.saturating_sub(prompt.len() as u16)) / 2;
        let prompt_y = play_area.y + play_height / 2;
        let prompt_area = Rect {
            x: prompt_x,
            y: prompt_y,
            width: prompt.len() as u16,
            height: 1,
        };
        let prompt_widget = Paragraph::new(prompt).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(prompt_widget, prompt_area);
    }
}

/// Render the dash mini-game
pub fn render_dash_game(frame: &mut Frame, game: &DashGame, area: Rect) {
    if area.width == 0 || area.height < 4 {
        return;
    }

    let inner_width = (game.bounds.0 * 2).min(area.width.saturating_sub(2)).max(2);
    let inner_height = game.bounds.1.min(area.height.saturating_sub(2)).max(1);
    let play_width = inner_width.saturating_add(2);
    let play_height = inner_height.saturating_add(2);
    let play_x = area.x + (area.width.saturating_sub(play_width) / 2);
    let play_y = area.y + (area.height.saturating_sub(play_height) / 2);

    let play_area = Rect {
        x: play_x,
        y: play_y,
        width: play_width,
        height: play_height,
    };

    let border_style = Style::default().fg(Color::DarkGray);
    let horizontal_border = "-".repeat(play_width as usize);

    frame.render_widget(
        Paragraph::new(horizontal_border.clone()).style(border_style),
        Rect {
            x: play_area.x,
            y: play_area.y,
            width: play_width,
            height: 1,
        },
    );
    frame.render_widget(
        Paragraph::new(horizontal_border).style(border_style),
        Rect {
            x: play_area.x,
            y: play_area.y + play_height.saturating_sub(1),
            width: play_width,
            height: 1,
        },
    );

    for dy in 1..play_height.saturating_sub(1) {
        let left_wall = Rect {
            x: play_area.x,
            y: play_area.y + dy,
            width: 1,
            height: 1,
        };
        let right_wall = Rect {
            x: play_area.x + play_width.saturating_sub(1),
            y: play_area.y + dy,
            width: 1,
            height: 1,
        };
        frame.render_widget(Paragraph::new("|").style(border_style), left_wall);
        frame.render_widget(Paragraph::new("|").style(border_style), right_wall);
    }

    let ground_y = play_area.y + play_height.saturating_sub(2);
    let ground_line = "=".repeat(inner_width as usize);
    frame.render_widget(
        Paragraph::new(ground_line).style(Style::default().fg(Color::DarkGray)),
        Rect {
            x: play_area.x + 1,
            y: ground_y,
            width: inner_width,
            height: 1,
        },
    );

    let obstacle_color = Color::Red;
    let ground_cell_y = game.bounds.1 as i32 - 1;
    for obstacle in &game.obstacles {
        let obs_x = obstacle.x.floor() as i32;
        let obs_width = obstacle.width.ceil() as i32;
        let obs_height = obstacle.height.round() as i32;

        for dx in 0..obs_width {
            for dy in 0..obs_height {
                let cell_x = obs_x + dx;
                let cell_y = ground_cell_y - dy;

                if cell_x < 0
                    || cell_y < 0
                    || cell_x >= game.bounds.0 as i32
                    || cell_y >= game.bounds.1 as i32
                {
                    continue;
                }

                let x = play_area.x + 1 + (cell_x as u16 * 2);
                let y = play_area.y + 1 + cell_y as u16;
                if x + 2 <= play_area.x + play_width.saturating_sub(1)
                    && y < play_area.y + play_height.saturating_sub(1)
                {
                    render_block_cell(frame, x, y, obstacle_color, Modifier::BOLD);
                }
            }
        }
    }

    let player_x = game.player_x().round() as i32;
    let player_y = game.player_y.round() as i32;
    if player_x >= 0
        && player_y >= 0
        && player_x < game.bounds.0 as i32
        && player_y < game.bounds.1 as i32
    {
        let x = play_area.x + 1 + (player_x as u16 * 2);
        let y = play_area.y + 1 + player_y as u16;
        if x + 2 <= play_area.x + play_width.saturating_sub(1)
            && y < play_area.y + play_height.saturating_sub(1)
        {
            render_block_cell(frame, x, y, Color::Rgb(120, 200, 255), Modifier::BOLD);
        }
    }

    let hud_lines = vec![
        Line::from(vec![Span::styled(
            "  Dash",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.score.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Speed: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}", game.speed),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let hud_width = 20.min(area.width.saturating_sub(2));
    let hud_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: hud_width.max(1),
        height: 5.min(area.height.saturating_sub(2)),
    };

    let hud_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let hud_widget = Paragraph::new(hud_lines).block(hud_block);
    frame.render_widget(hud_widget, hud_area);
}

/// Render the breakout game results screen
pub fn render_breakout_results(
    frame: &mut Frame,
    area: Rect,
    score: u32,
    victory: bool,
    app_state: &AppState,
) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    let title = if victory {
        "  BREAKOUT - VICTORY!"
    } else {
        "  BREAKOUT - GAME OVER"
    };
    let title_color = if victory { Color::Green } else { Color::Red };

    lines.push(Line::from(vec![Span::styled(
        title,
        Style::default()
            .fg(title_color)
            .add_modifier(Modifier::BOLD),
    )]));

    // Top 3 scores
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  TOP SCORES",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    if app_state.breakout_best_scores.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No scores yet",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for (index, best) in app_state.breakout_best_scores.iter().take(3).enumerate() {
            lines.push(Line::from(vec![Span::styled(
                format!("  #{} - {} pts", index + 1, best),
                Style::default().fg(Color::White),
            )]));
        }
    }

    // Current score with rank
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  YOUR SCORE",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    let rank = calculate_rank(&app_state.breakout_best_scores, score);
    let rank_color = match rank {
        1 => Color::Rgb(255, 215, 0),   // Gold
        2 => Color::Rgb(192, 192, 192), // Silver
        3 => Color::Rgb(205, 127, 50),  // Bronze
        _ => Color::Green,
    };
    lines.push(Line::from(vec![Span::styled(
        format!("  #{} - {} pts", rank, score),
        Style::default().fg(rank_color).add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 36.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Results ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the dash game results screen
pub fn render_dash_results(frame: &mut Frame, area: Rect, score: u32, app_state: &AppState) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  DASH",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  TOP SCORES",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    if app_state.dash_best_scores.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No scores yet",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for (index, best) in app_state.dash_best_scores.iter().take(3).enumerate() {
            lines.push(Line::from(vec![Span::styled(
                format!("  #{} - {} pts", index + 1, best),
                Style::default().fg(Color::White),
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  YOUR SCORE",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    let rank = calculate_rank(&app_state.dash_best_scores, score);
    let rank_color = match rank {
        1 => Color::Rgb(255, 215, 0),
        2 => Color::Rgb(192, 192, 192),
        3 => Color::Rgb(205, 127, 50),
        _ => Color::Green,
    };
    lines.push(Line::from(vec![Span::styled(
        format!("  #{} - {} pts", rank, score),
        Style::default().fg(rank_color).add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 36.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Results ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the Tetris game
pub fn render_tetris_game(frame: &mut Frame, game: &TetrisGame, area: Rect) {
    if area.width < 30 || area.height < 22 {
        return;
    }

    // Calculate centered play area (2 chars per cell for solid blocks)
    let grid_width = 22; // 10 cells * 2 chars + 2 borders
    let grid_height = 22; // 20 cells + 2 borders
    let play_x = area.x + (area.width.saturating_sub(grid_width) / 2);
    let play_y = area.y + (area.height.saturating_sub(grid_height) / 2);

    let play_area = Rect {
        x: play_x,
        y: play_y,
        width: grid_width,
        height: grid_height,
    };

    // Draw border
    let border_style = Style::default().fg(Color::DarkGray);
    let horizontal_border = "-".repeat(grid_width as usize);

    // Top border
    frame.render_widget(
        Paragraph::new(horizontal_border.clone()).style(border_style),
        Rect {
            x: play_area.x,
            y: play_area.y,
            width: grid_width,
            height: 1,
        },
    );

    // Bottom border
    frame.render_widget(
        Paragraph::new(horizontal_border).style(border_style),
        Rect {
            x: play_area.x,
            y: play_area.y + grid_height - 1,
            width: grid_width,
            height: 1,
        },
    );

    // Side borders
    for dy in 1..grid_height - 1 {
        frame.render_widget(
            Paragraph::new("|").style(border_style),
            Rect {
                x: play_area.x,
                y: play_area.y + dy,
                width: 1,
                height: 1,
            },
        );
        frame.render_widget(
            Paragraph::new("|").style(border_style),
            Rect {
                x: play_area.x + grid_width - 1,
                y: play_area.y + dy,
                width: 1,
                height: 1,
            },
        );
    }

    // Draw dotted background grid for empty cells
    for y in 0..20 {
        for x in 0..10 {
            if game.grid[y][x].is_none() {
                frame.render_widget(
                    Paragraph::new(" ·").style(Style::default().fg(Color::DarkGray)),
                    Rect {
                        x: play_area.x + 1 + (x as u16 * 2),
                        y: play_area.y + 1 + y as u16,
                        width: 2,
                        height: 1,
                    },
                );
            }
        }
    }

    // Draw locked blocks from grid (using solid blocks with bg color like samtay/tetris)
    for (y, row) in game.grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if let Some(piece_type) = cell {
                let color = piece_color(*piece_type);
                frame.render_widget(
                    Paragraph::new("  ").style(Style::default().bg(color)),
                    Rect {
                        x: play_area.x + 1 + (x as u16 * 2),
                        y: play_area.y + 1 + y as u16,
                        width: 2,
                        height: 1,
                    },
                );
            }
        }
    }

    // Draw ghost piece (where piece would land)
    if let Some(ghost_blocks) = game.get_ghost_position() {
        if let Some(ref piece) = game.current_piece {
            let ghost_color = piece_color(piece.piece_type);
            for (x, y) in ghost_blocks {
                if y >= 0 && x >= 0 && x < 10 && y < 20 {
                    // Only draw ghost if it's different from current piece position
                    let is_current_piece =
                        piece.blocks().iter().any(|(px, py)| *px == x && *py == y);
                    if !is_current_piece {
                        frame.render_widget(
                            Paragraph::new("[]").style(
                                Style::default().fg(ghost_color).add_modifier(Modifier::DIM),
                            ),
                            Rect {
                                x: play_area.x + 1 + (x as u16 * 2),
                                y: play_area.y + 1 + y as u16,
                                width: 2,
                                height: 1,
                            },
                        );
                    }
                }
            }
        }
    }

    // Draw current piece (using solid blocks with bg color)
    if let Some(ref piece) = game.current_piece {
        let color = piece_color(piece.piece_type);
        for (x, y) in piece.blocks() {
            if y >= 0 && x >= 0 && x < 10 && y < 20 {
                frame.render_widget(
                    Paragraph::new("  ").style(Style::default().bg(color)),
                    Rect {
                        x: play_area.x + 1 + (x as u16 * 2),
                        y: play_area.y + 1 + y as u16,
                        width: 2,
                        height: 1,
                    },
                );
            }
        }
    }

    // Draw HUD on the left
    let mut hud_lines = vec![
        Line::from(vec![Span::styled(
            format!("  {}", game.mode.name()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.score.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    // Mode-specific HUD
    if game.mode == TetrisMode::Sprint {
        hud_lines.push(Line::from(vec![
            Span::styled("  Time: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}s", game.elapsed_time),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        hud_lines.push(Line::from(vec![
            Span::styled("  Lines: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.lines_cleared, game.target_lines),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    } else {
        if game.mode != TetrisMode::Zen {
            hud_lines.push(Line::from(vec![
                Span::styled("  Level: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    game.level.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        hud_lines.push(Line::from(vec![
            Span::styled("  Lines: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.lines_cleared.to_string(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let hud_width = 20.min(area.width.saturating_sub(2));
    let hud_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: hud_width.max(1),
        height: 6.min(area.height.saturating_sub(2)),
    };

    let hud_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(hud_lines).block(hud_block), hud_area);

    // Draw next piece preview on the right
    let preview_x = play_area.x + grid_width + 2;
    if preview_x + 12 < area.x + area.width {
        let preview_lines = vec![
            Line::from(vec![Span::styled(
                "  Next",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        let preview_area = Rect {
            x: preview_x,
            y: area.y + 1,
            width: 12,
            height: 8,
        };

        let preview_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(
            Paragraph::new(preview_lines).block(preview_block),
            preview_area,
        );

        // Draw the next piece shape (centered in preview box with solid blocks)
        let shape = game.next_piece.shape();
        let color = piece_color(game.next_piece);
        for (dy, row) in shape.iter().enumerate() {
            for (dx, filled) in row.iter().enumerate() {
                let filled = *filled;
                if filled && dx < 4 && dy < 4 {
                    frame.render_widget(
                        Paragraph::new("  ").style(Style::default().bg(color)),
                        Rect {
                            x: preview_x + 2 + (dx as u16 * 2),
                            y: preview_area.y + 2 + dy as u16,
                            width: 2,
                            height: 1,
                        },
                    );
                }
            }
        }

        // Draw hold piece below the next piece preview
        let hold_y = preview_area.y + preview_area.height + 1;
        let hold_lines = vec![
            Line::from(vec![Span::styled(
                "  Hold",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        let hold_area = Rect {
            x: preview_x,
            y: hold_y,
            width: 12,
            height: 8,
        };

        let hold_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(Paragraph::new(hold_lines).block(hold_block), hold_area);

        // Draw the held piece if any
        if let Some(held_type) = game.hold_piece {
            let shape = held_type.shape();
            let color = piece_color(held_type);
            let hold_style = if game.can_hold {
                Style::default().bg(color)
            } else {
                // Dim if can't hold
                Style::default().bg(color).add_modifier(Modifier::DIM)
            };

            for (dy, row) in shape.iter().enumerate() {
                for (dx, filled) in row.iter().enumerate() {
                    let filled = *filled;
                    if filled && dx < 4 && dy < 4 {
                        frame.render_widget(
                            Paragraph::new("  ").style(hold_style),
                            Rect {
                                x: preview_x + 2 + (dx as u16 * 2),
                                y: hold_area.y + 2 + dy as u16,
                                width: 2,
                                height: 1,
                            },
                        );
                    }
                }
            }
        }
    }
}

/// Render a solid block cell (2 chars wide) with background color
fn render_block_cell(frame: &mut Frame, x: u16, y: u16, color: Color, modifier: Modifier) {
    let block_area = Rect {
        x,
        y,
        width: 2,
        height: 1,
    };
    let block_widget =
        Paragraph::new("  ").style(Style::default().bg(color).add_modifier(modifier));
    frame.render_widget(block_widget, block_area);
}

/// Helper function to get color for a piece type (matching samtay/tetris)
fn piece_color(piece_type: PieceType) -> Color {
    match piece_type {
        PieceType::I => Color::Cyan,
        PieceType::O => Color::Yellow,
        PieceType::T => Color::Magenta,
        PieceType::S => Color::Green,
        PieceType::Z => Color::Red,
        PieceType::J => Color::Blue,
        PieceType::L => Color::White, // White like samtay/tetris
    }
}

/// Render the Tetris mode selection menu
pub fn render_tetris_mode_menu(frame: &mut Frame, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  TETRIS MODES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [1] ", Style::default().fg(Color::Yellow)),
        Span::styled("Normal", Style::default().fg(Color::White)),
        Span::styled(" - Classic mode", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [2] ", Style::default().fg(Color::Yellow)),
        Span::styled("Sprint", Style::default().fg(Color::White)),
        Span::styled(" - Clear 40 lines", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [3] ", Style::default().fg(Color::Yellow)),
        Span::styled("Zen", Style::default().fg(Color::White)),
        Span::styled(" - No speed increase", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [4] ", Style::default().fg(Color::Yellow)),
        Span::styled("Dig", Style::default().fg(Color::White)),
        Span::styled(" - Clear garbage", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [5] ", Style::default().fg(Color::Yellow)),
        Span::styled("Survival", Style::default().fg(Color::White)),
        Span::styled(" - Intense speed", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press number to select mode",
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 45.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Select Mode ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the Tetris game results screen
pub fn render_tetris_results(
    frame: &mut Frame,
    area: Rect,
    mode: TetrisMode,
    score: u32,
    time: f32,
    app_state: &AppState,
) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        format!("  TETRIS - {}", mode.name().to_uppercase()),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    // Get leaderboard based on mode
    let (leaderboard_title, leaderboard, your_result, rank_text) = match mode {
        TetrisMode::Sprint => {
            let times = &app_state.tetris_sprint_times;
            let rank = times
                .iter()
                .position(|&t| t >= time)
                .map(|p| p + 1)
                .unwrap_or(times.len() + 1);
            let rank_color = match rank {
                1 => Color::Rgb(255, 215, 0),
                2 => Color::Rgb(192, 192, 192),
                3 => Color::Rgb(205, 127, 50),
                _ => Color::Green,
            };
            (
                "TOP TIMES",
                times
                    .iter()
                    .take(3)
                    .map(|&t| {
                        format!(
                            "  #{} - {:.2}s",
                            times.iter().position(|&x| x == t).unwrap() + 1,
                            t
                        )
                    })
                    .collect::<Vec<_>>(),
                format!("  #{} - {:.2}s", rank, time),
                rank_color,
            )
        }
        _ => {
            let scores = match mode {
                TetrisMode::Normal => &app_state.tetris_normal_scores,
                TetrisMode::Zen => &app_state.tetris_zen_scores,
                TetrisMode::Dig => &app_state.tetris_dig_scores,
                TetrisMode::Survival => &app_state.tetris_survival_scores,
                _ => &vec![],
            };
            let rank = calculate_rank(scores, score);
            let rank_color = match rank {
                1 => Color::Rgb(255, 215, 0),
                2 => Color::Rgb(192, 192, 192),
                3 => Color::Rgb(205, 127, 50),
                _ => Color::Green,
            };
            (
                "TOP SCORES",
                scores
                    .iter()
                    .take(3)
                    .enumerate()
                    .map(|(i, s)| format!("  #{} - {} pts", i + 1, s))
                    .collect::<Vec<_>>(),
                format!("  #{} - {} pts", rank, score),
                rank_color,
            )
        }
    };

    // Top 3 leaderboard
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", leaderboard_title),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    if leaderboard.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No records yet",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for entry in leaderboard {
            lines.push(Line::from(vec![Span::styled(
                entry,
                Style::default().fg(Color::White),
            )]));
        }
    }

    // Your result
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        if mode == TetrisMode::Sprint {
            "  YOUR TIME"
        } else {
            "  YOUR SCORE"
        },
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        your_result,
        Style::default().fg(rank_text).add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 40.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Results ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render a happiness bar
fn render_happiness_bar(happiness: u8) -> Line<'static> {
    let bar_width = 20;
    let filled = (happiness as usize * bar_width) / 100;
    let empty = bar_width - filled;

    let color = match happiness {
        90..=100 => Color::Magenta,
        70..=89 => Color::Green,
        40..=69 => Color::Yellow,
        20..=39 => Color::LightRed,
        _ => Color::Red,
    };

    Line::from(vec![
        Span::styled("  Happiness: [", Style::default().fg(Color::DarkGray)),
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
        Span::styled("] ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}%", happiness),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

/// Render the help overlay window
pub fn render_help_overlay(
    frame: &mut Frame,
    area: Rect,
    debug_mode: bool,
    multi_repo: bool,
    show_stats: bool,
) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  CONTROLS",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    lines.push(Line::from(vec![
        Span::styled("  [q] ", Style::default().fg(Color::Yellow)),
        Span::styled("quit", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [?] ", Style::default().fg(Color::Yellow)),
        Span::styled("close help", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [r] ", Style::default().fg(Color::Yellow)),
        Span::styled("refresh", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [space] ", Style::default().fg(Color::Yellow)),
        Span::styled("mini games", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [d] ", Style::default().fg(Color::Yellow)),
        Span::styled("details", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [s] ", Style::default().fg(Color::Yellow)),
        Span::styled(
            if show_stats {
                "hide stats"
            } else {
                "show stats"
            },
            Style::default().fg(Color::White),
        ),
    ]));

    if multi_repo {
        lines.push(Line::from(vec![
            Span::styled("  [a] ", Style::default().fg(Color::Yellow)),
            Span::styled("repos", Style::default().fg(Color::White)),
        ]));
    }

    if debug_mode {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "  DEBUG",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![
            Span::styled("  [f] ", Style::default().fg(Color::Yellow)),
            Span::styled("feed", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [p] ", Style::default().fg(Color::Yellow)),
            Span::styled("punish", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [g] ", Style::default().fg(Color::Yellow)),
            Span::styled("ground", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [x] ", Style::default().fg(Color::Yellow)),
            Span::styled("freeze", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [c] ", Style::default().fg(Color::Yellow)),
            Span::styled("fast cycle", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [m] ", Style::default().fg(Color::Yellow)),
            Span::styled("commit picker", Style::default().fg(Color::White)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  MINI GAME",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![
        Span::styled("  arrows/hjkl ", Style::default().fg(Color::Yellow)),
        Span::styled("move", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [space]/[up] ", Style::default().fg(Color::Yellow)),
        Span::styled("jump (dash)", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [q] ", Style::default().fg(Color::Yellow)),
        Span::styled("exit game", Style::default().fg(Color::White)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [?] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 42.min(area.width.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the title bar with Kani's message
pub fn render_title(frame: &mut Frame, area: Rect, message: &str) {
    let title = Line::from(vec![
        Span::styled("Kani: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("\"{}\"", message),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        ),
    ]);

    let paragraph = Paragraph::new(title).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Render the repo list overlay
pub fn render_repo_list(frame: &mut Frame, repo_names: &[String], area: Rect) {
    // Calculate overlay size - center it in the screen
    let overlay_width = 40.min(area.width.saturating_sub(4));
    let overlay_height = (repo_names.len() as u16 + 4).min(area.height.saturating_sub(4));

    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    // Clear the area behind the overlay
    frame.render_widget(Clear, overlay_area);

    // Build the list of repos
    let mut lines: Vec<Line> = vec![Line::from("")];

    for name in repo_names {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("  ", Style::default().fg(Color::Green)),
            Span::styled(name.clone(), Style::default().fg(Color::White)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [a] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Watched Repositories ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the activity details overlay
pub fn render_details_overlay(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let today_by_project = get_today_by_project(&app_state.commit_history);
    let week_summary = get_week_summary(&app_state.commit_history);

    // Calculate required height
    let today_lines = today_by_project.len().max(1) + 3; // projects + header + total + blank
    let week_lines = week_summary.len() + 2; // days + header + total
    let footer_lines = 2;
    let total_height = (today_lines + week_lines + footer_lines + 4) as u16; // +4 for borders and spacing

    let overlay_width = 45.min(area.width.saturating_sub(4));
    let overlay_height = total_height.min(area.height.saturating_sub(4));

    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    // Clear the area behind the overlay
    frame.render_widget(Clear, overlay_area);

    let mut lines: Vec<Line> = vec![];

    // Today section
    let today_total: u32 = today_by_project.iter().map(|(_, _, c)| c).sum();
    lines.push(Line::from(vec![Span::styled(
        format!("  TODAY ({})", chrono::Local::now().format("%b %d")),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    if today_by_project.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "    No commits yet today",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        let max_count = today_by_project
            .iter()
            .map(|(_, _, c)| *c)
            .max()
            .unwrap_or(1);
        for (_id, name, count) in &today_by_project {
            let bar_len = (*count as usize * 10) / max_count.max(1) as usize;
            let bar = "█".repeat(bar_len.max(1));
            let padding = " ".repeat(10 - bar_len.max(1));

            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<16}", truncate_str(name, 16)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(bar, Style::default().fg(Color::Green)),
                Span::styled(padding, Style::default()),
                Span::styled(
                    format!(" {} commit{}", count, if *count == 1 { "" } else { "s" }),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    lines.push(Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled("─".repeat(30), Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    Total", Style::default().fg(Color::White)),
        Span::styled(
            format!(
                "            {} commit{}",
                today_total,
                if today_total == 1 { "" } else { "s" }
            ),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));

    // This week section
    lines.push(Line::from(vec![Span::styled(
        "  THIS WEEK",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));

    let week_total: u32 = week_summary.iter().map(|(_, c)| c).sum();
    let max_day_count = week_summary.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let today_date = chrono::Local::now().date_naive();

    for (date, count) in &week_summary {
        let day_name = match date.weekday() {
            chrono::Weekday::Mon => "Mon",
            chrono::Weekday::Tue => "Tue",
            chrono::Weekday::Wed => "Wed",
            chrono::Weekday::Thu => "Thu",
            chrono::Weekday::Fri => "Fri",
            chrono::Weekday::Sat => "Sat",
            chrono::Weekday::Sun => "Sun",
        };

        let bar_len = if max_day_count > 0 {
            (*count as usize * 10) / max_day_count as usize
        } else {
            0
        };
        let bar = if *count > 0 {
            "█".repeat(bar_len.max(1))
        } else {
            "░".to_string()
        };
        let padding = " ".repeat(10usize.saturating_sub(bar.chars().count()));

        let is_today = *date == today_date;
        let day_style = if is_today {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let suffix = if is_today { " (today)" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(format!("    {:<6}", day_name), day_style),
            Span::styled(bar, Style::default().fg(Color::Magenta)),
            Span::styled(padding, Style::default()),
            Span::styled(
                format!(
                    " {} commit{}{}",
                    count,
                    if *count == 1 { "" } else { "s" },
                    suffix
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled("─".repeat(30), Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    Week total", Style::default().fg(Color::White)),
        Span::styled(
            format!(
                "       {} commit{}",
                week_total,
                if week_total == 1 { "" } else { "s" }
            ),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [d] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Activity Details ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Render the commit picker overlay (debug mode only)
pub fn render_commit_picker(
    frame: &mut Frame,
    commits: &[CommitInfo],
    selected: usize,
    scroll: usize,
    is_tracked_fn: impl Fn(&str) -> bool,
    area: Rect,
) {
    let overlay_width = 70.min(area.width.saturating_sub(4));
    let overlay_height = 22.min(area.height.saturating_sub(4));
    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    // Clear the area behind the overlay
    frame.render_widget(Clear, overlay_area);

    let mut lines: Vec<Line> = vec![];

    // Header
    lines.push(Line::from(vec![Span::styled(
        "  COMMIT PICKER (Debug)",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    if commits.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No commits found in this repository",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        // Calculate visible items (leave room for header, footer, and instructions)
        let visible_items = (overlay_height as usize).saturating_sub(8);
        let end_index = (scroll + visible_items).min(commits.len());

        for (i, commit) in commits.iter().enumerate().skip(scroll).take(visible_items) {
            let is_selected = i == selected;
            let is_tracked = is_tracked_fn(&commit.hash);

            // Checkbox
            let checkbox = if is_tracked { "[x]" } else { "[ ]" };

            // Format time ago
            let time_ago = format_time_ago(Some(commit.timestamp));

            // Truncate message to fit
            let max_msg_len = (overlay_width as usize).saturating_sub(30);
            let message = truncate_str(&commit.message, max_msg_len);

            // Build the line with different styles based on selection
            let (prefix_style, hash_style, msg_style, time_style) = if is_selected {
                (
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::default().fg(if is_tracked {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                    Style::default().fg(Color::Cyan),
                    Style::default().fg(Color::White),
                    Style::default().fg(Color::DarkGray),
                )
            };

            let prefix = if is_selected { "> " } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled(checkbox, prefix_style),
                Span::styled(" ", Style::default()),
                Span::styled(&commit.short_hash, hash_style),
                Span::styled(" ", Style::default()),
                Span::styled(message, msg_style),
                Span::styled(format!(" {}", time_ago), time_style),
            ]));
        }

        // Show scroll indicator if there are more items
        if commits.len() > visible_items {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "  Showing {}-{} of {} commits",
                    scroll + 1,
                    end_index,
                    commits.len()
                ),
                Style::default().fg(Color::DarkGray),
            )]));
        }
    }

    // Footer with instructions
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::styled(" toggle  ", Style::default().fg(Color::DarkGray)),
        Span::styled("m/Esc", Style::default().fg(Color::Yellow)),
        Span::styled(" close", Style::default().fg(Color::DarkGray)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(Span::styled(
            " Commit Picker ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, overlay_area);
}

/// Truncate a string to a maximum length, adding ".." if truncated
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}..", &s[..max_len - 2])
    }
}

/// Helper function to create a centered rect
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1]);

    horizontal[1]
}
