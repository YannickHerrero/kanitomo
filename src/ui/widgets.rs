use crate::crab::Crab;
use crate::git::GitStats;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
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

/// Render the stats panel
pub fn render_stats(frame: &mut Frame, stats: &GitStats, happiness: u8, area: Rect) {
    let mood = crate::crab::Mood::from_happiness(happiness);

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
        if let Some(ref name) = stats.repo_name {
            lines.push(Line::from(vec![
                Span::styled("  Repo: ", Style::default().fg(Color::DarkGray)),
                Span::styled(name.clone(), Style::default().fg(Color::Cyan)),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("  Commits today: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                stats.commits_today.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Streak: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{} day{}",
                    stats.current_streak,
                    if stats.current_streak == 1 { "" } else { "s" }
                ),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        if stats.best_streak > 0 {
            lines.push(Line::from(vec![
                Span::styled("  Best streak: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} days", stats.best_streak),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }

        lines.push(Line::from(vec![
            Span::styled("  Last commit: ", Style::default().fg(Color::DarkGray)),
            Span::styled(stats.last_commit_ago(), Style::default().fg(Color::White)),
        ]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            "  No git repository",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  Run in a git folder",
            Style::default().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  to track commits!",
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
pub fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = Line::from(vec![
        Span::styled(" [q] ", Style::default().fg(Color::Yellow)),
        Span::styled("quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[r] ", Style::default().fg(Color::Yellow)),
        Span::styled("refresh  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[f] ", Style::default().fg(Color::Yellow)),
        Span::styled("feed  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[p] ", Style::default().fg(Color::Yellow)),
        Span::styled("punish  ", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Render the title bar
pub fn render_title(frame: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(" 🦀 ", Style::default().fg(Color::Red)),
        Span::styled(
            "Kanitomo",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" 🦀 ", Style::default().fg(Color::Red)),
    ]);

    let paragraph = Paragraph::new(title).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
