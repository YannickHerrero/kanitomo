// ============================================================================
// Dash Game (Geometry Dash-inspired)
// ============================================================================

const DASH_PLAYFIELD_WIDTH: u16 = 48;
const DASH_PLAYFIELD_HEIGHT: u16 = 12;
const DASH_PLAYER_X: f32 = 6.0;
const DASH_GRAVITY: f32 = 35.0;
const DASH_JUMP_VELOCITY: f32 = -14.0;
const DASH_BASE_SPEED: f32 = 10.0;
const DASH_MAX_SPEED: f32 = 18.0;
const DASH_SPEED_RAMP: f32 = 0.6;
const DASH_SPAWN_AHEAD: f32 = 12.0;
const DASH_MAX_OBSTACLE_HEIGHT: f32 = 2.0;

const DASH_PATTERNS: &[(f32, f32, f32)] = &[
    (16.0, 1.0, 1.0),
    (18.0, 1.0, 2.0),
    (20.0, 2.0, 1.0),
    (16.0, 1.0, 1.0),
    (24.0, 1.0, 2.0),
    (18.0, 2.0, 2.0),
    (20.0, 1.0, 1.0),
    (16.0, 1.0, 2.0),
    (22.0, 1.0, 1.0),
];

#[derive(Debug, Clone)]
pub struct DashObstacle {
    pub x: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub struct DashGame {
    pub score: u32,
    pub distance: f32,
    pub speed: f32,
    pub bounds: (u16, u16),
    pub player_y: f32,
    pub player_vel: f32,
    pub obstacles: Vec<DashObstacle>,
    pub game_over: bool,
    pattern_index: usize,
}

impl DashGame {
    pub fn new(bounds: (u16, u16)) -> Self {
        let playfield = Self::calculate_playfield(bounds);
        let mut game = Self {
            score: 0,
            distance: 0.0,
            speed: DASH_BASE_SPEED,
            bounds: playfield,
            player_y: 0.0,
            player_vel: 0.0,
            obstacles: Vec::new(),
            game_over: false,
            pattern_index: 0,
        };

        game.player_y = game.ground_y();
        game.spawn_until_ahead();
        game
    }

    fn calculate_playfield(bounds: (u16, u16)) -> (u16, u16) {
        let width = bounds.0.saturating_sub(4).clamp(24, DASH_PLAYFIELD_WIDTH);
        let height = bounds.1.saturating_sub(4).clamp(8, DASH_PLAYFIELD_HEIGHT);
        (width, height)
    }

    pub fn update_bounds(&mut self, bounds: (u16, u16)) {
        let new_playfield = Self::calculate_playfield(bounds);
        if new_playfield != self.bounds {
            self.bounds = new_playfield;
            let ground = self.ground_y();
            if self.player_y > ground {
                self.player_y = ground;
                self.player_vel = 0.0;
            }
        }
    }

    pub fn jump(&mut self) {
        if self.is_on_ground() && !self.game_over {
            self.player_vel = DASH_JUMP_VELOCITY;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.game_over || self.bounds.0 == 0 || self.bounds.1 == 0 {
            return;
        }

        self.speed = (self.speed + DASH_SPEED_RAMP * dt).min(DASH_MAX_SPEED);
        self.distance += self.speed * dt;
        self.score = self.distance.floor().max(0.0) as u32;

        self.player_vel += DASH_GRAVITY * dt;
        self.player_y += self.player_vel * dt;

        let ground = self.ground_y();
        if self.player_y > ground {
            self.player_y = ground;
            self.player_vel = 0.0;
        }

        for obstacle in &mut self.obstacles {
            obstacle.x -= self.speed * dt;
        }

        self.obstacles.retain(|o| o.x + o.width >= -1.0);

        self.spawn_until_ahead();

        if self.check_collision() {
            self.game_over = true;
        }
    }

    pub fn player_x(&self) -> f32 {
        DASH_PLAYER_X
    }

    pub fn is_finished(&self) -> bool {
        self.game_over
    }

    fn ground_y(&self) -> f32 {
        self.bounds.1.saturating_sub(1) as f32
    }

    fn is_on_ground(&self) -> bool {
        (self.player_y - self.ground_y()).abs() < 0.01
    }

    fn max_obstacle_height(&self) -> f32 {
        (self.bounds.1 as f32 - 2.0)
            .max(1.0)
            .min(DASH_MAX_OBSTACLE_HEIGHT)
    }

    fn spawn_until_ahead(&mut self) {
        let target_x = self.bounds.0 as f32 + DASH_SPAWN_AHEAD;
        loop {
            let farthest = self
                .obstacles
                .iter()
                .map(|o| o.x + o.width)
                .fold(0.0, f32::max);
            if farthest >= target_x {
                break;
            }
            self.spawn_next_obstacle();
        }
    }

    fn spawn_next_obstacle(&mut self) {
        let (gap, width, height) = DASH_PATTERNS[self.pattern_index % DASH_PATTERNS.len()];
        self.pattern_index = self.pattern_index.wrapping_add(1);

        let start_x = if let Some(farthest) = self
            .obstacles
            .iter()
            .map(|o| o.x + o.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
        {
            farthest + gap
        } else {
            self.bounds.0 as f32 + 6.0
        };

        let clamped_height = height.min(self.max_obstacle_height());

        self.obstacles.push(DashObstacle {
            x: start_x,
            width,
            height: clamped_height,
        });
    }

    fn check_collision(&self) -> bool {
        let player_left = DASH_PLAYER_X;
        let player_right = DASH_PLAYER_X + 1.0;
        let player_top = self.player_y;
        let player_bottom = self.player_y + 1.0;

        let ground = self.ground_y();

        for obstacle in &self.obstacles {
            let left = obstacle.x;
            let right = obstacle.x + obstacle.width;
            let top = ground - obstacle.height + 1.0;
            let bottom = ground + 1.0;

            let overlaps_x = player_left < right && player_right > left;
            let overlaps_y = player_top < bottom && player_bottom > top;

            if overlaps_x && overlaps_y {
                return true;
            }
        }

        false
    }
}
