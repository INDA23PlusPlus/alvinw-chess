use std::collections::HashSet;

use crate::{pos::BoardPos, board::Color, piece::PieceType};

use super::Game;

#[derive(Debug)]
pub enum MovePieceError {
    NoTile,
    NotCurrentTurn,
    InvalidMove,
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

#[derive(PartialEq)]
enum MoveType {
    /// The piece is being moved to an unoccupied, empty tile.
    ToEmpty,
    /// The piece is being moved to a tile where an enemy piece is standing, which
    /// will result in capturing the enemy piece.
    Attacking
}

impl Game {

    // Movement

    pub fn move_piece(&mut self, from: &BoardPos, to: &BoardPos) -> Result<(), MovePieceError> {
        let moveset = match self.get_legal_moves(from) {
            Ok(moveset) => moveset,
            Err(GetMovesetError::NoTile) => return Err(MovePieceError::NoTile),
            Err(GetMovesetError::NotCurrentTurn) => return Err(MovePieceError::NotCurrentTurn),
        };

        if !moveset.contains(to) {
            return Err(MovePieceError::InvalidMove);
        }

        let tile = self.board.remove_tile(from).expect("Move is already validated.");
        self.board.set_tile(to, tile);

        self.current_turn = self.current_turn.opposite();
        
        Ok(())
    }

    /// Get the legal moves for a piece.
    ///
    /// Legal moves are defined as moves that:
    /// 1. follow the movement rules for the piece. Eg. a bishop can only walk
    ///    diagonally.
    /// 2. respect the environment. Eg. not jumping over pieces unless the piece
    ///    allows that.
    /// 3. do not move outside of the board.
    /// 4. do not move into a state of check.
    /// 
    /// This method will ensure there is a tile at the position, otherwise the
    /// `NoTile` error variant is returned.
    /// 
    /// This method will ensure that it is the correct turn. In other words, if the
    /// current turn is white, only white piece's moves can be gotten with this
    /// method. Otherwise the `NotCurrentTurn` error variant is returned.
    /// 
    /// ## Castling and en passant
    /// Not implemented yet!
    pub fn get_legal_moves(&mut self, pos: &BoardPos) -> Result<HashSet<BoardPos>, GetMovesetError> {
        let tile = self.board.get_tile(pos)
            .ok_or(GetMovesetError::NoTile)?;

        if tile.color() != self.current_turn {
            return Err(GetMovesetError::NotCurrentTurn);
        }

        let mut moveset = self.get_pseudo_legal_moves(pos);
        moveset.retain(|move_pos| {
            // Ensure the move does not move into a state of check.
            // Attempt the move.

            // Save tile on square.
            let old_square = self.board.get_tile(move_pos);

            // Move there by setting the tiles directly.
            // TODO method here to respect en passant and castling.
            self.board.set_tile(move_pos, tile);
            self.board.remove_tile(pos);

            let check = self.is_check(&tile.color());
            // This move resulted in a state of check. It is not a legal move.

            // Undo the move.
            if let Some(old_square) = old_square {
                self.board.set_tile(move_pos, old_square);
            } else {
                self.board.remove_tile(move_pos);
            }
            self.board.set_tile(pos, tile);

            !check
        });
        Ok(moveset)
    }

    /// Get the pseudo legal moves for a tile.
    /// 
    /// Users of this library are recomended to use the `get_legal_moves` method
    /// instead.
    /// 
    /// Psuedo legal moves are concidered moves that:
    /// 1. follow the movement rules for the piece. Eg. a bishop can only walk
    ///    diagonally.
    /// 2. respect the environment. Eg. not jumping over pieces unless the piece
    ///    allows that.
    /// 3. do not move outside of the board.
    /// 
    /// Note that this method will not validate the turn of the piece and will not
    /// validate whether the piece can be moved into a state of check.
    /// 
    /// ## Panics
    /// This function will panic if there is no piece at the tile.
    pub fn get_pseudo_legal_moves(&self, pos: &BoardPos) -> HashSet<BoardPos> {
        let tile = self.board.get_tile(pos)
            .expect("Attempt to get pseudo-legal moves from empty tile.");

        let mut moveset = HashSet::new();

        match tile.piece() {
            PieceType::King => {
                self.try_moves_once(&mut moveset, &pos, &tile.color(), [
                    (-1,  1), (0,  1), (1,  1),
                    (-1,  0), /******/ (1,  0),
                    (-1, -1), (0, -1), (1, -1),
                ]);
            }
            PieceType::Queen => {
                self.try_moves_multiple(&mut moveset, &pos, &tile.color(), [
                    (-1,  1), (0,  1), (1,  1),
                    (-1,  0), /******/ (1,  0),
                    (-1, -1), (0, -1), (1, -1),
                ]);
            },
            PieceType::Rook => {
                self.try_moves_multiple(&mut moveset, &pos, &tile.color(), [
                              (0,  1),
                    (-1,  0), /******/ (1,  0),
                              (0, -1),
                ]);
            },
            PieceType::Bishop => {
                self.try_moves_multiple(&mut moveset, &pos, &tile.color(), [
                    (-1,  1), (1,  1),
                    (-1, -1), (1, -1),
                ]);
            },
            PieceType::Knight => {
                self.try_moves_once(&mut moveset, &pos, &tile.color(), [
                    (-1,  2), (1,   2),
                    (2,   1), (2,  -1),
                    (-1, -2), (1,  -2),
                    (-2,  1), (-2, -1),
                ]);
            },
            PieceType::Pawn => {
                // Calculate the forward direction for this team.
                let dir: i8 = if tile.color() == Color::White { 1 } else { -1 };

                // Note that ranks are specified in their internal form, aka the 0-indexed index.
                let first_rank = if tile.color() == Color::White { 1 } else { 6 };

                // Since pawns can never move backwards, we can be sure that it is the pawn's
                // first move if it is located at the starting rank for pawns.
                let is_first_move = pos.rank() == first_rank;

                // Moving forward one tile is always possible (assuming it is valid in other
                // regards).
                self.try_moves_once(&mut moveset, &pos, &tile.color(), [
                    (0, dir)
                ]);

                // Moving two tiles is only possible if it is the pawn's first move and...
                if is_first_move {
                    let pos_one_forward = pos.offset(0, dir);
                    let piece_one_forward = pos_one_forward.and_then(
                        |pos_one_forward| self.board.get_tile(&pos_one_forward)
                    );

                    // ...there is no piece, regardless of color, one tile forward.
                    if piece_one_forward.is_none() {
                        self.try_moves_once(&mut moveset, &pos, &tile.color(), [
                            (0, 2 * dir)
                        ]);
                    }
                }

                // Diagonal moves are only possible when attacking.
                self.try_attacking_move(&mut moveset, &pos, &tile.color(), -1, dir);
                self.try_attacking_move(&mut moveset, &pos, &tile.color(), 1, dir);
            },
        }

        moveset
    }

    /// Test the specified delta position moves and add the valid moves to the
    /// moveset.
    fn try_moves_once<const COUNT: usize>(&self,
        moveset: &mut HashSet<BoardPos>,
        start: &BoardPos,
        friendly_color: &Color,
        delta_positions: [(i8, i8); COUNT]
    ) {
        for (delta_file, delta_rank) in delta_positions {
            let option_move = self.try_move_once(&start, delta_file, delta_rank, friendly_color);

            if let Some((pos, _move_type)) = option_move {
                moveset.insert(pos);
            }
        }
    }

    /// Test a move, and if it is a valid move, return it.
    fn try_move_once(&self,
        start: &BoardPos,
        delta_file: i8,
        delta_rank: i8,
        friendly_color: &Color
    ) -> Option<(BoardPos, MoveType)> {
        let pos = start.offset(delta_file, delta_rank);
        let pos = match pos {
            Some(pos) => pos,
            None => return None,
        };

        let tile = self.board.get_tile(&pos);
        let tile = match tile {
            Some(tile) => tile,
            None => return Some((pos, MoveType::ToEmpty)), // No tile at the position means we can move there
        };

        if tile.color() == *friendly_color {
            // A friendly piece is in the way.
            return None;
        }

        // Capturing enemy pieces is fine
        Some((pos, MoveType::Attacking))
    }

    /// Test an attacking move, and if it is a valid move, return it.
    ///
    /// Attacking moves are only valid if it involves capturing an enemy piece.
    fn try_attacking_move(&self,
        moveset: &mut HashSet<BoardPos>,
        start: &BoardPos,
        friendly_color: &Color,
        delta_file: i8,
        delta_rank: i8,
    ) {
        let option_move = self.try_move_once(start, delta_file, delta_rank, friendly_color);
        if let Some((pos, move_type)) = option_move {
            match move_type {
                MoveType::ToEmpty => {},
                MoveType::Attacking => {
                    // Only attacking moves are valid.
                    moveset.insert(pos);
                }
            }
        }
    }

    /// Test the specified direction and add all possible moves to the moveset.
    ///
    /// The vectors array provides the directions that this method should try in a
    /// repeated fashion until moving is no longer possible.
    fn try_moves_multiple<const COUNT: usize>(&self,
        moveset: &mut HashSet<BoardPos>,
        start: &BoardPos,
        friendly_color: &Color,
        vectors: [(i8, i8); COUNT]
    ) {
        for (delta_file, delta_rank) in vectors {
            self.try_move_multiple(moveset, start, friendly_color, delta_file, delta_rank);
        }
    }

    /// Test a direction and add all the possible moves to the moveset.
    fn try_move_multiple(&self,
        moveset: &mut HashSet<BoardPos>,
        start: &BoardPos,
        friendly_color: &Color,
        delta_file: i8,
        delta_rank: i8
    ) {
        let mut pos = (*start).clone();
        loop {
            let new_pos = self.try_move_once(&pos, delta_file, delta_rank, friendly_color);
            let new_move = match new_pos {
                None => break,
                Some(new_move) => new_move,
            };
            let (new_pos, move_type) = new_move;
            pos = new_pos;
            moveset.insert(pos.clone());
            if move_type == MoveType::Attacking {
                // Attacking a piece is a valid move, but the piece can not move further after
                // attacking, otherwise it would effectively be jumping over the enemy.
                break;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{board::{Tile, Board}, piece::PieceType};
    use super::*;

    #[test]
    fn move_piece() {
        let mut game = Game::new();
        game.move_piece(&"e2".parse().unwrap(), &"e4".parse().unwrap()).unwrap();
        // assert_eq!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", game.to_fen());
        // Castling and en passant not yet implemented
        assert_eq!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b - - 0 0", game.to_fen());
    }

    /// Prepare a game for a moveset test.
    /// 
    /// The specified piece is placed at `e4`.
    fn prepare_moveset_test(piece: PieceType) -> (Game, BoardPos) {
        let pos = "e4".parse().unwrap();
        let game = prepare_moveset_test_at(piece, &pos);
        (game, pos)
    }

    /// Prepare a game for a moveset test by placing the piece at the
    /// specified position.
    fn prepare_moveset_test_at(piece: PieceType, pos: &BoardPos) -> Game {
        const COLOR: Color = Color::White;

        let mut board = Board::empty();
        let tile = Tile::new(piece, COLOR);
        board.set_tile(&pos, tile);

        Game {
            board,
            current_turn: COLOR
        }
    }

    /// Format a set of board positions by sorting them and presenting their
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

    // Movement tests

    #[test]
    fn king_moves() {
        let (mut game, pos) = prepare_moveset_test(PieceType::King);
        let actual = game.get_legal_moves(&pos).unwrap();
        assert_moves(&actual, "d5 e5 f5 d4 f4 d3 e3 f3");
    }

    #[test]
    fn queen_moves() {
        let (mut game, pos) = prepare_moveset_test(PieceType::Queen);
        let actual = game.get_legal_moves(&pos).unwrap();
        assert_moves_exist(&actual, "f5 f4 h7 b4");
        assert_moves_dont_exist(&actual, "e4 d2 f6");
    }

    #[test]
    fn rook_moves() {
        let (mut game, pos) = prepare_moveset_test(PieceType::Rook);

        let pos2 = "c4".parse().unwrap();
        let enemy = Tile::new(PieceType::Pawn, Color::Black);
        game.board.set_tile(&pos2, enemy);

        let actual = game.get_legal_moves(&pos).unwrap();
        assert_moves_exist(&actual, "d4 c4 e5 e7 h4 e3");
        assert_moves_dont_exist(&actual, "e4 f5 d2 d7 b4 a4");
    }

    #[test]
    fn knight_moves() {
        let (mut game, pos) = prepare_moveset_test(PieceType::Knight);
        let actual = game.get_legal_moves(&pos).unwrap();
        assert_moves(&actual, "d6 f6 g5 g3 f2 d2 c3 c5");
    }

    #[test]
    fn first_pawn_moves() {
        let pos = "e2".parse().unwrap();
        let mut game = prepare_moveset_test_at(PieceType::Pawn, &pos);
        let actual = game.get_legal_moves(&pos).unwrap();
        assert_moves(&actual, "e3 e4");
    }

    #[test]
    fn moved_pawn_moves() {
        let (mut game, pos) = prepare_moveset_test(PieceType::Pawn);
        let actual = game.get_legal_moves(&pos).unwrap();
        assert_moves(&actual, "e5");
    }

    #[test]
    fn attacking_pawn_moves() {
        let (mut game, pos) = prepare_moveset_test(PieceType::Pawn);
        
        let enemy = Tile::new(PieceType::Pawn, Color::Black);
        game.board.set_tile(&"d5".parse().unwrap(), enemy);
        
        let actual = game.get_legal_moves(&pos).unwrap();
        assert_moves(&actual, "d5 e5");
    }

    // Movement tests including check

    #[test]
    fn valid_moves_not_state_of_check() {
        let pos = "e4".parse().unwrap();
        let mut game = Game::from_fen("k7/4r3/8/8/4R3/8/4K3/8 w - - 0 1").unwrap();

        let moves = game.get_legal_moves(&pos).unwrap();

        // It is not legal to move the rook so that it unblocks the black rook's
        // attacking path to the white king, which would result in a state of check.
        assert_moves(&moves, "e7 e6 e5 e3");
    }
}