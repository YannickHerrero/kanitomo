use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const PLAYFIELD_WIDTH: u16 = 32;
const CRAB_SPRITE_WIDTH: u16 = 7;
const IDLE_THRESHOLD: f32 = 0.5; // Time before crab returns to neutral
const CATCH_CELEBRATION_TIME: f32 = 0.5; // Time to show happy face

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrabFacing {
    Left,
    Right,
    Neutral,
}
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
    pub facing: CrabFacing,
    spawn_timer: f32,
    idle_timer: f32,
    catch_timer: f32,
    rng: rand::rngs::ThreadRng,
}

impl CrabCatchGame {
    pub fn new(bounds: (u16, u16)) -> Self {
        let crab_width = CRAB_SPRITE_WIDTH;
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
            facing: CrabFacing::Neutral,
            spawn_timer: 0.0,
            idle_timer: 0.0,
            catch_timer: 0.0,
            rng: rand::thread_rng(),
        };

        game.reset_spawn_timer();
        game
    }

    pub fn crab_sprite(&self) -> &'static str {
        let is_happy = self.catch_timer > 0.0;
        match (self.facing, is_happy) {
            (CrabFacing::Neutral, false) => ">('_')<",
            (CrabFacing::Neutral, true) => ">(^_^)<",
            (CrabFacing::Right, false) => "(<'_')<",
            (CrabFacing::Right, true) => "(<^_^)<",
            (CrabFacing::Left, false) => ">('_'>)",
            (CrabFacing::Left, true) => ">(^_^>)",
        }
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

        // Update facing direction and reset idle timer
        self.facing = if direction > 0 {
            CrabFacing::Right
        } else {
            CrabFacing::Left
        };
        self.idle_timer = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if self.bounds.0 == 0 || self.bounds.1 == 0 {
            return;
        }

        // Update idle timer - switch to neutral after threshold
        self.idle_timer += dt;
        if self.idle_timer >= IDLE_THRESHOLD && self.facing != CrabFacing::Neutral {
            self.facing = CrabFacing::Neutral;
        }

        // Update catch celebration timer
        if self.catch_timer > 0.0 {
            self.catch_timer = (self.catch_timer - dt).max(0.0);
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
                    // Trigger happy face celebration
                    self.catch_timer = CATCH_CELEBRATION_TIME;
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

// ============================================================================
// Tetris Game
// ============================================================================

const TETRIS_GRID_WIDTH: usize = 10;
const TETRIS_GRID_HEIGHT: usize = 20;
const TETRIS_BASE_FALL_INTERVAL: f32 = 1.0; // 1 second at level 0
const TETRIS_MIN_FALL_INTERVAL: f32 = 0.05; // 50ms at max speed

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceType {
    pub fn shape(&self) -> Vec<Vec<bool>> {
        match self {
            // I piece: ....
            //          ####
            //          ....
            //          ....
            PieceType::I => vec![
                vec![false, false, false, false],
                vec![true, true, true, true],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            // O piece: .##.
            //          .##.
            //          ....
            //          ....
            PieceType::O => vec![
                vec![false, true, true, false],
                vec![false, true, true, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            // T piece: .#..
            //          ###.
            //          ....
            //          ....
            PieceType::T => vec![
                vec![false, true, false, false],
                vec![true, true, true, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            // S piece: .##.
            //          ##..
            //          ....
            //          ....
            PieceType::S => vec![
                vec![false, true, true, false],
                vec![true, true, false, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            // Z piece: ##..
            //          .##.
            //          ....
            //          ....
            PieceType::Z => vec![
                vec![true, true, false, false],
                vec![false, true, true, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            // J piece: #...
            //          ###.
            //          ....
            //          ....
            PieceType::J => vec![
                vec![true, false, false, false],
                vec![true, true, true, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            // L piece: ..#.
            //          ###.
            //          ....
            //          ....
            PieceType::L => vec![
                vec![false, false, true, false],
                vec![true, true, true, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
        }
    }

    fn all() -> [PieceType; 7] {
        [
            PieceType::I,
            PieceType::O,
            PieceType::T,
            PieceType::S,
            PieceType::Z,
            PieceType::J,
            PieceType::L,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub piece_type: PieceType,
    pub x: i32,
    pub y: i32,
    pub rotation: u8, // 0-3
}

impl Piece {
    fn new(piece_type: PieceType) -> Self {
        Self {
            piece_type,
            x: (TETRIS_GRID_WIDTH as i32 / 2) - 2, // Center horizontally
            y: 0,
            rotation: 0,
        }
    }

    fn shape(&self) -> Vec<Vec<bool>> {
        let base = self.piece_type.shape();
        let mut result = base.clone();

        // Rotate the shape based on rotation value
        for _ in 0..self.rotation {
            result = rotate_shape_clockwise(&result);
        }

        result
    }

    pub fn blocks(&self) -> Vec<(i32, i32)> {
        let shape = self.shape();
        let mut blocks = Vec::new();

        for (dy, row) in shape.iter().enumerate() {
            for (dx, &filled) in row.iter().enumerate() {
                if filled {
                    blocks.push((self.x + dx as i32, self.y + dy as i32));
                }
            }
        }

        blocks
    }
}

fn rotate_shape_clockwise(shape: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let n = shape.len();
    let mut rotated = vec![vec![false; n]; n];

    for i in 0..n {
        for j in 0..n {
            rotated[j][n - 1 - i] = shape[i][j];
        }
    }

    rotated
}

#[derive(Debug)]
pub struct TetrisGame {
    pub grid: Vec<Vec<Option<PieceType>>>,
    pub current_piece: Option<Piece>,
    pub next_piece: PieceType,
    pub score: u32,
    pub level: u32,
    pub lines_cleared: u32,
    pub game_over: bool,
    fall_timer: f32,
    fall_interval: f32,
    bag: Vec<PieceType>, // 7-bag randomizer
    rng: rand::rngs::ThreadRng,
}

impl TetrisGame {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut bag = Self::new_bag(&mut rng);
        let next_piece = bag.pop().unwrap();

        let mut game = Self {
            grid: vec![vec![None; TETRIS_GRID_WIDTH]; TETRIS_GRID_HEIGHT],
            current_piece: None,
            next_piece,
            score: 0,
            level: 0,
            lines_cleared: 0,
            game_over: false,
            fall_timer: 0.0,
            fall_interval: TETRIS_BASE_FALL_INTERVAL,
            bag,
            rng,
        };

        game.spawn_piece();
        game
    }

    fn new_bag(rng: &mut rand::rngs::ThreadRng) -> Vec<PieceType> {
        let mut bag = PieceType::all().to_vec();
        use rand::seq::SliceRandom;
        bag.shuffle(rng);
        bag
    }

    fn spawn_piece(&mut self) {
        if self.bag.is_empty() {
            self.bag = Self::new_bag(&mut self.rng);
        }

        let piece_type = self.next_piece;
        self.next_piece = self.bag.pop().unwrap();

        let piece = Piece::new(piece_type);

        // Check if spawn position is blocked
        if self.check_collision(&piece) {
            self.game_over = true;
            return;
        }

        self.current_piece = Some(piece);
    }

    fn check_collision(&self, piece: &Piece) -> bool {
        for (x, y) in piece.blocks() {
            // Check bounds
            if x < 0 || x >= TETRIS_GRID_WIDTH as i32 || y >= TETRIS_GRID_HEIGHT as i32 {
                return true;
            }

            // Check grid (allow y < 0 for spawn area above grid)
            if y >= 0 && self.grid[y as usize][x as usize].is_some() {
                return true;
            }
        }
        false
    }

    pub fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if let Some(piece) = &self.current_piece {
            let mut test_piece = piece.clone();
            test_piece.x += dx;
            test_piece.y += dy;

            if !self.check_collision(&test_piece) {
                if let Some(ref mut piece) = self.current_piece {
                    piece.x = test_piece.x;
                    piece.y = test_piece.y;
                }
                return true;
            }
        }
        false
    }

    pub fn rotate_piece(&mut self) {
        if let Some(piece) = &self.current_piece {
            // O piece doesn't need rotation
            if piece.piece_type == PieceType::O {
                return;
            }

            let mut test_piece = piece.clone();
            test_piece.rotation = (test_piece.rotation + 1) % 4;

            // Try basic rotation first
            if !self.check_collision(&test_piece) {
                if let Some(ref mut piece) = self.current_piece {
                    piece.rotation = test_piece.rotation;
                }
                return;
            }

            // Try wall kicks (simple offsets)
            let kicks = [(1, 0), (-1, 0), (0, -1), (2, 0), (-2, 0)];
            let orig_x = piece.x;
            let orig_y = piece.y;
            for (kick_x, kick_y) in kicks.iter() {
                test_piece.x = orig_x + kick_x;
                test_piece.y = orig_y + kick_y;

                if !self.check_collision(&test_piece) {
                    if let Some(ref mut piece) = self.current_piece {
                        piece.x = test_piece.x;
                        piece.y = test_piece.y;
                        piece.rotation = test_piece.rotation;
                    }
                    return;
                }
            }
        }
    }

    pub fn soft_drop(&mut self) {
        self.move_piece(0, 1);
    }

    pub fn hard_drop(&mut self) {
        while self.move_piece(0, 1) {}
        self.lock_piece();
    }

    fn lock_piece(&mut self) {
        if let Some(piece) = self.current_piece.take() {
            // Place piece on grid
            for (x, y) in piece.blocks() {
                if y >= 0 && y < TETRIS_GRID_HEIGHT as i32 && x >= 0 && x < TETRIS_GRID_WIDTH as i32
                {
                    self.grid[y as usize][x as usize] = Some(piece.piece_type);
                }
            }

            // Check for completed lines
            self.clear_lines();

            // Spawn next piece
            self.spawn_piece();
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_to_clear = Vec::new();

        for y in 0..TETRIS_GRID_HEIGHT {
            if self.grid[y].iter().all(|cell| cell.is_some()) {
                lines_to_clear.push(y);
            }
        }

        if lines_to_clear.is_empty() {
            return;
        }

        // Remove completed lines
        for &y in lines_to_clear.iter() {
            self.grid.remove(y);
            self.grid.insert(0, vec![None; TETRIS_GRID_WIDTH]);
        }

        // Update score and stats
        let lines_count = lines_to_clear.len() as u32;
        self.lines_cleared += lines_count;

        // Scoring: 1/2/3/4 lines = 100/300/500/800 * (level + 1)
        let base_score = match lines_count {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
        self.score += base_score * (self.level + 1);

        // Level up every 10 lines
        let new_level = self.lines_cleared / 10;
        if new_level > self.level {
            self.level = new_level;
            self.update_fall_speed();
        }
    }

    fn update_fall_speed(&mut self) {
        // Fall speed increases with level
        // Formula: base_interval * (0.9 ^ level), clamped to minimum
        self.fall_interval = (TETRIS_BASE_FALL_INTERVAL * 0.9_f32.powi(self.level as i32))
            .max(TETRIS_MIN_FALL_INTERVAL);
    }

    pub fn update(&mut self, dt: f32) {
        if self.game_over {
            return;
        }

        self.fall_timer += dt;
        if self.fall_timer >= self.fall_interval {
            self.fall_timer = 0.0;

            // Try to move piece down
            if !self.move_piece(0, 1) {
                // Can't move down, lock the piece
                self.lock_piece();
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.game_over
    }
}

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
