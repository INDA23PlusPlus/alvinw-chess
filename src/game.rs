use std::collections::HashSet;

use crate::{board::{Board, Color}, pos::BoardPos, piece::PieceType};

pub struct Game {
    board: Board,
    current_turn: Color,
}

/// Errors returned from Game's `get_moveset`.
#[derive(Debug)]
pub enum GetMovesetError {
    /// There was no piece at the position where the call to `get_moveset` was made.
    NoTile,
    /// The piece was of the opposing color.
    /// 
    /// This error is returned when `get_moveset` is called on a piece that is of the
    /// color that is not the current turn. TODO reformat sentence
    NotCurrentTurn,
}

impl Game {

    pub fn get_moveset(&self, pos: &BoardPos) -> Result<HashSet<BoardPos>, GetMovesetError> {
        let tile = self.board.get_tile(pos)
            .ok_or(GetMovesetError::NoTile)?;

        if tile.color() != self.current_turn {
            return Err(GetMovesetError::NotCurrentTurn);
        }

        let mut moveset = HashSet::new();

        match tile.piece() {
            PieceType::King => {
                self.try_moves_once(&mut moveset, &pos, [
                    (-1,  1), (0,  1), (1,  1),
                    (-1,  0), /******/ (1,  0),
                    (-1, -1), (0, -1), (1, -1),
                ]);
            }
            PieceType::Queen => {
                self.try_moves_multiple(&mut moveset, &pos, [
                    (-1,  1), (0,  1), (1,  1),
                    (-1,  0), /******/ (1,  0),
                    (-1, -1), (0, -1), (1, -1),
                ]);
            },
            PieceType::Rook => {
                self.try_moves_multiple(&mut moveset, &pos, [
                              (0,  1),
                    (-1,  0), /******/ (1,  0),
                              (0, -1),
                ]);
            },
            PieceType::Bishop => {
                self.try_moves_multiple(&mut moveset, &pos, [
                    (-1,  1), (1,  1),
                    (-1, -1), (1, -1),
                ]);
            },
            PieceType::Knight => {
                self.try_moves_once(&mut moveset, &pos, [
                    (-1,  2), (1,   2),
                    (2,   1), (2,  -1),
                    (-1, -2), (1,  -2),
                    (-2,  1), (-2, -1),
                ]);
            },
            PieceType::Pawn => todo!(),
        }

        Ok(moveset)
    }

    fn try_moves(&self,
        moveset: &mut HashSet<BoardPos>,
        moves: Vec<Option<BoardPos>>
    ) {
        for option_move in moves {
            if let Some(m) = option_move {
                moveset.insert(m);
            }
        }
    }

    fn try_moves_once<const COUNT: usize>(&self,
        moveset: &mut HashSet<BoardPos>,
        start: &BoardPos,
        delta_positions: [(i8, i8); COUNT]
    ) {
        let moves = delta_positions.map(|(delta_file, delta_rank)| {
            self.try_move_once(&start, delta_file, delta_rank)
        });
        self.try_moves(moveset, Vec::from(moves));
    }

    fn try_move_once(&self,
        start: &BoardPos,
        delta_file: i8,
        delta_rank: i8
    ) -> Option<BoardPos> {
        let pos = start.offset(delta_file, delta_rank);
        let pos = match pos {
            Some(pos) => pos,
            None => return None,
        };

        let tile = self.board.get_tile(&pos);
        let tile = match tile {
            Some(tile) => tile,
            None => return Some(pos), // No tile at the position means we can move there
        };

        if tile.color() == self.current_turn {
            // A friendly piece is in the way.
            return None;
        }

        // Taking enemy pieces is fine
        Some(pos)
    }

    fn try_moves_multiple<const COUNT: usize>(&self,
        moveset: &mut HashSet<BoardPos>,
        start: &BoardPos,
        vectors: [(i8, i8); COUNT]
    ) {
        for (delta_file, delta_rank) in vectors {
            self.try_move_multiple(moveset, start, delta_file, delta_rank);
        }
    }

    fn try_move_multiple(&self,
        moveset: &mut HashSet<BoardPos>,
        start: &BoardPos,
        delta_file: i8,
        delta_rank: i8
    ) {
        let mut pos = (*start).clone();
        loop {
            let new_pos = self.try_move_once(&pos, delta_file, delta_rank);
            pos = match new_pos {
                None => break,
                Some(new_pos) => new_pos,
            };
            moveset.insert(pos.clone());
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{board::Tile, piece::PieceType};
    use super::*;

    // Prepare a game for a moveset test.
    fn prepare_moveset_test(piece: PieceType) -> (Game, BoardPos) {
        const COLOR: Color = Color::White;

        let mut board = Board::empty();
        let pos = "e4".parse().unwrap();
        let tile = Tile::new(piece, COLOR);
        board.set_tile(&pos, tile);

        (Game {
            board,
            current_turn: COLOR
        }, pos)
    }

    // Format a set of board positions by sorting them and presenting their
    /// human-readable format. This is a great way to compare two movesets.
    fn format_positions(set: &HashSet<BoardPos>) -> String {
        let mut arr = set.iter()
            .map(|pos| pos.to_string())
            .collect::<Vec<String>>();
        arr.sort();
        arr.join(" ")
    }

    /// Assert that the expected moves exist in the move set. There may be other
    /// moves in the actual moveset.
    fn assert_moves_exist(actual: &HashSet<BoardPos>, expected: &str) {
        let mut missing = HashSet::new();
        for str in expected.split_whitespace() {
            let pos = str.parse().unwrap();
            if !actual.contains(&pos) {
                missing.insert(pos);
            }
        }
        assert!(missing.is_empty(), "Expected {} to be valid moves.", format_positions(&missing));
    }

    /// Assert that the moves dont exist in the move set.
    fn assert_moves_dont_exist(actual: &HashSet<BoardPos>, unexpected: &str) {
        let mut existing = HashSet::new();
        for str in unexpected.split_whitespace() {
            let pos = str.parse().unwrap();
            if actual.contains(&pos) {
                existing.insert(pos);
            }
        }
        assert!(existing.is_empty(), "Expected {} to be invalid moves.", format_positions(&existing));
    }

    /// Assert that the moveset matches exactly the specified moves and no
    /// other moves.
    fn assert_moves(actual: &HashSet<BoardPos>, expected: &str) {
        let expected_set: HashSet<BoardPos> = expected
            .split_whitespace()
            .map(|str| str.parse().unwrap())
            .collect();
        assert_eq!(actual, &expected_set,
            "\n\nexpected {}\n   found {}\n",
            format_positions(&expected_set),
            format_positions(actual),
        );
    }

    #[test]
    fn king_moves() {
        let (game, pos) = prepare_moveset_test(PieceType::King);
        let actual = game.get_moveset(&pos).unwrap();
        assert_moves(&actual, "d5 e5 f5 d4 f4 d3 e3 f3");
    }

    #[test]
    fn queen_moves() {
        let (game, pos) = prepare_moveset_test(PieceType::Queen);
        let actual = game.get_moveset(&pos).unwrap();
        assert_moves_exist(&actual, "f5 f4 h7 b4");
        assert_moves_dont_exist(&actual, "e4 d2 f6");
    }

    #[test]
    fn rook_moves() {
        let (game, pos) = prepare_moveset_test(PieceType::Rook);
        let actual = game.get_moveset(&pos).unwrap();
        assert_moves_exist(&actual, "d4 e5 e7 h4 e3");
        assert_moves_dont_exist(&actual, "e4 f5 d2 d7");
    }

    #[test]
    fn knight_moves() {
        let (game, pos) = prepare_moveset_test(PieceType::Knight);
        let actual = game.get_moveset(&pos).unwrap();
        assert_moves(&actual, "d6 f6 g5 g3 f2 d2 c3 c5");
    }
}