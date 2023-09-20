use crate::{board::Color, pos::BoardPos, piece::PieceType};

use super::Game;

impl Game {

    pub(super) fn is_check(&self, color: &Color) -> bool {

        let king_pos = self.get_king_pos(color);
        let king_pos = match king_pos {
            Some(king_pos) => king_pos,
            None => return false // Assume not check if there is no king.
        };

        let enemy_color = color.opposite();

        return self.is_attacked_by(&king_pos, &enemy_color);
    }

    pub(super) fn is_attacked_by(&self, pos: &BoardPos, color: &Color) -> bool {
        for file in 0..8 {
            for rank in 0..8 {
                let enemy_pos = BoardPos::new(file, rank);
                let tile = self.board.get_tile(&enemy_pos);
                let tile = match tile {
                    Some(tile) => tile,
                    None => continue,
                };
                if tile.color() != *color {
                    // Only enemy pieces can attack.
                    continue;
                }
                let enemy_moves = self.get_pseudo_legal_moves(&enemy_pos, false);

                if enemy_moves.contains(pos) {
                    return true;
                }
            }
        }

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

    fn is_checkmate(&mut self, color: &Color) -> bool {
        if !self.is_check(color) {
            return false;
        }
        for file in 0..8 {
            for rank in 0..8 {
                let pos = BoardPos::new(file, rank);
                let tile = self.board.get_tile(&pos);
                if let Some(tile) = tile {
                    if tile.color() == *color {
                        // A friendly piece that can possibly move to stop the state of check.
                        
                        // Get all possible moves for this piece.
                        let moves = self.get_pseudo_legal_moves(&pos, false);
                        for move_pos in moves {
                            // Attempt each move
                            let old_tile = self.board.get_tile(&move_pos);

                            self.board.set_tile(&move_pos, tile);
                            self.board.remove_tile(&pos);

                            let check = self.is_check(color);

                            // Undo the move
                            self.board.set_or_remove_tile(&move_pos, old_tile);
                            self.board.set_tile(&pos, tile);

                            if !check {
                                // We found a possible move that resulted in a state that isn't check!
                                // That means it is not checkmate, only check.
                                return false;
                            }
                        }
                    }
                }
            }
        }
        // None of the possible moves that were attempted resulted in it no longer being
        // check, so there is nothing the team can do. It is checkmate.
        return true;
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

    #[test]
    fn checkmate1() {
        let mut game = Game::from_fen("8/8/8/5K1k/8/8/8/7R w - - 0 1").unwrap();
        assert!(!game.is_checkmate(&Color::White));
        assert!(game.is_checkmate(&Color::Black));
    }

    #[test]
    fn checkmate2() {
        // D. Byrne vs. Fischer
        let mut game = Game::from_fen("1Q6/5pk1/2p3p1/1p2N2p/1b5P/1bn5/2r3P1/2K5 b - - 0 1").unwrap();
        assert!(game.is_checkmate(&Color::White));
        assert!(!game.is_checkmate(&Color::Black));
    }
}