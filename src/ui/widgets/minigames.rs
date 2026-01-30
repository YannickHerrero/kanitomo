use crate::state::AppState;
use crate::ui::minigames::vsrg::{VsrgJudgment, VsrgLaneFlashKind};
use crate::ui::minigames::{
    vsrg_lane_count, BreakoutGame, DashGame, Game2048, SnakeGame, TetrisGame, TetrisMode, VsrgGame,
};
use crate::ui::CrabCatchGame;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::helpers::{
    calculate_rank, centered_rect, piece_color, render_block_cell, tile_colors, tile_label,
};

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
        Span::styled("Kanitomo", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [2] ", Style::default().fg(Color::Yellow)),
        Span::styled("Crab Catch", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [3] ", Style::default().fg(Color::Yellow)),
        Span::styled("Snake", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [4] ", Style::default().fg(Color::Yellow)),
        Span::styled("Breakout", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [5] ", Style::default().fg(Color::Yellow)),
        Span::styled("Tetris", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [6] ", Style::default().fg(Color::Yellow)),
        Span::styled("Dash", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [7] ", Style::default().fg(Color::Yellow)),
        Span::styled("2048", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [8] ", Style::default().fg(Color::Yellow)),
        Span::styled("VSRG", Style::default().fg(Color::White)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  Press [1]..[8] to start",
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "  Press [space] or [q] to close",
        Style::default().fg(Color::DarkGray),
    )]));

    let overlay_height = (lines.len() as u16 + 2).min(area.height.saturating_sub(4));
    let overlay_width = 46.min(area.width.saturating_sub(4));
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

/// Render the VSRG mini-game
pub fn render_vsrg_game(frame: &mut Frame, game: &VsrgGame, area: Rect) {
    if area.width < 24 || area.height < 10 {
        return;
    }

    let lanes = vsrg_lane_count() as u16;
    let lane_width: u16 = 4;
    let separator_width: u16 = 1;
    let inner_width = lanes * lane_width + (lanes - 1) * separator_width;
    let inner_height = area.height.saturating_sub(2).max(6);
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
                x: play_area.x + play_width.saturating_sub(1),
                y: play_area.y + dy,
                width: 1,
                height: 1,
            },
        );
    }

    let separator_style = Style::default().fg(Color::Yellow);
    for lane in 1..lanes {
        let sep_x = play_area.x + 1 + lane * lane_width + (lane - 1) * separator_width;
        for dy in 1..play_height.saturating_sub(1) {
            if dy % 2 == 0 {
                frame.render_widget(
                    Paragraph::new(":").style(separator_style),
                    Rect {
                        x: sep_x,
                        y: play_area.y + dy,
                        width: 1,
                        height: 1,
                    },
                );
            }
        }
    }

    let hit_line_y = play_area.y + play_height.saturating_sub(2);
    let hit_line = "=".repeat(inner_width as usize);
    frame.render_widget(
        Paragraph::new(hit_line).style(Style::default().fg(Color::Yellow)),
        Rect {
            x: play_area.x + 1,
            y: hit_line_y,
            width: inner_width,
            height: 1,
        },
    );

    let hit_zone_y = hit_line_y.saturating_sub(1);
    for lane in 0..lanes {
        let lane_x = play_area.x + 1 + lane * (lane_width + separator_width);
        let flash_color = game.lane_flashes[lane as usize].map(|flash| match flash.kind {
            VsrgLaneFlashKind::Hit => Color::Green,
            VsrgLaneFlashKind::Miss => Color::Red,
        });
        let base_color = flash_color.unwrap_or(Color::DarkGray);
        render_block_cell(frame, lane_x, hit_zone_y, base_color, Modifier::BOLD);
        render_block_cell(frame, lane_x + 2, hit_zone_y, base_color, Modifier::BOLD);
    }

    for note in &game.notes {
        let lane = note.lane as u16;
        let lane_x = play_area.x + 1 + lane * (lane_width + separator_width);
        let note_x = lane_x + (lane_width.saturating_sub(2) / 2);
        let note_y = play_area.y + 1 + note.y.round().max(0.0) as u16;
        for dy in 0..note.length {
            let draw_y = note_y + dy;
            if draw_y >= play_area.y + play_height.saturating_sub(1) {
                continue;
            }
            render_block_cell(frame, note_x, draw_y, Color::Cyan, Modifier::BOLD);
        }
    }

    if let Some(feedback) = game.last_judgment {
        let (label, color) = match feedback.judgment {
            VsrgJudgment::Perfect => ("PERFECT", Color::Green),
            VsrgJudgment::Great => ("GREAT", Color::Cyan),
            VsrgJudgment::Ok => ("OK", Color::Yellow),
            VsrgJudgment::Miss => ("MISS", Color::Red),
        };
        let feedback_area = Rect {
            x: play_area.x + 2,
            y: hit_line_y.saturating_sub(2),
            width: inner_width.saturating_sub(2),
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(label)
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center),
            feedback_area,
        );
    }

    let hud_lines = vec![
        Line::from(vec![Span::styled(
            "  VSRG",
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
            Span::styled("  Acc: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}%", game.accuracy()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Combo: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.combo.to_string(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Time: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}s", game.remaining_time().ceil()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let hud_width = 22.min(area.width.saturating_sub(2));
    let hud_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: hud_width.max(1),
        height: 7.min(area.height.saturating_sub(2)),
    };

    let hud_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let hud_widget = Paragraph::new(hud_lines).block(hud_block);
    frame.render_widget(hud_widget, hud_area);
}

/// Render the 2048 mini-game
pub fn render_2048_game(frame: &mut Frame, game: &Game2048, area: Rect) {
    let tile_width: u16 = 7;
    let tile_height: u16 = 3;
    let grid_width = tile_width * 4;
    let grid_height = tile_height * 4;
    let play_width = grid_width + 2;
    let play_height = grid_height + 2;

    if area.width < play_width || area.height < play_height {
        return;
    }

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
                x: play_area.x + play_width.saturating_sub(1),
                y: play_area.y + dy,
                width: 1,
                height: 1,
            },
        );
    }

    for row in 0..4 {
        for col in 0..4 {
            let value = game.board[row][col];
            let (fg, bg) = tile_colors(value);
            let label = tile_label(value);
            let modifier = if value >= 128 {
                Modifier::BOLD
            } else {
                Modifier::empty()
            };
            let tile_style = if value == 0 {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().bg(bg).fg(fg).add_modifier(modifier)
            };

            let tile_x = play_area.x + 1 + (col as u16 * tile_width);
            let tile_y = play_area.y + 1 + (row as u16 * tile_height);

            for dy in 0..tile_height {
                let text = if dy == 1 {
                    label.clone()
                } else {
                    "       ".to_string()
                };
                frame.render_widget(
                    Paragraph::new(text).style(tile_style),
                    Rect {
                        x: tile_x,
                        y: tile_y + dy,
                        width: tile_width,
                        height: 1,
                    },
                );
            }
        }
    }

    let hud_lines = vec![
        Line::from(vec![Span::styled(
            "  2048",
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
            Span::styled("  Max: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                game.max_tile().to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let hud_width = 22.min(area.width.saturating_sub(2));
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

/// Render the VSRG results screen
pub fn render_vsrg_results(
    frame: &mut Frame,
    area: Rect,
    score: u32,
    accuracy: f32,
    max_combo: u32,
    app_state: &AppState,
) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  VSRG",
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

    if app_state.vsrg_best_scores.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No scores yet",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for (index, best) in app_state.vsrg_best_scores.iter().take(3).enumerate() {
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

    let rank = calculate_rank(&app_state.vsrg_best_scores, score);
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

    lines.push(Line::from(vec![Span::styled(
        format!("  Accuracy: {:.1}%", accuracy),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::styled(
        format!("  Max Combo: {}", max_combo),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
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

/// Render the 2048 results screen
pub fn render_2048_results(
    frame: &mut Frame,
    area: Rect,
    score: u32,
    max_tile: u32,
    app_state: &AppState,
) {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(vec![Span::styled(
        "  2048",
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

    if app_state.game_2048_best_scores.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No scores yet",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for (index, best) in app_state.game_2048_best_scores.iter().take(3).enumerate() {
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

    let rank = calculate_rank(&app_state.game_2048_best_scores, score);
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

    lines.push(Line::from(vec![Span::styled(
        format!("  Max Tile: {}", max_tile),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
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
