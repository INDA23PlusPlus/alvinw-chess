use crate::{board::{Board, Color, Tile}, pos::BoardPos, piece::PieceType};

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
    promotion_required: Option<BoardPos>,
    halfmove_clock: u32,
    fullmove_number: u32,
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

    /// Get the `Color` of the team that is next to make a move.
    pub fn current_turn(&self) -> Color {
        self.current_turn
    }

    /// Get the tile at the specified position, or `None` if the square is empty.
    pub fn get_tile(&self, pos: &BoardPos) -> Option<Tile> {
        self.board.get_tile(pos)
    }

    /// Get the current game state. This method must be called after each move.
    /// 
    /// See the `GameState` enum for the possible values.
    /// 
    /// In case this method returns `PromotionRequired` the `promote` function must
    /// be called before the next move is performed.
    pub fn get_state(&mut self) -> GameState {
        if let Some(pos) = &self.promotion_required {
            return GameState::PromotionRequired(pos.clone());
        }

        if self.is_check(&Color::White) {
            if self.is_checkmate(&Color::White) {
                return GameState::Checkmate(Color::White);
            } else {
                return GameState::Check(Color::White);
            }
        }

        if self.is_check(&Color::Black) {
            if self.is_checkmate(&Color::Black) {
                return GameState::Checkmate(Color::Black);
            } else {
                return GameState::Check(Color::Black);
            }
        }

        return GameState::Normal;
    }

    /// Promote a pawn.
    /// 
    /// Only use this method directly after calling `get_state` and having it return
    /// `PromotionRequired`. Incorrect usage of this method will result in a panic.
    /// 
    /// Pawns and kings are not valid piece types to this method.
    /// 
    /// ## Panics
    /// This method will panic if the piece type is a pawn or king. This method will
    /// also panic if it is called when there is no piece to be promoted
    pub fn promote(&mut self, piece_type: PieceType) {
        if piece_type == PieceType::Pawn {
            panic!("Promoting is required. Pawn is not a valid argument.");
        }
        if piece_type == PieceType::King {
            panic!("Promoting to kings is not allowed");
        }
        let pos = self.promotion_required.as_ref()
            .expect("Promoting is not possible right now.");

        let pawn = self.board.get_tile(pos)
            .expect("Promotion can not occur on empty squares.");

        let new_tile = Tile::new(piece_type, pawn.color());
        self.board.set_tile(pos, new_tile);

        self.promotion_required = None;
    }
}

struct CastlingAvailability {
    pub kingside: bool,
    pub queenside: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GameState {
    /// Normal gameplay.
    Normal,
    /// The king is under threat. The color represents the color of the team that
    /// is in check.
    Check(Color),
    /// The game is won. The color represents the team that has won.
    Checkmate(Color),
    /// The player is required to choose which piece to promote a pawn to at the
    /// specified location.
    PromotionRequired(BoardPos),
    // TODO draw?
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotion_test() {
        let mut game = Game::from_fen("4k3/2P5/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        game.move_piece(&"c7".parse().unwrap(), &"c8".parse().unwrap()).unwrap();

        let status = game.get_state();

        let pos = match status {
            GameState::PromotionRequired(pos) => pos,
            _ => panic!("Expected PromotionRequired"),
        };

        assert_eq!(pos, "c8".parse().unwrap());
        game.promote(PieceType::Queen);

        assert_eq!(game.get_state(), GameState::Check(Color::Black));
        game.move_piece(&"e8".parse().unwrap(), &"e7".parse().unwrap()).unwrap();
        assert_eq!(game.get_state(), GameState::Normal);
    }
}