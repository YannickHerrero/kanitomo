use super::mood::Mood;
use rand::Rng;

/// Direction the crab is facing
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

/// Animation frames for different moods and directions
pub struct CrabFrames;

impl CrabFrames {
    // Standard pose facing right
    pub const STANDARD_RIGHT: &'static str = r#"    _~^~^~_
\) /  o o  \ (/
  '_   -   _'
  \ '-----' /"#;

    // Walking pose facing right
    pub const WALKING_RIGHT: &'static str = r#"    _~^~^~_
\) /  o o  \ (/
 '-_   -   _'\
  | '-----' "#;

    // Happy/clapping pose facing right
    pub const HAPPY_RIGHT: &'static str = r#"    _~^~^~_
\/ /  ^o o^  \ \/
  '_   u   _'
  \ '-----' /"#;

    // Sad pose facing right
    pub const SAD_RIGHT: &'static str = r#"    _~^~^~_
\) /  - -  \ (/
  '_   n   _'
  \ '-----' /"#;

    // Hungry/begging pose facing right
    pub const HUNGRY_RIGHT: &'static str = r#"    _~^~^~_
\\ /  T T  \ //
  '_   ~   _'
  \ '-----' /"#;

    // Standard pose facing left
    pub const STANDARD_LEFT: &'static str = r#"    _~^~^~_
(\ /  o o  \ ()
  '_   -   _'
  / '-----' \"#;

    // Walking pose facing left
    pub const WALKING_LEFT: &'static str = r#"    _~^~^~_
(\ /  o o  \ ()
 /'_   -   _-'
    '-----' |"#;

    // Happy/clapping pose facing left
    pub const HAPPY_LEFT: &'static str = r#"    _~^~^~_
\/ /  ^o o^  \ \/
  '_   u   _'
  / '-----' \"#;

    // Sad pose facing left
    pub const SAD_LEFT: &'static str = r#"    _~^~^~_
(\ /  - -  \ ()
  '_   n   _'
  / '-----' \"#;

    // Hungry/begging pose facing left
    pub const HUNGRY_LEFT: &'static str = r#"    _~^~^~_
// /  T T  \ \\
  '_   ~   _'
  / '-----' \"#;

    // Ecstatic dance frame 1
    pub const ECSTATIC_1: &'static str = r#"   \\_~^~^~_//
   /  *o o*  \
  '_    w    _'
  \\ '-----' //"#;

    // Ecstatic dance frame 2
    pub const ECSTATIC_2: &'static str = r#"  //_~^~^~_\\
   /  *o o*  \
  '_    w    _'
  // '-----' \\"#;
}

/// The main crab entity with position, animation state, and mood
pub struct Crab {
    /// Position (x, y) as floats for smooth movement
    pub position: (f32, f32),
    /// Velocity (vx, vy)
    pub velocity: (f32, f32),
    /// Current facing direction
    pub direction: Direction,
    /// Current mood
    pub mood: Mood,
    /// Happiness level (0-100)
    pub happiness: u8,
    /// Animation frame index
    frame_index: usize,
    /// Animation timer
    animation_timer: f32,
    /// Whether the crab just received a boost (e.g., new commit)
    pub celebrating: bool,
    /// Timer for celebration animation
    celebration_timer: f32,
    /// Random number generator
    rng: rand::rngs::ThreadRng,
}

impl Crab {
    /// Create a new crab at the given position with initial happiness
    pub fn new(position: (f32, f32), happiness: u8) -> Self {
        let mut rng = rand::thread_rng();
        let direction = if rng.gen_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };

        Self {
            position,
            velocity: (0.0, 0.0),
            direction,
            mood: Mood::from_happiness(happiness),
            happiness,
            frame_index: 0,
            animation_timer: 0.0,
            celebrating: false,
            celebration_timer: 0.0,
            rng,
        }
    }

    /// Update the crab's animation and position
    pub fn update(&mut self, dt: f32, bounds: (f32, f32)) {
        // Update mood from happiness
        self.mood = Mood::from_happiness(self.happiness);

        // Handle celebration
        if self.celebrating {
            self.celebration_timer -= dt;
            if self.celebration_timer <= 0.0 {
                self.celebrating = false;
            }
        }

        // Animation speed based on mood
        let speed_mult = if self.celebrating {
            2.5
        } else {
            self.mood.animation_speed()
        };

        // Update animation timer
        self.animation_timer += dt * speed_mult;
        if self.animation_timer >= 0.3 {
            self.animation_timer = 0.0;
            self.frame_index = (self.frame_index + 1) % 4;
        }

        // Movement based on mood
        let move_chance = match self.mood {
            Mood::Ecstatic => 0.05,
            Mood::Happy => 0.03,
            Mood::Neutral => 0.02,
            Mood::Sad => 0.01,
            Mood::Hungry => 0.005,
        };

        // Randomly decide to move
        if self.rng.gen::<f32>() < move_chance {
            let base_speed = match self.mood {
                Mood::Ecstatic => 1.5,
                Mood::Happy => 1.0,
                Mood::Neutral => 0.5,
                Mood::Sad => 0.3,
                Mood::Hungry => 0.1,
            };

            self.velocity.0 = self.rng.gen_range(-base_speed..base_speed);
            self.velocity.1 = self.rng.gen_range(-base_speed * 0.3..base_speed * 0.3);

            // Update direction based on velocity
            if self.velocity.0 > 0.1 {
                self.direction = Direction::Right;
            } else if self.velocity.0 < -0.1 {
                self.direction = Direction::Left;
            }
        }

        // Apply friction
        self.velocity.0 *= 0.95;
        self.velocity.1 *= 0.95;

        // Update position
        self.position.0 += self.velocity.0;
        self.position.1 += self.velocity.1;

        // Get frame dimensions
        let frame_width = 20.0;
        let frame_height = 4.0;

        // Boundary collision
        if self.position.0 < 0.0 {
            self.position.0 = 0.0;
            self.velocity.0 = self.velocity.0.abs();
            self.direction = Direction::Right;
        } else if self.position.0 + frame_width > bounds.0 {
            self.position.0 = bounds.0 - frame_width;
            self.velocity.0 = -self.velocity.0.abs();
            self.direction = Direction::Left;
        }

        if self.position.1 < 0.0 {
            self.position.1 = 0.0;
            self.velocity.1 = self.velocity.1.abs();
        } else if self.position.1 + frame_height > bounds.1 {
            self.position.1 = bounds.1 - frame_height;
            self.velocity.1 = -self.velocity.1.abs();
        }
    }

    /// Get the current animation frame as a string
    pub fn get_frame(&self) -> &'static str {
        // If celebrating, use ecstatic frames
        if self.celebrating || self.mood == Mood::Ecstatic {
            return if self.frame_index % 2 == 0 {
                CrabFrames::ECSTATIC_1
            } else {
                CrabFrames::ECSTATIC_2
            };
        }

        let is_moving = self.velocity.0.abs() > 0.05 || self.velocity.1.abs() > 0.05;

        match self.mood {
            Mood::Ecstatic => {
                if self.frame_index % 2 == 0 {
                    CrabFrames::ECSTATIC_1
                } else {
                    CrabFrames::ECSTATIC_2
                }
            }
            Mood::Happy => {
                if is_moving {
                    if self.direction == Direction::Right {
                        if self.frame_index % 2 == 0 {
                            CrabFrames::STANDARD_RIGHT
                        } else {
                            CrabFrames::WALKING_RIGHT
                        }
                    } else if self.frame_index % 2 == 0 {
                        CrabFrames::STANDARD_LEFT
                    } else {
                        CrabFrames::WALKING_LEFT
                    }
                } else if self.frame_index % 4 == 0 {
                    // Occasional happy clap when idle
                    if self.direction == Direction::Right {
                        CrabFrames::HAPPY_RIGHT
                    } else {
                        CrabFrames::HAPPY_LEFT
                    }
                } else if self.direction == Direction::Right {
                    CrabFrames::STANDARD_RIGHT
                } else {
                    CrabFrames::STANDARD_LEFT
                }
            }
            Mood::Neutral => {
                if is_moving {
                    if self.direction == Direction::Right {
                        if self.frame_index % 2 == 0 {
                            CrabFrames::STANDARD_RIGHT
                        } else {
                            CrabFrames::WALKING_RIGHT
                        }
                    } else if self.frame_index % 2 == 0 {
                        CrabFrames::STANDARD_LEFT
                    } else {
                        CrabFrames::WALKING_LEFT
                    }
                } else if self.direction == Direction::Right {
                    CrabFrames::STANDARD_RIGHT
                } else {
                    CrabFrames::STANDARD_LEFT
                }
            }
            Mood::Sad => {
                if self.direction == Direction::Right {
                    CrabFrames::SAD_RIGHT
                } else {
                    CrabFrames::SAD_LEFT
                }
            }
            Mood::Hungry => {
                // Alternate between sad and hungry for begging effect
                if self.frame_index % 2 == 0 {
                    if self.direction == Direction::Right {
                        CrabFrames::HUNGRY_RIGHT
                    } else {
                        CrabFrames::HUNGRY_LEFT
                    }
                } else if self.direction == Direction::Right {
                    CrabFrames::SAD_RIGHT
                } else {
                    CrabFrames::SAD_LEFT
                }
            }
        }
    }

    /// Trigger celebration (e.g., when a new commit is detected)
    pub fn celebrate(&mut self) {
        self.celebrating = true;
        self.celebration_timer = 3.0; // 3 seconds of celebration
    }

    /// Boost happiness (e.g., from a commit)
    pub fn boost_happiness(&mut self, amount: u8) {
        self.happiness = self.happiness.saturating_add(amount).min(100);
        self.mood = Mood::from_happiness(self.happiness);
    }

    /// Decay happiness over time
    pub fn decay_happiness(&mut self, amount: u8) {
        self.happiness = self.happiness.saturating_sub(amount);
        self.mood = Mood::from_happiness(self.happiness);
    }

    /// Get the crab's color based on mood
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        if self.celebrating {
            Color::LightMagenta
        } else {
            // Reddish-orange crab color, slightly adjusted by mood
            match self.mood {
                Mood::Ecstatic => Color::Rgb(255, 100, 100),
                Mood::Happy => Color::Rgb(255, 120, 80),
                Mood::Neutral => Color::Rgb(220, 100, 80),
                Mood::Sad => Color::Rgb(180, 80, 80),
                Mood::Hungry => Color::Rgb(150, 60, 60),
            }
        }
    }
}
