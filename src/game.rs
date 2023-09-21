use crate::{board::{Board, Color}, pos::BoardPos};

mod fen;
pub use fen::FenParseError;

mod movement;
pub use movement::{MovePieceError, GetMovesetError};

mod check;

/// The FEN for the starting position of the game.
const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct Game {
    board: Board,
    current_turn: Color,
    white_castling: CastlingAvailability,
    black_castling: CastlingAvailability,
    en_passant_target: Option<BoardPos>,
}

impl Game {
    /// Create a new standard game of chess with the default starting position.
    pub fn new() -> Self {
        Self::from_fen(STARTING_POSITION_FEN).expect("Hardcoded FEN is valid.")
    }

    /// Get the underlying `Board` instance for this game.
    /// 
    /// It is not recomended that users of this library use this method, but it
    /// exists if low-level access and modification to the board is required.
    pub fn board(&mut self) -> &mut Board { &mut self.board }
}

struct CastlingAvailability {
    pub kingside: bool,
    pub queenside: bool,
}