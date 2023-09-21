use crate::{board::{Board, Color}, pos::ParseBoardPosError};

use super::{Game, CastlingAvailability};

#[derive(Debug)]
pub enum FenParseError<'a> {
    LargeSkip,
    OutsideBoard(u8, u8),
    InvalidPiece(char),
    TooShort,
    InvalidTurn(&'a str),
    InvalidEnPassantTarget(ParseBoardPosError),
}

impl Game {

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
        
        let castling_availability = iter.next().ok_or(FenParseError::TooShort)?;

        let white_castling = CastlingAvailability {
            kingside: castling_availability.contains('K'),
            queenside: castling_availability.contains('Q'),
        };
        let black_castling = CastlingAvailability {
            kingside: castling_availability.contains('k'),
            queenside: castling_availability.contains('q'),
        };

        let en_passant = iter.next().ok_or(FenParseError::TooShort)?;

        let en_passant_target = if en_passant == "-" {
            None
        } else {
            Some(
                en_passant.parse()
                    .map_err(|err| FenParseError::InvalidEnPassantTarget(err))?
            )
        };

        let _halfmove_clock = iter.next().ok_or(FenParseError::TooShort)?;
        let _fullmove_number = iter.next().ok_or(FenParseError::TooShort)?;

        Ok(Self { board, current_turn, white_castling, black_castling, en_passant_target })
    }

    pub fn to_fen(&self) -> String {
        let mut str = String::new();
        str.push_str(&self.board.to_fen_placement_data());
        str.push(' ');
        str.push(if self.current_turn == Color::White { 'w' } else { 'b' });
        str.push(' ');
        let len1 = str.len();
        if self.white_castling.kingside { str.push('K') }
        if self.white_castling.queenside { str.push('Q') }
        if self.black_castling.kingside { str.push('k') }
        if self.black_castling.queenside { str.push('q') }
        if str.len() == len1 {
            // No castling
            str.push('-');
        }
        str.push(' ');
        if let Some(en_passant_target) = &self.en_passant_target {
            str.push_str(&en_passant_target.to_string());
        } else {
            str.push('-');
        }
        str.push(' ');
        str.push_str("0 0"); // TODO clocks
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