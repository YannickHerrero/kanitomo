use rand::Rng;

// ============================================================================
// 2048 Game
// ============================================================================

const GAME_2048_SIZE: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Game2048Move {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Game2048 {
    pub board: [[u32; GAME_2048_SIZE]; GAME_2048_SIZE],
    pub score: u32,
    pub game_over: bool,
    rng: rand::rngs::ThreadRng,
}

impl Game2048 {
    pub fn new() -> Self {
        let mut game = Self {
            board: [[0; GAME_2048_SIZE]; GAME_2048_SIZE],
            score: 0,
            game_over: false,
            rng: rand::thread_rng(),
        };
        game.add_random_tile();
        game.add_random_tile();
        game
    }

    pub fn reset(&mut self) {
        self.board = [[0; GAME_2048_SIZE]; GAME_2048_SIZE];
        self.score = 0;
        self.game_over = false;
        self.add_random_tile();
        self.add_random_tile();
    }

    pub fn is_finished(&self) -> bool {
        self.game_over
    }

    pub fn max_tile(&self) -> u32 {
        self.board
            .iter()
            .flat_map(|row| row.iter())
            .copied()
            .max()
            .unwrap_or(0)
    }

    pub fn make_move(&mut self, direction: Game2048Move) -> bool {
        if self.game_over {
            return false;
        }

        let mut moved = false;
        let mut gained = 0u32;

        match direction {
            Game2048Move::Left => {
                for row in 0..GAME_2048_SIZE {
                    let (next, row_moved, row_gained) = slide_line(self.board[row]);
                    if row_moved {
                        self.board[row] = next;
                        moved = true;
                    }
                    gained += row_gained;
                }
            }
            Game2048Move::Right => {
                for row in 0..GAME_2048_SIZE {
                    let mut reversed = self.board[row];
                    reversed.reverse();
                    let (mut next, row_moved, row_gained) = slide_line(reversed);
                    next.reverse();
                    if row_moved {
                        self.board[row] = next;
                        moved = true;
                    }
                    gained += row_gained;
                }
            }
            Game2048Move::Up => {
                for col in 0..GAME_2048_SIZE {
                    let column = self.get_column(col);
                    let (next, col_moved, col_gained) = slide_line(column);
                    if col_moved {
                        self.set_column(col, next);
                        moved = true;
                    }
                    gained += col_gained;
                }
            }
            Game2048Move::Down => {
                for col in 0..GAME_2048_SIZE {
                    let mut column = self.get_column(col);
                    column.reverse();
                    let (mut next, col_moved, col_gained) = slide_line(column);
                    next.reverse();
                    if col_moved {
                        self.set_column(col, next);
                        moved = true;
                    }
                    gained += col_gained;
                }
            }
        }

        if moved {
            self.score = self.score.saturating_add(gained);
            self.add_random_tile();
            if !self.has_moves() {
                self.game_over = true;
            }
        } else if !self.has_moves() {
            self.game_over = true;
        }

        moved
    }

    fn get_column(&self, col: usize) -> [u32; GAME_2048_SIZE] {
        let mut column = [0u32; GAME_2048_SIZE];
        for row in 0..GAME_2048_SIZE {
            column[row] = self.board[row][col];
        }
        column
    }

    fn set_column(&mut self, col: usize, values: [u32; GAME_2048_SIZE]) {
        for row in 0..GAME_2048_SIZE {
            self.board[row][col] = values[row];
        }
    }

    fn add_random_tile(&mut self) {
        let mut empty_positions = Vec::new();
        for row in 0..GAME_2048_SIZE {
            for col in 0..GAME_2048_SIZE {
                if self.board[row][col] == 0 {
                    empty_positions.push((row, col));
                }
            }
        }

        if empty_positions.is_empty() {
            return;
        }

        let index = self.rng.gen_range(0..empty_positions.len());
        let (row, col) = empty_positions[index];
        let value = if self.rng.gen_range(0..10) == 0 { 4 } else { 2 };
        self.board[row][col] = value;
    }

    fn has_moves(&self) -> bool {
        if self
            .board
            .iter()
            .flat_map(|row| row.iter())
            .any(|&cell| cell == 0)
        {
            return true;
        }

        for row in 0..GAME_2048_SIZE {
            for col in 0..GAME_2048_SIZE {
                let value = self.board[row][col];
                if row + 1 < GAME_2048_SIZE && self.board[row + 1][col] == value {
                    return true;
                }
                if col + 1 < GAME_2048_SIZE && self.board[row][col + 1] == value {
                    return true;
                }
            }
        }

        false
    }
}

fn slide_line(mut line: [u32; GAME_2048_SIZE]) -> ([u32; GAME_2048_SIZE], bool, u32) {
    let original = line;
    let mut compacted = Vec::with_capacity(GAME_2048_SIZE);
    for value in line.iter().copied().filter(|v| *v != 0) {
        compacted.push(value);
    }

    let mut merged = Vec::with_capacity(GAME_2048_SIZE);
    let mut gained = 0u32;
    let mut index = 0;
    while index < compacted.len() {
        if index + 1 < compacted.len() && compacted[index] == compacted[index + 1] {
            let next = compacted[index] * 2;
            merged.push(next);
            gained = gained.saturating_add(next);
            index += 2;
        } else {
            merged.push(compacted[index]);
            index += 1;
        }
    }

    for slot in line.iter_mut() {
        *slot = 0;
    }
    for (i, value) in merged.into_iter().enumerate() {
        if i < GAME_2048_SIZE {
            line[i] = value;
        }
    }

    (line, line != original, gained)
}
