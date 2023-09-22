use crate::{pos::BoardPos, piece::PieceType, game::FenParseError};

const BOARD_SIZE: usize = 8;

pub struct Board {
    data: [[Option<Tile>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn empty() -> Self {
        Self {
            data: [[None; BOARD_SIZE]; BOARD_SIZE]
        }
    }

    /// Get the tile at the position, or `None` if there is no tile there.
    pub fn get_tile(&self, pos: &BoardPos) -> Option<Tile> {
        self.data[pos.rank() as usize][pos.file() as usize]
    }

    /// Set the tile at the position.
    pub fn set_tile(&mut self, pos: &BoardPos, tile: Tile) {
        self.data[pos.rank() as usize][pos.file() as usize] = Some(tile);
    }

    /// Remove the tile at the position.
    pub fn remove_tile(&mut self, pos: &BoardPos) -> Option<Tile> {
        let existing = self.data[pos.rank() as usize][pos.file() as usize];
        self.data[pos.rank() as usize][pos.file() as usize] = None;
        return existing;
    }

    /// If the `tile` parameter is `Some`, the tile is set, otherwise, the tile at
    /// the position is removed.
    pub fn set_or_remove_tile(&mut self, pos: &BoardPos, tile: Option<Tile>) {
        self.data[pos.rank() as usize][pos.file() as usize] = tile;
    }

    /// Create a `Board` instance from FEN placement data.
    /// 
    /// Note that the string should not be the entire FEN string, but should only be
    /// the first part of the FEN data, the part known as the "placement data".
    pub fn from_fen_placement_data(fen: &str) -> Result<Self, FenParseError> {
        let mut board = Board::empty();

        let mut file = 0;
        let mut rank = 7;
        for char in fen.chars() {
            if let Some(skip) = char.to_digit(10) {
                if skip > 8 || file > 8 {
                    return Err(FenParseError::LargeSkip);
                }
                file += skip as u8;
            } else if char == '/' {
                file = 0;
                rank -= 1;
            } else {
                let lowercase = char.to_ascii_lowercase();
                let is_lowercase = char == lowercase;
                let piece = match PieceType::from_char(lowercase) {
                    Ok(piece_type) => piece_type,
                    Err(_) => return Err(FenParseError::InvalidPiece(lowercase)),
                };
                let color = if is_lowercase { Color::Black } else { Color::White };
                let tile = Tile::new(piece, color);
                if file > 7 || rank > 7 {
                    return Err(FenParseError::OutsideBoard(file, rank));
                }
                let pos = BoardPos::new(file, rank);
                board.set_tile(&pos, tile);
                file += 1;
            }
        }

        Ok(board)
    }


    /// Export the `Board` to the FEN placement data.
    /// 
    /// Note that the string is not be the entire FEN string, but only the first
    /// part of the FEN data, the part known as the "placement data".
    pub fn to_fen_placement_data(&self) -> String {
        let mut str = String::new();
        for rank in (0..8_u8).rev() {
            let mut empty_count = 0;
            for file in 0_u8..8 {
                let tile = self.get_tile(&BoardPos::new(file, rank));
                match tile {
                    None => {
                        empty_count += 1;
                    }
                    Some(tile) => {
                        if empty_count > 0 {
                            str.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
                        let mut char = tile.piece.char();
                        if tile.color() == Color::White {
                            char = char.to_ascii_uppercase();
                        }
                        str.push(char);
                    }
                }
            }
            if empty_count > 0 {
                str.push_str(&empty_count.to_string());
            }
            str.push('/');
        }
        str.pop(); // Remove trailing /
        str
    }
}

/// A tile on the chess board, for example a black king or a white knight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tile {
    piece: PieceType,
    color: Color,
}

impl Tile {
    pub fn new(piece: PieceType, color: Color) -> Tile {
        Tile { piece, color }
    }

    pub fn piece(&self) -> PieceType { self.piece }
    pub fn color(&self) -> Color { self.color }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
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

    #[test]
    fn from_to_fen_placement_data() {
        const FEN_PLACEMENT_DATA: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let board = Board::from_fen_placement_data(FEN_PLACEMENT_DATA).unwrap();
        
        assert_eq!(FEN_PLACEMENT_DATA, board.to_fen_placement_data());
    }
}