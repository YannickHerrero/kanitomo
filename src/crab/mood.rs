use serde::{Deserialize, Serialize};

/// The mood states for the crab, from best to worst
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    /// Just committed! Crab is doing a happy dance
    Ecstatic,
    /// Good streak, commits today - normal happy walking
    Happy,
    /// Some activity recently - slower movement
    Neutral,
    /// No commits today - droopy, minimal movement
    Sad,
    /// Long idle period - dramatic sad pose, begging
    Hungry,
}

impl Mood {
    /// Calculate mood from happiness level (0-100)
    pub fn from_happiness(happiness: u8) -> Self {
        match happiness {
            90..=100 => Mood::Ecstatic,
            70..=89 => Mood::Happy,
            40..=69 => Mood::Neutral,
            20..=39 => Mood::Sad,
            _ => Mood::Hungry,
        }
    }

    /// Get a display name for the mood
    pub fn display_name(&self) -> &'static str {
        match self {
            Mood::Ecstatic => "Ecstatic",
            Mood::Happy => "Happy",
            Mood::Neutral => "Neutral",
            Mood::Sad => "Sad",
            Mood::Hungry => "Hungry",
        }
    }

    /// Get a color for the mood (for UI)
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            Mood::Ecstatic => Color::Magenta,
            Mood::Happy => Color::Green,
            Mood::Neutral => Color::Yellow,
            Mood::Sad => Color::Blue,
            Mood::Hungry => Color::Red,
        }
    }

    /// Get animation speed multiplier
    pub fn animation_speed(&self) -> f32 {
        match self {
            Mood::Ecstatic => 2.0, // Fast happy dance
            Mood::Happy => 1.0,    // Normal speed
            Mood::Neutral => 0.6,  // Slower
            Mood::Sad => 0.3,      // Very slow
            Mood::Hungry => 0.2,   // Barely moving
        }
    }
}

impl std::fmt::Display for Mood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
