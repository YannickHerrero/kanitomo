use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub(crate) fn render_element(
    frame: &mut Frame,
    content: &[String],
    x: i32,
    y: i32,
    color: Color,
    area: Rect,
) {
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

pub(crate) fn render_block_cell(
    frame: &mut Frame,
    x: u16,
    y: u16,
    color: Color,
    modifier: Modifier,
) {
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

pub(crate) fn piece_color(piece_type: crate::ui::minigames::PieceType) -> Color {
    use crate::ui::minigames::PieceType;

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

pub(crate) fn tile_label(value: u32) -> String {
    if value == 0 {
        return "   ·   ".to_string();
    }

    let label = value.to_string();
    let total = 7usize;
    let padding = total.saturating_sub(label.len());
    let left = padding / 2;
    let right = padding.saturating_sub(left);
    format!("{}{}{}", " ".repeat(left), label, " ".repeat(right))
}

pub(crate) fn tile_colors(value: u32) -> (Color, Color) {
    match value {
        0 => (Color::DarkGray, Color::Reset),
        2 => (Color::Rgb(119, 110, 101), Color::Rgb(238, 228, 218)),
        4 => (Color::Rgb(119, 110, 101), Color::Rgb(237, 224, 200)),
        8 => (Color::Rgb(249, 246, 242), Color::Rgb(242, 177, 121)),
        16 => (Color::Rgb(249, 246, 242), Color::Rgb(245, 149, 99)),
        32 => (Color::Rgb(249, 246, 242), Color::Rgb(246, 124, 95)),
        64 => (Color::Rgb(249, 246, 242), Color::Rgb(246, 94, 59)),
        128 => (Color::Rgb(249, 246, 242), Color::Rgb(237, 207, 114)),
        256 => (Color::Rgb(249, 246, 242), Color::Rgb(237, 204, 97)),
        512 => (Color::Rgb(249, 246, 242), Color::Rgb(237, 200, 80)),
        1024 => (Color::Rgb(249, 246, 242), Color::Rgb(237, 197, 63)),
        2048 => (Color::Rgb(249, 246, 242), Color::Rgb(237, 194, 46)),
        _ => (Color::Rgb(249, 246, 242), Color::Rgb(60, 58, 50)),
    }
}

pub(crate) fn calculate_rank(scores: &[u32], current_score: u32) -> usize {
    scores
        .iter()
        .position(|&s| s <= current_score)
        .map(|p| p + 1)
        .unwrap_or(scores.len() + 1)
}

pub(crate) fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}..", &s[..max_len - 2])
    }
}

pub(crate) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
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

pub(crate) fn render_happiness_bar(happiness: u8) -> Line<'static> {
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
