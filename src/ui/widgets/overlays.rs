use crate::git::{format_time_ago, CommitInfo};
use crate::state::{get_today_by_project, get_week_summary, AppState};
use chrono::Datelike;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::helpers::{centered_rect, truncate_str};

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
        Span::styled("  [d][f][j][k] ", Style::default().fg(Color::Yellow)),
        Span::styled("hit notes (vsrg)", Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [r] ", Style::default().fg(Color::Yellow)),
        Span::styled("restart (2048)", Style::default().fg(Color::White)),
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
