use rand::seq::SliceRandom;
use rand::Rng;
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
