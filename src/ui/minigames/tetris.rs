// ============================================================================
// Tetris Game
// ============================================================================

const TETRIS_GRID_WIDTH: usize = 10;
const TETRIS_GRID_HEIGHT: usize = 20;
const TETRIS_BASE_FALL_INTERVAL: f32 = 1.0; // 1 second at level 0
const TETRIS_MIN_FALL_INTERVAL: f32 = 0.05; // 50ms at max speed
const TETRIS_DIG_GARBAGE_ROWS: usize = 10; // Number of garbage rows for Dig mode

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TetrisMode {
    Normal,   // Classic mode with level progression
    Sprint,   // Race to clear 40 lines as fast as possible
    Zen,      // Relaxed mode with no speed increase
    Dig,      // Clear pre-filled garbage lines
    Survival, // Increasingly fast mode
}

impl TetrisMode {
    pub fn name(&self) -> &str {
        match self {
            TetrisMode::Normal => "Normal",
            TetrisMode::Sprint => "Sprint",
            TetrisMode::Zen => "Zen",
            TetrisMode::Dig => "Dig",
            TetrisMode::Survival => "Survival",
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationState {
    Zero = 0, // Spawn state
    R = 1,    // Clockwise from spawn
    Two = 2,  // 180 from spawn
    L = 3,    // Counter-clockwise from spawn
}

impl RotationState {
    fn from_u8(val: u8) -> Self {
        match val % 4 {
            0 => RotationState::Zero,
            1 => RotationState::R,
            2 => RotationState::Two,
            3 => RotationState::L,
            _ => unreachable!(),
        }
    }

    fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub piece_type: PieceType,
    pub x: i32,
    pub y: i32,
    pub rotation: RotationState,
}

impl Piece {
    fn new(piece_type: PieceType) -> Self {
        Self {
            piece_type,
            x: (TETRIS_GRID_WIDTH as i32 / 2) - 2, // Center horizontally
            y: 0,
            rotation: RotationState::Zero,
        }
    }

    pub fn blocks(&self) -> Vec<(i32, i32)> {
        let mut blocks = Vec::new();
        for (dx, dy) in blocks_for_piece(self.piece_type, self.rotation) {
            blocks.push((self.x + dx, self.y + dy));
        }
        blocks
    }
}

fn blocks_for_piece(piece_type: PieceType, rotation: RotationState) -> Vec<(i32, i32)> {
    let base = match piece_type {
        PieceType::T => vec![(1, 0), (0, 1), (1, 1), (2, 1)],
        PieceType::J => vec![(0, 0), (0, 1), (1, 1), (2, 1)],
        PieceType::L => vec![(2, 0), (0, 1), (1, 1), (2, 1)],
        PieceType::S => vec![(1, 0), (2, 0), (0, 1), (1, 1)],
        PieceType::Z => vec![(0, 0), (1, 0), (1, 1), (2, 1)],
        PieceType::O => vec![(1, 0), (2, 0), (1, 1), (2, 1)],
        PieceType::I => vec![(0, 1), (1, 1), (2, 1), (3, 1)],
    };

    let rotations = rotation.to_u8();
    if piece_type == PieceType::O || rotations == 0 {
        return base;
    }

    let mut rotated = base;
    for _ in 0..rotations {
        rotated = rotated
            .into_iter()
            .map(|(x, y)| match piece_type {
                PieceType::I => (3 - y, x), // Rotate around (1.5, 1.5)
                _ => (2 - y, x),            // Rotate around (1, 1)
            })
            .collect();
    }
    rotated
}

// SRS kick tables for J, L, S, T, Z pieces
// NOTE: Y-axis is inverted (positive Y = down) compared to SRS standard (positive Y = up)
fn get_jlstz_kicks(from: RotationState, to: RotationState) -> [(i32, i32); 5] {
    match (from, to) {
        (RotationState::Zero, RotationState::R) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        (RotationState::R, RotationState::Zero) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        (RotationState::R, RotationState::Two) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        (RotationState::Two, RotationState::R) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        (RotationState::Two, RotationState::L) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (RotationState::L, RotationState::Two) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        (RotationState::L, RotationState::Zero) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        (RotationState::Zero, RotationState::L) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        _ => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)], // Should not happen
    }
}

// SRS kick tables for I piece
// NOTE: Y-axis is inverted (positive Y = down) compared to SRS standard (positive Y = up)
fn get_i_kicks(from: RotationState, to: RotationState) -> [(i32, i32); 5] {
    match (from, to) {
        (RotationState::Zero, RotationState::R) => [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
        (RotationState::R, RotationState::Zero) => [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
        (RotationState::R, RotationState::Two) => [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
        (RotationState::Two, RotationState::R) => [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
        (RotationState::Two, RotationState::L) => [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
        (RotationState::L, RotationState::Two) => [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
        (RotationState::L, RotationState::Zero) => [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
        (RotationState::Zero, RotationState::L) => [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
        _ => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)], // Should not happen
    }
}

#[derive(Debug)]
pub struct TetrisGame {
    pub mode: TetrisMode,
    pub grid: Vec<Vec<Option<PieceType>>>,
    pub current_piece: Option<Piece>,
    pub next_piece: PieceType,
    pub hold_piece: Option<PieceType>,
    pub can_hold: bool, // Can only hold once per piece
    pub score: u32,
    pub level: u32,
    pub lines_cleared: u32,
    pub game_over: bool,
    fall_timer: f32,
    fall_interval: f32,
    bag: Vec<PieceType>, // 7-bag randomizer
    rng: rand::rngs::ThreadRng,
    // Lock delay (Tetr.io style: longer than guideline 0.5s)
    lock_delay_timer: f32,
    lock_delay_max: f32, // ~1.0s for Tetr.io feel
    lock_delay_active: bool,
    lock_delay_resets: u32,
    lock_delay_max_resets: u32, // ~15-20 for Tetr.io feel
    last_action_was_rotation: bool,
    last_kick_index: usize,
    // Mode-specific fields
    pub elapsed_time: f32, // For Sprint mode timing
    pub target_lines: u32, // For Sprint mode (40 lines)
}

impl TetrisGame {
    pub fn new(mode: TetrisMode) -> Self {
        let mut rng = rand::thread_rng();
        let mut bag = Self::new_bag(&mut rng);
        let next_piece = bag.pop().unwrap();

        let mut game = Self {
            mode,
            grid: vec![vec![None; TETRIS_GRID_WIDTH]; TETRIS_GRID_HEIGHT],
            current_piece: None,
            next_piece,
            hold_piece: None,
            can_hold: true,
            score: 0,
            level: 0,
            lines_cleared: 0,
            game_over: false,
            fall_timer: 0.0,
            fall_interval: TETRIS_BASE_FALL_INTERVAL,
            bag,
            rng,
            lock_delay_timer: 0.0,
            lock_delay_max: 1.0, // 1 second for Tetr.io feel (vs 0.5s guideline)
            lock_delay_active: false,
            lock_delay_resets: 0,
            lock_delay_max_resets: 20, // 20 resets for Tetr.io feel (vs 15 guideline)
            last_action_was_rotation: false,
            last_kick_index: 0,
            elapsed_time: 0.0,
            target_lines: if mode == TetrisMode::Sprint { 40 } else { 0 },
        };

        // Initialize mode-specific setup
        game.setup_mode();
        game.spawn_piece();
        game
    }

    fn setup_mode(&mut self) {
        match self.mode {
            TetrisMode::Dig => {
                // Fill bottom rows with garbage (with one random hole per row)
                use rand::Rng;
                for y in (TETRIS_GRID_HEIGHT - TETRIS_DIG_GARBAGE_ROWS)..TETRIS_GRID_HEIGHT {
                    let hole_x = self.rng.gen_range(0..TETRIS_GRID_WIDTH);
                    for x in 0..TETRIS_GRID_WIDTH {
                        if x != hole_x {
                            // Use a gray color for garbage
                            self.grid[y][x] = Some(PieceType::L); // Reuse L for garbage visual
                        }
                    }
                }
            }
            TetrisMode::Survival => {
                // Start at a higher level for faster initial speed
                self.level = 5;
                self.update_fall_speed();
            }
            _ => {}
        }
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
                self.last_action_was_rotation = false;

                // Reset lock delay if piece was grounded
                if self.lock_delay_active && self.lock_delay_resets < self.lock_delay_max_resets {
                    self.lock_delay_timer = 0.0;
                    self.lock_delay_resets += 1;
                }

                return true;
            }
        }
        false
    }

    pub fn rotate_piece_cw(&mut self) {
        self.rotate_piece_internal(true);
    }

    pub fn rotate_piece_ccw(&mut self) {
        self.rotate_piece_internal(false);
    }

    fn rotate_piece_internal(&mut self, clockwise: bool) {
        if let Some(piece) = &self.current_piece {
            // O piece doesn't need rotation
            if piece.piece_type == PieceType::O {
                return;
            }

            let from_state = piece.rotation;
            let to_state = if clockwise {
                RotationState::from_u8((from_state.to_u8() + 1) % 4)
            } else {
                RotationState::from_u8((from_state.to_u8() + 3) % 4)
            };

            // Get kick table based on piece type
            let kicks = if piece.piece_type == PieceType::I {
                get_i_kicks(from_state, to_state)
            } else {
                get_jlstz_kicks(from_state, to_state)
            };

            let mut test_piece = piece.clone();
            test_piece.rotation = to_state;

            // Try each kick offset in order
            for (kick_index, (kick_x, kick_y)) in kicks.iter().enumerate() {
                test_piece.x = piece.x + kick_x;
                test_piece.y = piece.y + kick_y;

                let collides = self.check_collision(&test_piece);

                if !collides {
                    if let Some(ref mut piece) = self.current_piece {
                        piece.x = test_piece.x;
                        piece.y = test_piece.y;
                        piece.rotation = test_piece.rotation;
                    }
                    self.last_action_was_rotation = true;
                    self.last_kick_index = kick_index;

                    // Reset lock delay if piece was grounded
                    if self.lock_delay_active && self.lock_delay_resets < self.lock_delay_max_resets
                    {
                        self.lock_delay_timer = 0.0;
                        self.lock_delay_resets += 1;
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

    pub fn hold(&mut self) {
        // Can only hold once per piece
        if !self.can_hold {
            return;
        }

        if let Some(current) = self.current_piece.take() {
            if let Some(held_type) = self.hold_piece {
                // Swap current with hold
                self.hold_piece = Some(current.piece_type);
                self.current_piece = Some(Piece::new(held_type));
            } else {
                // First time holding, store current and spawn next
                self.hold_piece = Some(current.piece_type);
                self.spawn_piece();
            }

            // Reset lock delay state
            self.lock_delay_active = false;
            self.lock_delay_timer = 0.0;
            self.lock_delay_resets = 0;
            self.last_action_was_rotation = false;

            // Can't hold again until piece locks
            self.can_hold = false;
        }
    }

    fn is_grounded(&self) -> bool {
        if let Some(piece) = &self.current_piece {
            let mut test_piece = piece.clone();
            test_piece.y += 1;
            self.check_collision(&test_piece)
        } else {
            false
        }
    }

    fn check_t_spin(&self, piece: &Piece) -> Option<bool> {
        // Only T pieces can T-Spin
        if piece.piece_type != PieceType::T {
            return None;
        }

        // Last action must have been a rotation
        if !self.last_action_was_rotation {
            return None;
        }

        // Get the 4 diagonal corners around the T's center
        // T center is at position (1, 1) in the 4x4 bounding box when rotation is 0
        let center_x = piece.x + 1;
        let center_y = piece.y + 1;

        let corners = [
            (center_x - 1, center_y - 1), // Top-left
            (center_x + 1, center_y - 1), // Top-right
            (center_x - 1, center_y + 1), // Bottom-left
            (center_x + 1, center_y + 1), // Bottom-right
        ];

        // Count occupied corners (out of bounds counts as occupied)
        let mut occupied_count = 0;
        let mut front_corners_occupied = 0;

        for (i, (x, y)) in corners.iter().enumerate() {
            let is_occupied = *x < 0
                || *x >= TETRIS_GRID_WIDTH as i32
                || *y < 0
                || *y >= TETRIS_GRID_HEIGHT as i32
                || self.grid[*y as usize][*x as usize].is_some();

            if is_occupied {
                occupied_count += 1;

                // Determine which are front corners based on rotation
                let is_front = match piece.rotation {
                    RotationState::Zero => i == 0 || i == 1, // Top corners
                    RotationState::R => i == 1 || i == 3,    // Right corners
                    RotationState::Two => i == 2 || i == 3,  // Bottom corners
                    RotationState::L => i == 0 || i == 2,    // Left corners
                };

                if is_front {
                    front_corners_occupied += 1;
                }
            }
        }

        // Need at least 3 corners occupied for T-Spin
        if occupied_count < 3 {
            return None;
        }

        // Determine if it's a proper T-Spin or Mini T-Spin
        // Proper: 2 front corners occupied, OR last kick was the (0, -2) or similar offset
        let is_proper = front_corners_occupied == 2 || self.last_kick_index == 4;

        Some(is_proper)
    }

    fn lock_piece(&mut self) {
        if let Some(piece) = self.current_piece.take() {
            // Check for T-Spin before locking
            let t_spin_type = self.check_t_spin(&piece);

            // Place piece on grid
            for (x, y) in piece.blocks() {
                if y >= 0 && y < TETRIS_GRID_HEIGHT as i32 && x >= 0 && x < TETRIS_GRID_WIDTH as i32
                {
                    self.grid[y as usize][x as usize] = Some(piece.piece_type);
                }
            }

            // Check for completed lines
            let lines_cleared = self.clear_lines_and_return_count();

            // Award T-Spin bonus points if applicable
            if let Some(is_proper) = t_spin_type {
                let base_score = if is_proper {
                    match lines_cleared {
                        0 => 400,
                        1 => 800,
                        2 => 1200,
                        3 => 1600,
                        _ => 0,
                    }
                } else {
                    // Mini T-Spin
                    match lines_cleared {
                        0 => 100,
                        1 => 200,
                        2 => 400,
                        _ => 0,
                    }
                };
                self.score += base_score * (self.level + 1);
            }

            // Reset lock delay state
            self.lock_delay_active = false;
            self.lock_delay_timer = 0.0;
            self.lock_delay_resets = 0;
            self.last_action_was_rotation = false;

            // Allow hold again
            self.can_hold = true;

            // Spawn next piece
            self.spawn_piece();
        }
    }

    fn clear_lines_and_return_count(&mut self) -> u32 {
        let mut lines_to_clear = Vec::new();

        for y in 0..TETRIS_GRID_HEIGHT {
            if self.grid[y].iter().all(|cell| cell.is_some()) {
                lines_to_clear.push(y);
            }
        }

        if lines_to_clear.is_empty() {
            return 0;
        }

        // Remove completed lines
        for &y in lines_to_clear.iter() {
            self.grid.remove(y);
            self.grid.insert(0, vec![None; TETRIS_GRID_WIDTH]);
        }

        // Update stats
        let lines_count = lines_to_clear.len() as u32;
        self.lines_cleared += lines_count;

        // Regular line clear scoring (only if not T-Spin, which is handled in lock_piece)
        let base_score = match lines_count {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
        self.score += base_score * (self.level + 1);

        // Level progression depends on mode
        match self.mode {
            TetrisMode::Zen => {
                // Zen: no level progression
            }
            TetrisMode::Sprint => {
                // Sprint: check if target reached
                if self.lines_cleared >= self.target_lines {
                    self.game_over = true;
                }
            }
            TetrisMode::Survival => {
                // Survival: level up every 5 lines for faster progression
                let new_level = self.lines_cleared / 5;
                if new_level > self.level {
                    self.level = new_level;
                    self.update_fall_speed();
                }
            }
            TetrisMode::Normal | TetrisMode::Dig => {
                // Normal/Dig: level up every 10 lines
                let new_level = self.lines_cleared / 10;
                if new_level > self.level {
                    self.level = new_level;
                    self.update_fall_speed();
                }
            }
        }

        lines_count
    }

    fn update_fall_speed(&mut self) {
        match self.mode {
            TetrisMode::Zen => {
                // Zen mode: speed never increases
                self.fall_interval = TETRIS_BASE_FALL_INTERVAL;
            }
            TetrisMode::Survival => {
                // Survival: faster progression
                // More aggressive formula
                self.fall_interval = (TETRIS_BASE_FALL_INTERVAL * 0.85_f32.powi(self.level as i32))
                    .max(TETRIS_MIN_FALL_INTERVAL);
            }
            _ => {
                // Normal, Sprint, Dig: standard progression
                // Formula: base_interval * (0.9 ^ level), clamped to minimum
                self.fall_interval = (TETRIS_BASE_FALL_INTERVAL * 0.9_f32.powi(self.level as i32))
                    .max(TETRIS_MIN_FALL_INTERVAL);
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.game_over {
            return;
        }

        // Track elapsed time for Sprint mode
        if self.mode == TetrisMode::Sprint {
            self.elapsed_time += dt;
        }

        // Check if piece is grounded
        let is_grounded = self.is_grounded();

        if is_grounded {
            // Activate lock delay if not already active
            if !self.lock_delay_active {
                self.lock_delay_active = true;
                self.lock_delay_timer = 0.0;
            }

            // Update lock delay timer
            self.lock_delay_timer += dt;

            // Lock piece if timer exceeds max or max resets reached
            if self.lock_delay_timer >= self.lock_delay_max
                || self.lock_delay_resets >= self.lock_delay_max_resets
            {
                self.lock_piece();
                return;
            }
        } else {
            // Reset lock delay if piece is no longer grounded
            self.lock_delay_active = false;
            self.lock_delay_timer = 0.0;
            self.lock_delay_resets = 0;
        }

        // Normal gravity fall
        self.fall_timer += dt;
        if self.fall_timer >= self.fall_interval {
            self.fall_timer = 0.0;

            // Try to move piece down
            if !self.move_piece(0, 1) {
                // Piece is now grounded, lock delay will handle it
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.game_over
    }

    /// Calculate ghost piece position (where piece would land if hard dropped)
    pub fn get_ghost_position(&self) -> Option<Vec<(i32, i32)>> {
        if let Some(piece) = &self.current_piece {
            let mut ghost_piece = piece.clone();

            // Drop until collision
            while !self.check_collision_for_piece(&ghost_piece, 0, 1) {
                ghost_piece.y += 1;
            }

            Some(ghost_piece.blocks())
        } else {
            None
        }
    }

    fn check_collision_for_piece(&self, piece: &Piece, dx: i32, dy: i32) -> bool {
        let mut test_piece = piece.clone();
        test_piece.x += dx;
        test_piece.y += dy;
        self.check_collision(&test_piece)
    }
}
