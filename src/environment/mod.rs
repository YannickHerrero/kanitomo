//! Environment module - handles ground, background, and decorative elements

pub mod elements;

use chrono::{Local, Timelike};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Time of day for background rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Morning, // 6-11: Sun rising, warm colors
    Day,     // 12-17: Full sun, bright
    Evening, // 18-20: Sunset colors
    Night,   // 21-5: Moon and stars
}

impl TimeOfDay {
    /// Get current time of day based on system clock
    pub fn current() -> Self {
        let hour = Local::now().hour();
        match hour {
            6..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Day,
            18..=20 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }
}

/// Ground style themes that rotate weekly
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum GroundStyle {
    #[default]
    Beach, // Sand, shells, waves
    Garden,  // Grass, flowers
    Rocky,   // Pebbles, stones
    Minimal, // Simple line
}

impl GroundStyle {
    /// Get a random ground style
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => GroundStyle::Beach,
            1 => GroundStyle::Garden,
            2 => GroundStyle::Rocky,
            _ => GroundStyle::Minimal,
        }
    }

    /// Get the ground characters for this style
    pub fn ground_chars(&self) -> &'static [char] {
        match self {
            GroundStyle::Beach => elements::BEACH_CHARS,
            GroundStyle::Garden => elements::GARDEN_CHARS,
            GroundStyle::Rocky => elements::ROCKY_CHARS,
            GroundStyle::Minimal => &[elements::MINIMAL_CHAR],
        }
    }
}

/// A positioned element in the environment
#[derive(Debug, Clone)]
pub struct PositionedElement {
    pub x: u16,
    pub y: u16,
    pub content: Vec<String>,
}

/// A star in the night sky
#[derive(Debug, Clone)]
pub struct Star {
    pub x: u16,
    pub y: u16,
    pub char: char,
    #[allow(dead_code)] // Reserved for future twinkling animation
    pub twinkle_offset: f32,
}

/// The complete environment state
#[derive(Debug, Clone)]
pub struct Environment {
    /// Current ground style
    pub ground_style: GroundStyle,
    /// Generated ground line (cached)
    pub ground_line: String,
    /// Background objects (clouds, sun/moon)
    pub background_elements: Vec<PositionedElement>,
    /// Stars for nighttime
    pub stars: Vec<Star>,
    /// Width of the environment
    pub width: u16,
    /// Height of the environment (crab area)
    pub height: u16,
    /// Current time of day
    pub time_of_day: TimeOfDay,
}

impl Environment {
    /// Generate a new environment for the given dimensions
    pub fn generate(width: u16, height: u16, style: GroundStyle) -> Self {
        let mut rng = rand::thread_rng();
        let time_of_day = TimeOfDay::current();

        // Generate ground line
        let ground_line = Self::generate_ground_line(width, style, &mut rng);

        // Generate background elements
        let background_elements = Self::generate_background(width, height, time_of_day, &mut rng);

        // Generate stars for nighttime
        let stars = if time_of_day == TimeOfDay::Night {
            Self::generate_stars(width, height, &mut rng)
        } else {
            Vec::new()
        };

        Self {
            ground_style: style,
            ground_line,
            background_elements,
            stars,
            width,
            height,
            time_of_day,
        }
    }

    /// Generate the ground decoration line
    fn generate_ground_line(width: u16, style: GroundStyle, rng: &mut impl Rng) -> String {
        let chars = style.ground_chars();
        (0..width)
            .map(|_| *chars.choose(rng).unwrap_or(&'.'))
            .collect()
    }

    /// Generate background elements (sun/moon, clouds)
    fn generate_background(
        width: u16,
        height: u16,
        time: TimeOfDay,
        rng: &mut impl Rng,
    ) -> Vec<PositionedElement> {
        let mut elements = Vec::new();

        // Only add background elements if we have enough space
        if height < 6 || width < 20 {
            return elements;
        }

        // Add sun or moon based on time of day
        match time {
            TimeOfDay::Morning | TimeOfDay::Day | TimeOfDay::Evening => {
                // Add sun in the upper portion
                if width >= 15 {
                    let sun_x = rng.gen_range(width / 3..width * 2 / 3);
                    let sun_y = rng.gen_range(0..2.min(height / 4));
                    elements.push(PositionedElement {
                        x: sun_x,
                        y: sun_y,
                        content: elements::SUN.iter().map(|s| s.to_string()).collect(),
                    });
                }
            }
            TimeOfDay::Night => {
                // Add moon
                if width >= 12 && height >= 5 {
                    let moon_x = rng.gen_range(width / 4..width / 2);
                    elements.push(PositionedElement {
                        x: moon_x,
                        y: 0,
                        content: elements::MOON_SMALL.iter().map(|s| s.to_string()).collect(),
                    });
                }
            }
        }

        // Add 0-2 clouds (more during day, fewer at night)
        let cloud_count = match time {
            TimeOfDay::Day => rng.gen_range(0..=2),
            TimeOfDay::Morning | TimeOfDay::Evening => rng.gen_range(0..=1),
            TimeOfDay::Night => 0, // No clouds at night (stars instead)
        };

        for _ in 0..cloud_count {
            if width >= 15 && height >= 5 {
                let cloud = if rng.gen_bool(0.5) {
                    elements::CLOUD_SMALL
                } else {
                    elements::CLOUD_LARGE
                };

                let cloud_width = cloud[0].len() as u16;
                let cloud_x = rng.gen_range(0..width.saturating_sub(cloud_width));
                let cloud_y = rng.gen_range(0..height / 3);

                elements.push(PositionedElement {
                    x: cloud_x,
                    y: cloud_y,
                    content: cloud.iter().map(|s| s.to_string()).collect(),
                });
            }
        }

        elements
    }

    /// Generate stars for nighttime
    fn generate_stars(width: u16, height: u16, rng: &mut impl Rng) -> Vec<Star> {
        let mut stars = Vec::new();

        // Add stars based on available space
        let star_count = (width as usize * height as usize) / 40; // ~1 star per 40 cells
        let star_count = star_count.min(30); // Cap at 30 stars

        for _ in 0..star_count {
            // Keep stars in upper 2/3 of the area (above where Kani walks)
            let max_y = (height * 2 / 3).max(1);
            stars.push(Star {
                x: rng.gen_range(0..width),
                y: rng.gen_range(0..max_y),
                char: *elements::STAR_CHARS.choose(rng).unwrap_or(&'*'),
                twinkle_offset: rng.gen_range(0.0..std::f32::consts::TAU),
            });
        }

        stars
    }

    /// Update time of day (call periodically to refresh)
    pub fn update_time(&mut self) {
        let new_time = TimeOfDay::current();
        if new_time != self.time_of_day {
            // Regenerate background for new time of day
            let mut rng = rand::thread_rng();
            self.time_of_day = new_time;
            self.background_elements =
                Self::generate_background(self.width, self.height, new_time, &mut rng);
            self.stars = if new_time == TimeOfDay::Night {
                Self::generate_stars(self.width, self.height, &mut rng)
            } else {
                Vec::new()
            };
        }
    }
}
