use super::mood::Mood;
use rand::Rng;

/// Physics constants
const GRAVITY: f32 = 0.1;
const GROUND_FRICTION: f32 = 0.92;
const AIR_FRICTION: f32 = 0.98;

/// Jump strengths for different triggers
const JUMP_STRENGTH_CELEBRATION: f32 = 2.2;
const JUMP_STRENGTH_ECSTATIC: f32 = 1.8;
const JUMP_STRENGTH_HAPPY: f32 = 1.4;
const JUMP_STRENGTH_NEUTRAL: f32 = 1.0;

/// Direction the crab is facing
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

/// Eye expressions for different moods
pub struct Eyes;

impl Eyes {
    pub const NEUTRAL: &'static str = "o o";
    pub const HAPPY: &'static str = "^ ^";
    pub const SAD: &'static str = "- -";
    pub const HUNGRY: &'static str = "T T";
    pub const ECSTATIC: &'static str = "* *";
}

/// Mouth expressions for different moods
pub struct Mouths;

impl Mouths {
    pub const NEUTRAL: &'static str = "-";
    pub const HAPPY: &'static str = "u";
    pub const SAD: &'static str = "n";
    pub const HUNGRY: &'static str = "~";
    pub const ECSTATIC: &'static str = "w";
}

/// Body pose templates with {eyes} and {mouth} placeholders
pub struct BodyTemplates;

impl BodyTemplates {
    // Standing pose facing right
    pub const STANDING_RIGHT: &'static str = r#"    _~^~^~_
\) /  {eyes}  \ (/
  '_   {mouth}   _'
  \ '-----' /"#;

    // Standing pose facing left
    pub const STANDING_LEFT: &'static str = r#"    _~^~^~_
(\ /  {eyes}  \ ()
  '_   {mouth}   _'
  / '-----' \"#;

    // Walking pose facing right (legs offset)
    pub const WALKING_RIGHT: &'static str = r#"    _~^~^~_
\) /  {eyes}  \ (/
  '_   {mouth}   _'
  / '-----' \"#;

    // Walking pose facing left (legs offset)
    pub const WALKING_LEFT: &'static str = r#"    _~^~^~_
(\ /  {eyes}  \ ()
  '_   {mouth}   _'
  \ '-----' /"#;

    // Happy/clapping pose facing right (arms up)
    pub const CLAPPING_RIGHT: &'static str = r#"    _~^~^~_
\/ /  {eyes}  \ \/
  '_   {mouth}   _'
  \ '-----' /"#;

    // Happy/clapping pose facing left (arms up)
    pub const CLAPPING_LEFT: &'static str = r#"    _~^~^~_
|| /  {eyes}  \ ||
  '_   {mouth}   _'
  / '-----' \"#;

    // Hungry/begging pose facing right (arms raised)
    pub const BEGGING_RIGHT: &'static str = r#"    _~^~^~_
\\ /  {eyes}  \ //
  '_   {mouth}   _'
  \ '-----' /"#;

    // Hungry/begging pose facing left (arms raised)
    pub const BEGGING_LEFT: &'static str = r#"    _~^~^~_
// /  {eyes}  \ \\
  '_   {mouth}   _'
  / '-----' \"#;

    // Ecstatic dance frame 1
    pub const ECSTATIC_1: &'static str = r#"   ()_~^~^~_()
    /  {eyes}  \
   '_   {mouth}   _'
  \\ '-----' //"#;

    // Ecstatic dance frame 2
    pub const ECSTATIC_2: &'static str = r#"   \/_~^~^~_\/
    /  {eyes}  \
   '_   {mouth}   _'
  // '-----' \\"#;
}

/// Helper to build a frame from a body template and face components
pub fn build_frame(body: &str, eyes: &str, mouth: &str) -> String {
    body.replace("{eyes}", eyes).replace("{mouth}", mouth)
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
    /// Whether movement is frozen (animation still plays)
    pub movement_frozen: bool,
    /// Whether the crab is on the ground
    pub is_grounded: bool,
    /// The y-position of the ground (calculated from bounds)
    ground_y: f32,
    /// Cooldown timer for jumping (prevents spam)
    jump_cooldown: f32,
    /// Whether celebration jump has been triggered (once per celebration)
    celebration_jump_done: bool,
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
            movement_frozen: false,
            is_grounded: true,
            ground_y: position.1, // Will be set properly on first update
            jump_cooldown: 0.0,
            celebration_jump_done: false,
        }
    }

    /// Update the crab's animation and position
    pub fn update(&mut self, dt: f32, bounds: (f32, f32)) {
        // Update mood from happiness
        self.mood = Mood::from_happiness(self.happiness);

        // Calculate ground position (leave 1 line space at bottom for ground decoration)
        let frame_height = 4.0;
        let new_ground_y = bounds.1 - frame_height - 1.0;

        // If ground level changed significantly and crab was grounded, snap to new ground
        if self.is_grounded && (new_ground_y - self.ground_y).abs() > 0.5 {
            self.position.1 = new_ground_y;
        }

        self.ground_y = new_ground_y;

        // Handle celebration
        if self.celebrating {
            self.celebration_timer -= dt;
            if self.celebration_timer <= 0.0 {
                self.celebrating = false;
                self.celebration_jump_done = false; // Reset for next celebration
            }
        }

        // Animation speed based on mood
        let speed_mult = if self.celebrating || !self.is_grounded {
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

        // Update jump cooldown
        if self.jump_cooldown > 0.0 {
            self.jump_cooldown -= dt;
        }

        // Skip movement when frozen
        if self.movement_frozen {
            return;
        }

        // Trigger celebration jump (once per celebration)
        if self.celebrating && !self.celebration_jump_done && self.is_grounded {
            let strength = self.randomize_jump_strength(JUMP_STRENGTH_CELEBRATION);
            self.jump(strength);
            self.celebration_jump_done = true;
        }

        // Random jumps based on mood (only when grounded)
        if self.is_grounded && !self.celebrating {
            let jump_chance = match self.mood {
                Mood::Ecstatic => 0.015,         // ~1.5% per tick
                Mood::Happy => 0.004,            // ~0.4% per tick
                Mood::Neutral => 0.001,          // ~0.1% per tick
                Mood::Sad | Mood::Hungry => 0.0, // No jumping when sad/hungry
            };

            if self.rng.gen::<f32>() < jump_chance {
                let strength = match self.mood {
                    Mood::Ecstatic => JUMP_STRENGTH_ECSTATIC,
                    Mood::Happy => JUMP_STRENGTH_HAPPY,
                    Mood::Neutral => JUMP_STRENGTH_NEUTRAL,
                    _ => 0.0,
                };
                if strength > 0.0 {
                    let strength = self.randomize_jump_strength(strength);
                    self.jump(strength);
                }
            }
        }

        // Horizontal movement based on mood (walking)
        let move_chance = match self.mood {
            Mood::Ecstatic => 0.05,
            Mood::Happy => 0.03,
            Mood::Neutral => 0.02,
            Mood::Sad => 0.01,
            Mood::Hungry => 0.005,
        };

        // Randomly decide to walk
        if self.is_grounded && self.rng.gen::<f32>() < move_chance {
            let base_speed = match self.mood {
                Mood::Ecstatic => 1.5,
                Mood::Happy => 1.0,
                Mood::Neutral => 0.5,
                Mood::Sad => 0.3,
                Mood::Hungry => 0.1,
            };

            self.velocity.0 = self.rng.gen_range(-base_speed..base_speed);

            // Update direction based on velocity
            if self.velocity.0 > 0.1 {
                self.direction = Direction::Right;
            } else if self.velocity.0 < -0.1 {
                self.direction = Direction::Left;
            }
        }

        // Apply gravity when airborne
        if !self.is_grounded {
            self.velocity.1 += GRAVITY * dt * 60.0; // Scale by 60 for ~60 ticks/sec feel
        }

        // Apply friction (different for ground vs air)
        let friction = if self.is_grounded {
            GROUND_FRICTION
        } else {
            AIR_FRICTION
        };
        self.velocity.0 *= friction;

        // Update position
        self.position.0 += self.velocity.0;
        self.position.1 += self.velocity.1;

        // Ground collision - snap to ground when landing
        if self.position.1 >= self.ground_y {
            self.position.1 = self.ground_y;
            self.velocity.1 = 0.0;
            self.is_grounded = true;
        } else {
            self.is_grounded = false;
        }

        // Ceiling collision
        if self.position.1 < 0.0 {
            self.position.1 = 0.0;
            self.velocity.1 = 0.0; // Stop upward momentum
        }

        // Get frame dimensions
        let frame_width = 20.0;

        // Horizontal boundary collision
        if self.position.0 < 0.0 {
            self.position.0 = 0.0;
            self.velocity.0 = self.velocity.0.abs();
            self.direction = Direction::Right;
        } else if self.position.0 + frame_width > bounds.0 {
            self.position.0 = bounds.0 - frame_width;
            self.velocity.0 = -self.velocity.0.abs();
            self.direction = Direction::Left;
        }
    }

    /// Make the crab jump with the given strength
    pub fn jump(&mut self, strength: f32) {
        if self.is_grounded && self.jump_cooldown <= 0.0 {
            self.velocity.1 = -strength; // Negative = upward
            self.is_grounded = false;
            self.jump_cooldown = 0.3; // Short cooldown to prevent spam
        }
    }

    fn randomize_jump_strength(&mut self, base: f32) -> f32 {
        let variance = self.rng.gen_range(0.6..0.95);
        (base * variance).max(0.7)
    }

    /// Get the current animation frame as a string
    pub fn get_frame(&self) -> String {
        let is_moving = self.velocity.0.abs() > 0.05;
        let is_jumping = !self.is_grounded;

        // If jumping, celebrating, or ecstatic, use ecstatic frames
        if is_jumping || self.celebrating || self.mood == Mood::Ecstatic {
            let body = if self.frame_index % 2 == 0 {
                BodyTemplates::ECSTATIC_1
            } else {
                BodyTemplates::ECSTATIC_2
            };
            return build_frame(body, Eyes::ECSTATIC, Mouths::ECSTATIC);
        }

        // Determine eyes and mouth based on mood
        let (eyes, mouth) = match self.mood {
            Mood::Ecstatic => (Eyes::ECSTATIC, Mouths::ECSTATIC),
            Mood::Happy => (Eyes::HAPPY, Mouths::HAPPY),
            Mood::Neutral => (Eyes::NEUTRAL, Mouths::NEUTRAL),
            Mood::Sad => (Eyes::SAD, Mouths::SAD),
            Mood::Hungry => (Eyes::HUNGRY, Mouths::HUNGRY),
        };

        // Determine body pose based on mood and movement
        let body = match self.mood {
            Mood::Ecstatic => {
                // Already handled above, but for completeness
                if self.frame_index % 2 == 0 {
                    BodyTemplates::ECSTATIC_1
                } else {
                    BodyTemplates::ECSTATIC_2
                }
            }
            Mood::Happy => {
                if is_moving {
                    // Walking animation
                    if self.direction == Direction::Right {
                        if self.frame_index % 2 == 0 {
                            BodyTemplates::STANDING_RIGHT
                        } else {
                            BodyTemplates::WALKING_RIGHT
                        }
                    } else if self.frame_index % 2 == 0 {
                        BodyTemplates::STANDING_LEFT
                    } else {
                        BodyTemplates::WALKING_LEFT
                    }
                } else if self.frame_index % 4 == 0 {
                    // Occasional happy clap when idle
                    if self.direction == Direction::Right {
                        BodyTemplates::CLAPPING_RIGHT
                    } else {
                        BodyTemplates::CLAPPING_LEFT
                    }
                } else if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
            Mood::Neutral => {
                if is_moving {
                    if self.direction == Direction::Right {
                        if self.frame_index % 2 == 0 {
                            BodyTemplates::STANDING_RIGHT
                        } else {
                            BodyTemplates::WALKING_RIGHT
                        }
                    } else if self.frame_index % 2 == 0 {
                        BodyTemplates::STANDING_LEFT
                    } else {
                        BodyTemplates::WALKING_LEFT
                    }
                } else if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
            Mood::Sad => {
                // Sad crab doesn't move much, just stands
                if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
            Mood::Hungry => {
                // Alternate between standing and begging for effect
                if self.frame_index % 2 == 0 {
                    if self.direction == Direction::Right {
                        BodyTemplates::BEGGING_RIGHT
                    } else {
                        BodyTemplates::BEGGING_LEFT
                    }
                } else if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
        };

        build_frame(body, eyes, mouth)
    }

    /// Trigger celebration (e.g., when a new commit is detected)
    pub fn celebrate(&mut self) {
        self.celebrating = true;
        self.celebration_timer = 3.0; // 3 seconds of celebration
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
