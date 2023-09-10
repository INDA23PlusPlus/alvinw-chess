use crate::{pos::BoardPos, piece::PieceType};

const BOARD_SIZE: usize = 8;

pub struct Board {
    data: [[Option<Tile>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn empty() -> Board {
        Board {
            data: [[None; BOARD_SIZE]; BOARD_SIZE]
        }
    }

    pub fn get_tile(&self, pos: &BoardPos) -> Option<Tile> {
        self.data[pos.rank() as usize][pos.file() as usize]
    }

    pub fn set_tile(&mut self, pos: &BoardPos, tile: Tile) {
        self.data[pos.rank() as usize][pos.file() as usize] = Some(tile);
    }
}

/// A tile on the chess board, for example a black king or a white knight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tile {
    piece: PieceType,
    color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_board() {
        let board = Board::empty();

        let pos = "b2".parse().unwrap();
        assert!(board.get_tile(&pos).is_none());
    }

    #[test]
    fn set_get_tiles() {
        let mut board = Board::empty();

        let pos = "b2".parse().unwrap();
        let tile1 = Tile { piece: PieceType::King, color: Color::White };

        board.set_tile(&pos, tile1.clone());

        let tile2 = board.get_tile(&pos);

        assert_eq!(tile1, tile2.unwrap());
    }
}