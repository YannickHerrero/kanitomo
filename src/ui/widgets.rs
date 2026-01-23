use crate::crab::Crab;
use crate::environment::{Environment, TimeOfDay};
use crate::git::{format_time_ago, GitStats};
use crate::state::{get_today_by_project, get_week_summary, AppState};
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
            crate::environment::GroundStyle::Beach => Color::Rgb(80, 70, 50),
            crate::environment::GroundStyle::Garden => Color::Rgb(30, 60, 30),
            crate::environment::GroundStyle::Rocky => Color::Rgb(60, 60, 60),
            crate::environment::GroundStyle::Minimal => Color::DarkGray,
        },
        _ => match env.ground_style {
            crate::environment::GroundStyle::Beach => Color::Rgb(194, 178, 128), // Sandy
            crate::environment::GroundStyle::Garden => Color::Rgb(34, 139, 34),  // Forest green
            crate::environment::GroundStyle::Rocky => Color::Rgb(128, 128, 128), // Gray
            crate::environment::GroundStyle::Minimal => Color::DarkGray,
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

/// Render the help bar at the bottom
pub fn render_help(frame: &mut Frame, area: Rect, debug_mode: bool, multi_repo: bool) {
    let mut spans = vec![
        Span::styled(" [q] ", Style::default().fg(Color::Yellow)),
        Span::styled("quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[r] ", Style::default().fg(Color::Yellow)),
        Span::styled("refresh  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[d] ", Style::default().fg(Color::Yellow)),
        Span::styled("details  ", Style::default().fg(Color::DarkGray)),
    ];

    if multi_repo {
        spans.extend([
            Span::styled("[a] ", Style::default().fg(Color::Yellow)),
            Span::styled("repos  ", Style::default().fg(Color::DarkGray)),
        ]);
    }

    if debug_mode {
        spans.extend([
            Span::styled("[f] ", Style::default().fg(Color::Yellow)),
            Span::styled("feed  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[p] ", Style::default().fg(Color::Yellow)),
            Span::styled("punish  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[s] ", Style::default().fg(Color::Yellow)),
            Span::styled("freeze  ", Style::default().fg(Color::DarkGray)),
        ]);
    }

    let help_text = Line::from(spans);
    let paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
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
