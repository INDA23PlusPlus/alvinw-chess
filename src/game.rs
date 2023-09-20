use crate::{board::{Board, Color}, pos::BoardPos, piece::PieceType};

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

    fn is_check(&self, color: &Color) -> bool {

        let king_pos = self.get_king_pos(color);
        let king_pos = match king_pos {
            Some(king_pos) => king_pos,
            None => return false // Assume not check if there is no king.
        };

        for file in 0..8 {
            for rank in 0..8 {
                let pos = BoardPos::new(file, rank);
                let tile = self.board.get_tile(&pos);
                let tile = match tile {
                    Some(tile) => tile,
                    None => continue,
                };
                if tile.color() == *color {
                    // Friendly pieces are not a threat to their own king
                    continue;
                }
                let enemy_moves = self.get_pseudo_legal_moves(&pos);

                if enemy_moves.contains(&king_pos) {
                    // An enemy can capture the king on the next move, the color
                    // is in a state of chess.
                    return true;
                }
            }
        }

        // Not chess.
        return false;
    }

    /// Get the position of the king of the specified color.
    /// 
    /// Returns `None` if there is no king.
    fn get_king_pos(&self, color: &Color) -> Option<BoardPos> {
        for file in 0..8 {
            for rank in 0..8 {
                let pos = BoardPos::new(file, rank);
                let tile = self.board.get_tile(&pos);
                if let Some(tile) = tile {
                    if tile.piece() == PieceType::King && tile.color() == *color {
                        return Some(pos);
                    }
                }
            }
        }
        return None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_not_check() {
        let game = Game::new();
        // A brand new game should not be in a state of check.
        assert!(!game.is_check(&Color::White));
        assert!(!game.is_check(&Color::Black));
    }

    #[test]
    fn game_in_check() {
        let game = Game::from_fen("rnbqkbnr/pppp2pp/6Q1/4pp2/8/4P3/PPPP1PPP/RNB1KBNR b KQkq - 0 1").unwrap();
        assert!(!game.is_check(&Color::White));
        assert!(game.is_check(&Color::Black));
    }
}