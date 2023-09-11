use std::fmt;

/// A struct that represends valid positions on a chess board.
/// 
/// The file is a number between `[0-7]` (inclusive) where `0` represents the
/// file `a` and `7` represents the file `h`.
/// 
/// The rank is a number between `[0-7]` (inclusive) where `0` represents the
/// first rank, commonly displayed as rank 1 and where `7` is the last rank,
/// commonly displayed as rank 8.
/// 
/// ## Example
/// The position `b4` has a rank value `3` and a file value `1`.
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct BoardPos {
    file: u8,
    rank: u8,
}

impl BoardPos {
    /// Create a BoardPos instance.
    /// 
    /// ## Panics
    /// This function will panic if either the rank or file is outside of the
    /// inclusive range `[0, 7]`.
    pub fn new(file: u8, rank: u8) -> BoardPos {
        if file > 7 {
            panic!("file must be in the inclusive range [0-7], got {}", file);
        }
        if rank > 7 {
            panic!("rank must be in the inclusive range [0-7], got {}", rank);
        }
        BoardPos { file, rank }
    }

    /// Get the internal representation of the file as an integer between `[0-7]`.
    /// 
    /// This is not the human-readable file.
    pub fn file(&self) -> u8 {
        self.file
    }

    /// Get the lowercase character for the file represented by this board position.
    pub fn file_char(&self) -> char {
        // 0 -> a, 1 -> b, ..., 7 -> h
        ('a' as u8 + self.file as u8) as char
    }

    /// Get the internal representation of the rank as an integer between `[0-7]`.
    /// 
    /// This is not the human-readable rank.
    pub fn rank(&self) -> u8 {
        self.rank
    }

    pub fn offset(&self, delta_file: i8, delta_rank: i8) -> Option<BoardPos> {
        let file = self.file as i8 + delta_file;
        let rank = self.rank as i8 + delta_rank;
        if file < 0 || file > 7 || rank < 0 || rank > 7 {
            return None;
        }
        Some(BoardPos::new(file as u8, rank as u8))
    }

}

impl fmt::Display for BoardPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file_char(), self.rank + 1)
    }
}

#[derive(Debug)]
pub struct ParseBoardPosError {
    msg: &'static str,
}

impl ParseBoardPosError {
    pub fn msg(&self) -> &'static str {
        self.msg
    }
}

impl std::str::FromStr for BoardPos {
    type Err = ParseBoardPosError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let file_char = chars.next()
            .ok_or(ParseBoardPosError { msg: "String too short." })?;

        let rank = chars.next()
            .ok_or(ParseBoardPosError { msg: "String too short." })?;

        if !chars.next().is_none() {
            return Err(ParseBoardPosError { msg: "String too long." });
        }

        let rank = rank.to_digit(10)
            .ok_or(ParseBoardPosError { msg: "Second character must be a digit." })? as u8;

        let rank = rank - 1;

        let file = file_char as u8 - 'a' as u8;

        Ok(BoardPos { file, rank })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_pos() {
        let pos = BoardPos::new(7, 3);
        
        assert_eq!(pos.rank(), 3);
        assert_eq!(pos.file(), 7);
        assert_eq!(pos.file_char(), 'h');
    }

    #[test]
    #[should_panic]
    fn invalid_board_pos_file() {
        BoardPos::new(8, 2);
    }

    #[test]
    #[should_panic]
    fn invalid_board_pos_rank() {
        BoardPos::new(2, 8);
    }

    #[test]
    fn format_board_pos() {
        let pos = BoardPos::new(1, 3);

        assert_eq!(pos.to_string(), "b4");
    }

    #[test]
    fn parse_board_pos() {
        let pos1: BoardPos = "b4".parse().unwrap();
        let pos2 = BoardPos::new(1, 3);

        assert_eq!(pos1, pos2);
        assert_eq!(pos1.to_string(), "b4");
    }

    #[test]
    fn valid_offset() {
        let pos1 = BoardPos::new(1, 3);
        let pos2 = pos1.offset(1, -2).unwrap();

        assert_eq!(pos2.file(), 2);
        assert_eq!(pos2.rank(), 1);
    }

    #[test]
    fn invalid_offset() {
        let pos1 = BoardPos::new(1, 3);
        let pos2 = pos1.offset(-2, 10);

        assert!(pos2.is_none())
    }

}
