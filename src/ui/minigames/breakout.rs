use rand::Rng;

// ============================================================================
// Breakout Game
// ============================================================================

const BREAKOUT_PLAYFIELD_WIDTH: u16 = 40;
const BREAKOUT_PLAYFIELD_HEIGHT: u16 = 20;
const BREAKOUT_PADDLE_WIDTH: u16 = 8;
const BREAKOUT_BALL_BASE_SPEED: f32 = 8.0;
const BREAKOUT_BALL_MAX_SPEED: f32 = 18.0;
const BREAKOUT_BRICK_WIDTH: u16 = 3;
const BREAKOUT_BRICK_ROWS: u16 = 5;

#[derive(Debug, Clone)]
pub struct Brick {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub points: u32,
}

#[derive(Debug)]
pub struct BreakoutGame {
    pub score: u32,
    pub lives: u8,
    pub paddle_x: f32,
    pub paddle_width: u16,
    pub ball_pos: (f32, f32),
    pub ball_vel: (f32, f32),
    pub ball_launched: bool,
    pub bricks: Vec<Brick>,
    pub bounds: (u16, u16),
    pub game_over: bool,
    pub victory: bool,
    current_speed: f32,
    total_bricks: usize,
    rng: rand::rngs::ThreadRng,
}

impl BreakoutGame {
    pub fn new(bounds: (u16, u16)) -> Self {
        let playfield = Self::calculate_playfield(bounds);
        let paddle_x = (playfield.0 as f32 - BREAKOUT_PADDLE_WIDTH as f32) / 2.0;

        let mut game = Self {
            score: 0,
            lives: 3,
            paddle_x,
            paddle_width: BREAKOUT_PADDLE_WIDTH,
            ball_pos: (0.0, 0.0), // Will be set by reset_ball
            ball_vel: (0.0, 0.0),
            ball_launched: false,
            bricks: Vec::new(),
            bounds: playfield,
            game_over: false,
            victory: false,
            current_speed: BREAKOUT_BALL_BASE_SPEED,
            total_bricks: 0,
            rng: rand::thread_rng(),
        };

        game.generate_bricks();
        game.total_bricks = game.bricks.len();
        game.reset_ball();
        game
    }

    fn calculate_playfield(bounds: (u16, u16)) -> (u16, u16) {
        let width = bounds
            .0
            .saturating_sub(4)
            .clamp(20, BREAKOUT_PLAYFIELD_WIDTH);
        let height = bounds
            .1
            .saturating_sub(2)
            .clamp(12, BREAKOUT_PLAYFIELD_HEIGHT);
        (width, height)
    }

    pub fn update_bounds(&mut self, bounds: (u16, u16)) {
        let new_playfield = Self::calculate_playfield(bounds);
        if new_playfield != self.bounds {
            // Scale paddle position proportionally
            let scale_x = new_playfield.0 as f32 / self.bounds.0.max(1) as f32;
            self.paddle_x =
                (self.paddle_x * scale_x).clamp(0.0, (new_playfield.0 - self.paddle_width) as f32);

            // Scale ball position proportionally
            let scale_y = new_playfield.1 as f32 / self.bounds.1.max(1) as f32;
            self.ball_pos.0 = (self.ball_pos.0 * scale_x).clamp(0.0, new_playfield.0 as f32 - 1.0);
            self.ball_pos.1 = (self.ball_pos.1 * scale_y).clamp(0.0, new_playfield.1 as f32 - 1.0);

            self.bounds = new_playfield;

            // Regenerate bricks if needed (keeps game playable on resize)
            if self.bricks.iter().any(|b| b.x + b.width > new_playfield.0) {
                self.generate_bricks();
            }
        }
    }

    fn generate_bricks(&mut self) {
        self.bricks.clear();

        let brick_width = BREAKOUT_BRICK_WIDTH;
        let gap = 1u16;
        let bricks_per_row = (self.bounds.0 - 2) / (brick_width + gap);
        let total_brick_width = bricks_per_row * (brick_width + gap) - gap;
        let start_x = (self.bounds.0 - total_brick_width) / 2;

        // Point values for each row (top rows worth more)
        let row_points = [50, 40, 30, 20, 10];

        for row in 0..BREAKOUT_BRICK_ROWS.min(self.bounds.1 / 3) {
            let y = row + 2; // Start 2 rows from top
            let points = row_points.get(row as usize).copied().unwrap_or(10);

            for col in 0..bricks_per_row {
                let x = start_x + col * (brick_width + gap);
                self.bricks.push(Brick {
                    x,
                    y,
                    width: brick_width,
                    points,
                });
            }
        }
    }

    fn reset_ball(&mut self) {
        // Place ball on top of paddle
        let paddle_center = self.paddle_x + self.paddle_width as f32 / 2.0;
        self.ball_pos = (paddle_center, self.bounds.1 as f32 - 3.0);
        self.ball_vel = (0.0, 0.0);
        self.ball_launched = false;
    }

    pub fn launch_ball(&mut self) {
        if !self.ball_launched && !self.game_over {
            // Launch at an angle (slightly random)
            let angle: f32 = self.rng.gen_range(-0.5..0.5);
            let speed = self.current_speed;
            self.ball_vel = (angle * speed, -speed);
            self.ball_launched = true;
        }
    }

    pub fn move_paddle(&mut self, direction: f32) {
        let move_speed = 2.0;
        self.paddle_x += direction * move_speed;
        self.paddle_x = self
            .paddle_x
            .clamp(0.0, (self.bounds.0 - self.paddle_width) as f32);

        // If ball not launched, move it with paddle
        if !self.ball_launched {
            let paddle_center = self.paddle_x + self.paddle_width as f32 / 2.0;
            self.ball_pos.0 = paddle_center;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.game_over || self.victory || !self.ball_launched {
            return;
        }

        // Move ball
        self.ball_pos.0 += self.ball_vel.0 * dt;
        self.ball_pos.1 += self.ball_vel.1 * dt;

        // Wall collisions (left/right)
        if self.ball_pos.0 <= 0.0 {
            self.ball_pos.0 = 0.0;
            self.ball_vel.0 = self.ball_vel.0.abs();
        } else if self.ball_pos.0 >= self.bounds.0 as f32 - 1.0 {
            self.ball_pos.0 = self.bounds.0 as f32 - 1.0;
            self.ball_vel.0 = -self.ball_vel.0.abs();
        }

        // Top wall collision
        if self.ball_pos.1 <= 0.0 {
            self.ball_pos.1 = 0.0;
            self.ball_vel.1 = self.ball_vel.1.abs();
        }

        // Bottom - lose life
        if self.ball_pos.1 >= self.bounds.1 as f32 {
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.game_over = true;
            } else {
                self.reset_ball();
            }
            return;
        }

        // Paddle collision
        let paddle_y = self.bounds.1 as f32 - 2.0;
        let paddle_left = self.paddle_x;
        let paddle_right = self.paddle_x + self.paddle_width as f32;

        if self.ball_vel.1 > 0.0
            && self.ball_pos.1 >= paddle_y - 0.5
            && self.ball_pos.1 <= paddle_y + 0.5
            && self.ball_pos.0 >= paddle_left - 0.5
            && self.ball_pos.0 <= paddle_right + 0.5
        {
            // Calculate bounce angle based on hit position
            let hit_pos = (self.ball_pos.0 - paddle_left) / self.paddle_width as f32;
            let angle = (hit_pos - 0.5) * 1.2; // -0.6 to 0.6 based on position

            let speed = (self.ball_vel.0.powi(2) + self.ball_vel.1.powi(2)).sqrt();
            self.ball_vel.0 = angle * speed;
            self.ball_vel.1 = -((speed.powi(2) - self.ball_vel.0.powi(2)).sqrt());

            self.ball_pos.1 = paddle_y - 0.5;
        }

        // Brick collisions
        self.check_brick_collisions();

        // Check victory
        if self.bricks.is_empty() {
            self.victory = true;
        }
    }

    fn check_brick_collisions(&mut self) {
        let ball_x = self.ball_pos.0.round() as i32;
        let ball_y = self.ball_pos.1.round() as i32;

        let mut hit_index = None;
        let mut hit_side = HitSide::None;

        for (i, brick) in self.bricks.iter().enumerate() {
            let brick_left = brick.x as i32;
            let brick_right = (brick.x + brick.width) as i32 - 1;
            let brick_top = brick.y as i32;
            let brick_bottom = brick.y as i32;

            // Check if ball is hitting this brick
            if ball_x >= brick_left - 1
                && ball_x <= brick_right + 1
                && ball_y >= brick_top - 1
                && ball_y <= brick_bottom + 1
            {
                // Determine which side was hit
                let from_left = (ball_x - brick_left).abs();
                let from_right = (ball_x - brick_right).abs();
                let from_top = (ball_y - brick_top).abs();
                let from_bottom = (ball_y - brick_bottom).abs();

                let min_dist = from_left.min(from_right).min(from_top).min(from_bottom);

                hit_side = if min_dist == from_top || min_dist == from_bottom {
                    HitSide::Vertical
                } else {
                    HitSide::Horizontal
                };

                hit_index = Some(i);
                break;
            }
        }

        if let Some(i) = hit_index {
            let brick = self.bricks.remove(i);
            self.score += brick.points;

            // Bounce based on hit side
            match hit_side {
                HitSide::Vertical => self.ball_vel.1 = -self.ball_vel.1,
                HitSide::Horizontal => self.ball_vel.0 = -self.ball_vel.0,
                HitSide::None => {}
            }

            // Increase speed slightly
            let destroyed = self.total_bricks - self.bricks.len();
            if destroyed.is_multiple_of(5) {
                self.current_speed = (self.current_speed * 1.05).min(BREAKOUT_BALL_MAX_SPEED);
                // Normalize velocity to new speed
                let current = (self.ball_vel.0.powi(2) + self.ball_vel.1.powi(2)).sqrt();
                if current > 0.0 {
                    let scale = self.current_speed / current;
                    self.ball_vel.0 *= scale;
                    self.ball_vel.1 *= scale;
                }
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.game_over || self.victory
    }
}

#[derive(Debug, Clone, Copy)]
enum HitSide {
    None,
    Vertical,
    Horizontal,
}
