pub mod breakout;
pub mod crab_catch;
pub mod dash;
pub mod game_2048;
pub mod snake;
pub mod tetris;

pub use breakout::BreakoutGame;
#[allow(unused_imports)]
pub use breakout::Brick;
pub use crab_catch::CrabCatchGame;
#[allow(unused_imports)]
pub use crab_catch::{CrabFacing, FallingFood};
pub use dash::DashGame;
#[allow(unused_imports)]
pub use dash::DashObstacle;
pub use game_2048::{Game2048, Game2048Move};
pub use snake::{Direction, SnakeGame};
pub use tetris::{PieceType, TetrisGame, TetrisMode};
