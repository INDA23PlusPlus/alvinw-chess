use crate::board::{Board, Color};

mod fen;
pub use fen::FenParseError;

mod movement;
pub use movement::{MovePieceError, GetMovesetError};

/// The FEN for the starting position of the game.
const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct Game {
    board: Board,
    current_turn: Color,
}

impl Game {
    pub fn new() -> Self {
        Self::from_fen(STARTING_POSITION_FEN).expect("Hardcoded FEN is valid.")
    }
}