use crate::board::{Board, Color};

use super::Game;

#[derive(Debug)]
pub enum FenParseError<'a> {
    LargeSkip,
    OutsideBoard(u8, u8),
    InvalidPiece(char),
    TooShort,
    InvalidTurn(&'a str),
}

impl Game {

    // FEN I/O

    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let mut iter = fen.split_whitespace();
        
        let placement_data = iter.next().ok_or(FenParseError::TooShort)?;
        let board = Board::from_fen_placement_data(placement_data)?;

        let current_turn = iter.next().ok_or(FenParseError::TooShort)?;
        let current_turn = match current_turn {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(FenParseError::InvalidTurn(current_turn)),
        };
        
        let _castling_availability = iter.next().ok_or(FenParseError::TooShort)?;
        let _en_passant = iter.next().ok_or(FenParseError::TooShort)?;
        let _halfmove_clock = iter.next().ok_or(FenParseError::TooShort)?;
        let _fullmove_number = iter.next().ok_or(FenParseError::TooShort)?;

        Ok(Self { board, current_turn })
    }

    pub fn to_fen(&self) -> String {
        let mut str = String::new();
        str.push_str(&self.board.to_fen_placement_data());
        str.push(' ');
        str.push(if self.current_turn == Color::White { 'w' } else { 'b' });
        str.push_str(" - - 0 0"); // Castling, En passant TODO
        str
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game() {
        // Ensure FEN parsing of starting position doesn't panic
        Game::new();
    }
}