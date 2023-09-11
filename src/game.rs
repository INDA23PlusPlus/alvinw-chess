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
            PieceType::Queen => todo!(),
            PieceType::Rook => todo!(),
            PieceType::Bishop => todo!(),
            PieceType::Knight => todo!(),
            PieceType::Pawn => todo!(),
        }

        Ok(moveset)
    }

    fn try_moves<const COUNT: usize>(&self,
        moveset: &mut HashSet<BoardPos>,
        moves: [Option<BoardPos>; COUNT]
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
        self.try_moves(moveset, moves);
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
        // TODO try_moves will need to be a vector (or maybe just iterable?) since
        // we have dynamic length now
        let moves = vectors.flat_map(|(delta_file, delta_rank)| {
            self.try_move_multiple(&start, delta_file, delta_rank);
        });
        self.try_moves(moveset, moves);
    }

    fn try_move_multiple(&self,
        start: &BoardPos,
        delta_file: i8,
        delta_rank: i8
    ) -> Vec<Option<BoardPos>> {
        todo!();
    }
}


#[cfg(test)]
mod tests {
    use crate::{board::Tile, piece::PieceType};

    use super::*;

    fn prepare_moveset_test(piece: PieceType) -> (Game, BoardPos) {
        const COLOR: Color = Color::White;

        let mut board = Board::empty();
        let pos = "c3".parse().unwrap();
        let tile = Tile::new(piece, COLOR);
        board.set_tile(&pos, tile);

        (Game {
            board,
            current_turn: COLOR
        }, pos)
    }

    #[test]
    fn king_moves() {
        let (game, pos) = prepare_moveset_test(PieceType::King);

        let actual = game.get_moveset(&pos).unwrap();
        
        let expected = HashSet::from([
            pos.offset(-1, 1).unwrap(), pos.offset(0, 1).unwrap(), pos.offset(1, 1).unwrap(),
            pos.offset(-1, 0).unwrap(), /* self, */ pos.offset(1, 0).unwrap(),
            pos.offset(-1, -1).unwrap(), pos.offset(0, -1).unwrap(), pos.offset(1, -1).unwrap(),
        ]);

        assert_eq!(expected, actual);
    }
}