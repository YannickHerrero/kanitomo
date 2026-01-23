//! Environment module - handles ground, background, and decorative elements

pub mod elements;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Time of day for background rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Morning, // 6-11: Sun rising, warm colors
    Day,     // 12-17: Full sun, bright
    Evening, // 18-20: Sunset colors
    Night,   // 21-5: Moon and stars
}

impl TimeOfDay {
    pub fn from_phase(phase: f32) -> Self {
        if phase < 0.2 {
            TimeOfDay::Morning
        } else if phase < 0.45 {
            TimeOfDay::Day
        } else if phase < 0.5 {
            TimeOfDay::Evening
        } else {
            TimeOfDay::Night
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

/// A star in the night sky
#[derive(Debug, Clone)]
pub struct Star {
    pub x: u16,
    pub y: u16,
    pub char: char,
    #[allow(dead_code)] // Reserved for future twinkling animation
    pub twinkle_offset: f32,
}

/// A moving cloud
#[derive(Debug, Clone)]
pub struct Cloud {
    pub x: f32,
    pub y: u16,
    pub speed: f32,
    pub content: Vec<String>,
    pub width: u16,
    pub night_visible: bool,
}

/// The complete environment state
#[derive(Debug, Clone)]
pub struct Environment {
    /// Current ground style
    pub ground_style: GroundStyle,
    /// Generated ground line (cached)
    pub ground_line: String,
    /// Moving clouds
    pub clouds: Vec<Cloud>,
    /// Stars for nighttime
    pub stars: Vec<Star>,
    /// Width of the environment
    pub width: u16,
    /// Height of the environment (crab area)
    pub height: u16,
    /// Current time of day
    pub time_of_day: TimeOfDay,
    /// Day/night cycle phase (0.0 to 1.0)
    pub cycle_phase: f32,
    /// Total cycle duration
    pub cycle_duration: Duration,
}

impl Environment {
    /// Generate a new environment for the given dimensions
    pub fn generate(width: u16, height: u16, style: GroundStyle) -> Self {
        let mut rng = rand::thread_rng();
        let cycle_duration = Duration::from_secs(18 * 60);
        let cycle_phase = 0.0; // Always start at sunrise
        let time_of_day = TimeOfDay::from_phase(cycle_phase);

        // Generate ground line
        let ground_line = Self::generate_ground_line(width, style, &mut rng);

        // Generate moving clouds
        let clouds = Self::generate_clouds(width, height, &mut rng);

        // Generate stars for nighttime
        let stars = if time_of_day == TimeOfDay::Night {
            Self::generate_stars(width, height, &mut rng)
        } else {
            Vec::new()
        };

        Self {
            ground_style: style,
            ground_line,
            clouds,
            stars,
            width,
            height,
            time_of_day,
            cycle_phase,
            cycle_duration,
        }
    }

    /// Generate the ground decoration line
    fn generate_ground_line(width: u16, style: GroundStyle, rng: &mut impl Rng) -> String {
        let chars = style.ground_chars();
        (0..width)
            .map(|_| *chars.choose(rng).unwrap_or(&'.'))
            .collect()
    }

    /// Generate moving clouds
    fn generate_clouds(width: u16, height: u16, rng: &mut impl Rng) -> Vec<Cloud> {
        let mut clouds = Vec::new();

        if height < 6 || width < 20 {
            return clouds;
        }

        let cloud_count = rng.gen_range(2..=4);

        for _ in 0..cloud_count {
            let cloud = if rng.gen_bool(0.5) {
                elements::CLOUD_SMALL
            } else {
                elements::CLOUD_LARGE
            };

            let cloud_width = cloud[0].len() as u16;
            let spawn_left = -(cloud_width as f32 * rng.gen_range(1.0..2.5));
            let spawn_right = width as f32 + cloud_width as f32;
            let cloud_x = rng.gen_range(spawn_left..spawn_right);
            let cloud_y = rng.gen_range(0..height / 3);
            let speed = rng.gen_range(0.25..0.7); // chars/sec, slow
            let night_visible = rng.gen_bool(0.6);

            clouds.push(Cloud {
                x: cloud_x,
                y: cloud_y,
                speed,
                content: cloud.iter().map(|s| s.to_string()).collect(),
                width: cloud_width,
                night_visible,
            });
        }

        clouds
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

    /// Update day/night cycle and move clouds
    pub fn update_cycle(&mut self, dt: f32, cycle_speed: f32, cloud_speed: f32) {
        let cycle_seconds = self.cycle_duration.as_secs_f32().max(1.0);
        let cycle_dt = dt * cycle_speed;
        let cloud_dt = dt * cloud_speed;
        self.cycle_phase = (self.cycle_phase + (cycle_dt / cycle_seconds)) % 1.0;

        let new_time = TimeOfDay::from_phase(self.cycle_phase);
        if new_time != self.time_of_day {
            let mut rng = rand::thread_rng();
            self.time_of_day = new_time;
            self.stars = if new_time == TimeOfDay::Night {
                Self::generate_stars(self.width, self.height, &mut rng)
            } else {
                Vec::new()
            };
        }

        for cloud in &mut self.clouds {
            cloud.x += cloud.speed * cloud_dt;
            if cloud.x > self.width as f32 + cloud.width as f32 {
                cloud.x = -(cloud.width as f32);
            }
        }
    }

    pub fn sun_position(&self) -> Option<(i32, i32)> {
        if self.cycle_phase >= 0.5 {
            return None;
        }
        Some(self.arc_position(self.cycle_phase * 2.0))
    }

    pub fn moon_position(&self) -> Option<(i32, i32)> {
        if self.cycle_phase < 0.5 {
            return None;
        }
        Some(self.arc_position((self.cycle_phase - 0.5) * 2.0))
    }

    fn arc_position(&self, t: f32) -> (i32, i32) {
        let width = self.width.max(1) as f32;
        let height = self.height.max(1) as f32;
        let left_x = -(width * 0.1).max(3.0);
        let right_x = width * 1.1;
        let base_y = height * 0.25;
        let apex_y = (height * 0.05).max(0.0);
        let arc_height = (base_y - apex_y).max(1.0);

        let x = left_x + (right_x - left_x) * t;
        let y = base_y - arc_height * (std::f32::consts::PI * t).sin();

        (x.round() as i32, y.round() as i32)
    }
}
