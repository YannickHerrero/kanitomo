use rand::seq::SliceRandom;
use std::collections::VecDeque;

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
        let mut new_head = (head.0 + dx, head.1 + dy);

        // Wall wrapping (teleport to other side)
        if new_head.0 < 0 {
            new_head.0 = self.bounds.0 as i32 - 1;
        } else if new_head.0 >= self.bounds.0 as i32 {
            new_head.0 = 0;
        }

        if new_head.1 < 0 {
            new_head.1 = self.bounds.1 as i32 - 1;
        } else if new_head.1 >= self.bounds.1 as i32 {
            new_head.1 = 0;
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
