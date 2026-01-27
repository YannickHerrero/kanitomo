use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const CRAB_CATCHER_SPRITE: &str = "<(o_o)>";
const PLAYFIELD_WIDTH: u16 = 32;
const FOOD_CHARS: &[char] = &['o', '*', '+', '@'];
const SPAWN_RANGE: std::ops::Range<f32> = 0.45..0.9;
const SPEED_RANGE: std::ops::Range<f32> = 3.0..7.0;

#[derive(Debug, Clone)]
pub struct FallingFood {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub glyph: char,
}

#[derive(Debug)]
pub struct CrabCatchGame {
    pub score: u32,
    pub misses: u32,
    pub foods: Vec<FallingFood>,
    pub crab_x: i32,
    pub crab_width: u16,
    pub move_step: i32,
    pub bounds: (u16, u16),
    pub start_time: Instant,
    pub duration: Duration,
    spawn_timer: f32,
    rng: rand::rngs::ThreadRng,
}

impl CrabCatchGame {
    pub fn new(bounds: (u16, u16)) -> Self {
        let crab_width = CRAB_CATCHER_SPRITE.len() as u16;
        let playfield_width = Self::playfield_width(bounds.0);
        let initial_x = playfield_width.saturating_sub(crab_width) as i32 / 2;
        let move_step = Self::move_step(playfield_width, crab_width);

        let mut game = Self {
            score: 0,
            misses: 0,
            foods: Vec::new(),
            crab_x: initial_x,
            crab_width,
            move_step,
            bounds: (playfield_width, bounds.1),
            start_time: Instant::now(),
            duration: Duration::from_secs(20),
            spawn_timer: 0.0,
            rng: rand::thread_rng(),
        };

        game.reset_spawn_timer();
        game
    }

    pub fn crab_sprite(&self) -> &'static str {
        CRAB_CATCHER_SPRITE
    }

    pub fn update_bounds(&mut self, bounds: (u16, u16)) {
        let playfield_width = Self::playfield_width(bounds.0);
        self.bounds = (playfield_width, bounds.1);
        self.move_step = Self::move_step(playfield_width, self.crab_width);
        self.crab_x = self
            .crab_x
            .clamp(0, playfield_width.saturating_sub(self.crab_width) as i32);
    }

    pub fn remaining_time(&self) -> Duration {
        self.duration
            .checked_sub(self.start_time.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0))
    }

    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    pub fn move_crab(&mut self, direction: i32) {
        let max_x = self.bounds.0.saturating_sub(self.crab_width) as i32;
        let delta = direction * self.move_step;
        self.crab_x = (self.crab_x + delta).clamp(0, max_x);
    }

    pub fn update(&mut self, dt: f32) {
        if self.bounds.0 == 0 || self.bounds.1 == 0 {
            return;
        }

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_food();
            self.reset_spawn_timer();
        }

        let catch_y = self.bounds.1.saturating_sub(2) as f32;
        let crab_min = self.crab_x;
        let crab_max = self.crab_x + self.crab_width as i32 - 1;

        for food in &mut self.foods {
            food.y += food.speed * dt;
        }

        let mut next_foods = Vec::with_capacity(self.foods.len());
        for food in self.foods.drain(..) {
            if food.y >= catch_y {
                let food_x = food.x.round() as i32;
                if food_x >= crab_min && food_x <= crab_max {
                    self.score += 1;
                } else {
                    self.misses += 1;
                }
            } else {
                next_foods.push(food);
            }
        }
        self.foods = next_foods;
    }

    fn spawn_food(&mut self) {
        let width = self.bounds.0.max(1) as f32;
        let max_x = (width - 1.0).max(0.0);
        let x = self.rng.gen_range(0.0..=max_x);
        let speed = self.rng.gen_range(SPEED_RANGE);
        let glyph = *FOOD_CHARS.choose(&mut self.rng).unwrap_or(&'o');

        self.foods.push(FallingFood {
            x,
            y: 0.0,
            speed,
            glyph,
        });
    }

    fn reset_spawn_timer(&mut self) {
        self.spawn_timer = self.rng.gen_range(SPAWN_RANGE);
    }

    fn playfield_width(width: u16) -> u16 {
        let available = width.saturating_sub(2).max(1);
        available.min(PLAYFIELD_WIDTH).max(8)
    }

    fn move_step(width: u16, crab_width: u16) -> i32 {
        let travel = width.saturating_sub(crab_width).max(1);
        let step = (travel as f32 / 7.0).ceil().max(1.0);
        step as i32
    }
}

// ============================================================================
// Snake Game
// ============================================================================

const SNAKE_PLAYFIELD_WIDTH: u16 = 32;
const SNAKE_PLAYFIELD_HEIGHT: u16 = 16;
const SNAKE_BASE_SPEED: f32 = 0.20; // seconds per move
const SNAKE_MIN_SPEED: f32 = 0.08; // minimum seconds per move (max speed)
const SNAKE_SPEED_DECREASE: f32 = 0.008; // speed increase per food eaten

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns true if the given direction is opposite to self
    pub fn is_opposite(&self, other: &Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }

    /// Returns the delta (dx, dy) for this direction
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Debug)]
pub struct SnakeGame {
    pub score: u32,
    pub snake: VecDeque<(i32, i32)>, // Body segments (head first)
    pub direction: Direction,
    next_direction: Direction, // Buffered input direction
    pub food: (i32, i32),
    pub bounds: (u16, u16), // Playfield dimensions (width, height)
    pub game_over: bool,
    move_timer: f32,
    current_speed: f32, // Current move interval (decreases as score increases)
    rng: rand::rngs::ThreadRng,
}

impl SnakeGame {
    pub fn new(bounds: (u16, u16)) -> Self {
        let playfield = Self::calculate_playfield(bounds);
        let mut rng = rand::thread_rng();

        // Start snake in the center, moving right
        let center_x = playfield.0 as i32 / 2;
        let center_y = playfield.1 as i32 / 2;

        let mut snake = VecDeque::new();
        // Head first, then body segments going left
        snake.push_back((center_x, center_y));
        snake.push_back((center_x - 1, center_y));
        snake.push_back((center_x - 2, center_y));

        // Spawn initial food
        let food = Self::spawn_food_position(&snake, playfield, &mut rng);

        Self {
            score: 0,
            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food,
            bounds: playfield,
            game_over: false,
            move_timer: SNAKE_BASE_SPEED,
            current_speed: SNAKE_BASE_SPEED,
            rng,
        }
    }

    fn calculate_playfield(bounds: (u16, u16)) -> (u16, u16) {
        let width = bounds.0.saturating_sub(4).clamp(10, SNAKE_PLAYFIELD_WIDTH);
        let height = bounds.1.saturating_sub(2).clamp(6, SNAKE_PLAYFIELD_HEIGHT);
        (width, height)
    }

    pub fn update_bounds(&mut self, bounds: (u16, u16)) {
        let new_playfield = Self::calculate_playfield(bounds);
        if new_playfield != self.bounds {
            self.bounds = new_playfield;
            // Clamp snake positions to new bounds
            for segment in &mut self.snake {
                segment.0 = segment.0.clamp(0, self.bounds.0 as i32 - 1);
                segment.1 = segment.1.clamp(0, self.bounds.1 as i32 - 1);
            }
            // Respawn food if out of bounds
            if self.food.0 >= self.bounds.0 as i32 || self.food.1 >= self.bounds.1 as i32 {
                self.food = Self::spawn_food_position(&self.snake, self.bounds, &mut self.rng);
            }
        }
    }

    /// Set the direction for the next move (prevents 180-degree turns)
    pub fn set_direction(&mut self, dir: Direction) {
        if !self.direction.is_opposite(&dir) {
            self.next_direction = dir;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.game_over || self.bounds.0 == 0 || self.bounds.1 == 0 {
            return;
        }

        self.move_timer -= dt;
        if self.move_timer <= 0.0 {
            self.move_snake();
            self.move_timer = self.current_speed;
        }
    }

    fn move_snake(&mut self) {
        // Apply buffered direction
        self.direction = self.next_direction;

        // Calculate new head position
        let head = self.snake.front().copied().unwrap_or((0, 0));
        let (dx, dy) = self.direction.delta();
        let new_head = (head.0 + dx, head.1 + dy);

        // Check wall collision
        if new_head.0 < 0
            || new_head.0 >= self.bounds.0 as i32
            || new_head.1 < 0
            || new_head.1 >= self.bounds.1 as i32
        {
            self.game_over = true;
            return;
        }

        // Check self collision (excluding tail since it will move)
        let tail_will_move = new_head != self.food;
        for (i, segment) in self.snake.iter().enumerate() {
            // Skip the last segment if tail will move (it won't be there)
            if tail_will_move && i == self.snake.len() - 1 {
                continue;
            }
            if *segment == new_head {
                self.game_over = true;
                return;
            }
        }

        // Move snake: add new head
        self.snake.push_front(new_head);

        // Check if we ate food
        if new_head == self.food {
            self.score += 1;
            // Increase speed (decrease move interval)
            self.current_speed = (self.current_speed - SNAKE_SPEED_DECREASE).max(SNAKE_MIN_SPEED);
            // Spawn new food
            self.food = Self::spawn_food_position(&self.snake, self.bounds, &mut self.rng);
            // Don't remove tail - snake grows
        } else {
            // Remove tail - snake moves
            self.snake.pop_back();
        }
    }

    fn spawn_food_position(
        snake: &VecDeque<(i32, i32)>,
        bounds: (u16, u16),
        rng: &mut rand::rngs::ThreadRng,
    ) -> (i32, i32) {
        // Collect all possible positions
        let mut available: Vec<(i32, i32)> = Vec::new();
        for x in 0..bounds.0 as i32 {
            for y in 0..bounds.1 as i32 {
                if !snake.contains(&(x, y)) {
                    available.push((x, y));
                }
            }
        }

        // Pick a random available position
        if let Some(&pos) = available.choose(rng) {
            pos
        } else {
            // Fallback if somehow no space (shouldn't happen)
            (0, 0)
        }
    }

    pub fn is_finished(&self) -> bool {
        self.game_over
    }
}
