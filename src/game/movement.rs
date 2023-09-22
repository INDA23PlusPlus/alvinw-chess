use std::collections::HashSet;

use crate::{pos::BoardPos, board::{Color, Tile}, piece::PieceType};

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
    /// color that is not the current turn.
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

struct PerformedMove {
    changed_tiles: Vec<(BoardPos, Option<Tile>)>,
    had_capture: bool,
}

impl Game {

    /// Move a piece.
    /// 
    /// This method will move the piece, increment the move counter and change the
    /// turn to the opposite color.
    /// 
    /// In case the `to` and `from` positions describe an en passant or castling
    /// move, other pieces than those two positions will also be changed in order
    /// to complete the en passant or castling.
    /// 
    /// This method should always be immediately followed by `get_status` since a
    /// move might result in the player needing to promote a piece. See the
    /// `promote` method.
    /// 
    /// # Errors
    /// If there is no tile (no piece) at the position `NoTile` will be errored.
    /// 
    /// If the piece at `from` is of the wrong color, aka the color who's turn it is
    /// not to play right now, this method will error with `NotCurrentTurn`.
    /// 
    /// In case the move is not valid, `InvalidMove` is returned. If this method
    /// was immediately preceded by `get_legal_move` on `from`, and the `to`
    /// position was a part of the returned moveset, this method will never error
    /// since the move is guaranteed to be valid.
    pub fn move_piece(&mut self, from: &BoardPos, to: &BoardPos) -> Result<(), MovePieceError> {
        let moveset = match self.get_legal_moves(from) {
            Ok(moveset) => moveset,
            Err(GetMovesetError::NoTile) => return Err(MovePieceError::NoTile),
            Err(GetMovesetError::NotCurrentTurn) => return Err(MovePieceError::NotCurrentTurn),
        };

        if !moveset.contains(to) {
            return Err(MovePieceError::InvalidMove);
        }

        let tile = self.board.get_tile(from).expect("Move is already validated.");

        let performed_move = self.perform_move(from, to);

        self.halfmove_clock += 1;
        if performed_move.had_capture {
            self.halfmove_clock = 0;
        }

        // Clear any potensial previous en passant squares as en passant is only valid
        // if the pawn moved directly before the en passant attack occurs.
        self.en_passant_target = None;

        // Check for new en passant possibilities
        if tile.piece() == PieceType::Pawn && from.rank().abs_diff(to.rank()) == 2 {
            // A pawn moved two steps. Record the en passant target position for the passed
            // square.
            let dir = if to.rank() > from.rank() { 1 } else { -1 };
            let rank = from.rank() as i8 + dir;
            if rank >= 0 {
                self.en_passant_target = Some(BoardPos::new(from.file(), rank as u8));
            }
        }

        // Remove castling availability when moving the king.
        if tile.piece() == PieceType::King {
            let castling_availability = match tile.color() {
                Color::White => &mut self.white_castling,
                Color::Black => &mut self.black_castling,
            };
            castling_availability.kingside = false;
            castling_availability.queenside = false;
        }

        // Remove castling availability when moving rooks.
        if tile.piece() == PieceType::Rook {
            // Check if the rooks are moving away from their starting positions.
            let starting_rank = if tile.color() == Color::White { 0 } else { 7 };
            if from.rank() == starting_rank {
                let castling_availability = match tile.color() {
                    Color::White => &mut self.white_castling,
                    Color::Black => &mut self.black_castling,
                };
                if from.file() == 0 {
                    castling_availability.queenside = false;
                }
                if from.file() == 7 {
                    castling_availability.kingside = false;
                }
            }
        }

        // Check if promotion is required
        let last_rank = if tile.color() == Color::White { 7 } else { 0 };
        if to.rank() == last_rank && tile.piece() == PieceType::Pawn {
            self.promotion_required = Some(to.clone());
        }

        if self.current_turn == Color::Black {
            self.fullmove_number += 1;
        }

        self.current_turn = self.current_turn.opposite();
        
        Ok(())
    }

    /// An internal method for performing moves without validating them or affecting
    /// future gameplay.
    ///
    /// It is important that the move being performed is valid, else this method
    /// will panic.
    ///
    /// This method will only move the pieces in accordance to chess rules. In most
    /// cases this method will move the piece at `from` to the tile at `to`. This
    /// method will additionally move the rook to perform castling, and capture
    /// pieces being taken en passant.
    /// 
    /// This method does not change the playing team and does not store information
    /// about changes to castling or en passant as a result of this move.
    /// 
    /// Therefore, this method can be used to "preview" a move without affecting
    /// gameplay, and can easially be reversed by calling `undo_performed_move`
    /// with the return value of this method.
    fn perform_move(&mut self, from: &BoardPos, to: &BoardPos) -> PerformedMove {

        let tile = self.board.get_tile(from).expect("Move is already validated.");

        let mut performed_move = PerformedMove {
            changed_tiles: Vec::with_capacity(3),
            had_capture: false,
        };
        
        // Record the tile before it is moved.
        performed_move.changed_tiles.push((from.clone(), Some(tile)));

        // Record the tile currently at the position we are about to move to.
        let to_tile = self.board.get_tile(to);
        performed_move.changed_tiles.push((to.clone(), to_tile));
        if to_tile.is_some() {
            performed_move.had_capture = true;
        }

        // Castling
        if tile.piece() == PieceType::King && from.file().abs_diff(to.file()) == 2 {
            // The king moved two tiles. This means we are castling.
            let dir = if to.file() > from.file() { 1 } else { -1 };
            
            let new_rook_pos = from.offset(dir, 0)
                .expect("Move is already validated by get_legal_moves");
            
            let rook_pos = self.find_rook(&new_rook_pos, &tile.color(), dir)
                .expect("Move is already validated by get_legal_moves");

            let rook = self.board.remove_tile(&rook_pos).expect("Rook exists.");

            // Record tiles before performing the move in case the move
            // needs to be undone.
            performed_move.changed_tiles.push((rook_pos, Some(rook)));
            self.record_tile(&new_rook_pos, &mut performed_move);

            self.board.remove_tile(from);
            self.board.set_tile(to, tile);
            self.board.set_tile(&new_rook_pos, rook);

        } else {
            // Normal move
            self.board.remove_tile(from);
            self.board.set_tile(to, tile);
        }

        // En passant
        if tile.piece() == PieceType::Pawn
            && self.en_passant_target.as_ref().is_some_and(|en_passant_target| en_passant_target == to) {
            // A pawn just performed en passant.
            // We need to capture the pawn being taken en passant.
            // That pawn will be placed on the same file as the "to" position, and the same
            // rank as the "from" position.
            let attacked_pawn_pos = BoardPos::new(to.file(), from.rank());
            let attacked_pawn = self.board.remove_tile(&attacked_pawn_pos);

            // The following panic! statements should never be called but exist for
            // validating purposes.
            // The get_legal_moves function will only allow an en passant situation when it
            // is valid, so we are already know the situation is a valid en passant.
            match attacked_pawn {
                None => panic!("En passant occured but there was no pawn to attack."),
                Some(attacked_pawn) => {
                    if attacked_pawn.piece() != PieceType::Pawn || attacked_pawn.color() == tile.color() {
                        panic!("Did not attack an enemy pawn.");
                    }
                    performed_move.had_capture = true;
                    performed_move.changed_tiles.push((attacked_pawn_pos, Some(attacked_pawn)));
                }
            }
        }

        performed_move
    }

    fn record_tile(&self, pos: &BoardPos, performed_move: &mut PerformedMove) {
        let tile = self.board.get_tile(pos);
        performed_move.changed_tiles.push((pos.clone(), tile));
    }

    /// Undo a move that was just performed by `perform_move`.
    fn undo_performed_move(&mut self, performed_move: PerformedMove) {
        // Restore all tiles that changed to their state before the change.
        for (pos, tile) in performed_move.changed_tiles {
            self.board.set_or_remove_tile(&pos, tile);
        }
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

        let mut moveset = self.get_pseudo_legal_moves(pos, true);
        moveset.retain(|move_pos| {
            // Ensure the move does not move into a state of check.
            // Attempt the move.

            // Move there by setting the tiles directly.
            let performed_move = self.perform_move(pos, move_pos);
            let check = self.is_check(&tile.color());
            // This move resulted in a state of check. It is not a legal move.

            // Undo the move.
            self.undo_performed_move(performed_move);

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
    /// If the `include_castling` parameter is `true`, castling will also be checked
    /// and added to the moveset when applicable.
    ///
    /// ## Panics
    /// This function will panic if there is no piece at the tile.
    pub(super) fn get_pseudo_legal_moves(&self, pos: &BoardPos, include_castling: bool) -> HashSet<BoardPos> {
        let tile = self.board.get_tile(pos)
            .expect("Attempt to get pseudo-legal moves from empty tile.");

        let mut moveset = HashSet::new();

        match tile.piece() {
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
            PieceType::King => {
                self.try_moves_once(&mut moveset, &pos, &tile.color(), [
                    (-1,  1), (0,  1), (1,  1),
                    (-1,  0), /******/ (1,  0),
                    (-1, -1), (0, -1), (1, -1),
                ]);

                // Castling

                let castling_availability = match tile.color() {
                    Color::White => &self.white_castling,
                    Color::Black => &self.black_castling,
                };

                if include_castling
                    && (castling_availability.kingside || castling_availability.queenside)
                    && !self.is_check(&tile.color()) {
                    // Castling is not possible if the king is in check.

                    if castling_availability.kingside {
                        self.try_castling(&pos, &tile.color(), &mut moveset, 1);
                    }
                    if castling_availability.queenside {
                        self.try_castling(&pos, &tile.color(), &mut moveset, -1);
                    }
                }
            }
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

                // En passant
                if let Some(en_passant_target) = &self.en_passant_target {
                    if en_passant_target.rank() as i8 == pos.rank() as i8 + dir && en_passant_target.file().abs_diff(pos.file()) == 1 {
                        // There is an en passant target square forward-diagonally to this pawn.

                        // The position of the pawn that will be attacked by this en passant.
                        let attacked_pawn_pos = BoardPos::new(en_passant_target.file(), pos.rank());

                        let attacked_pawn = self.board.get_tile(&attacked_pawn_pos);
                        if let Some(attacked_pawn) = attacked_pawn {
                            if attacked_pawn.piece() == PieceType::Pawn && attacked_pawn.color() != tile.color() {
                                // There is an enemy pawn at the location. En passant is possible.
                                moveset.insert(en_passant_target.clone());
                            }                            
                        }
                    }
                }
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

    /// Test castling in the specified direction, and if castling is possible, add
    /// the position where the king will result after castling to the moveset.
    fn try_castling(&self, start: &BoardPos, color: &Color, moveset: &mut HashSet<BoardPos>, dir: i8) {
        // The tile that the king will cross over while castling.
        let cross_over_pos = start.offset(dir, 0);
        let cross_over_pos = match cross_over_pos {
            Some(p) => p,
            None => return
        };

        let enemy_color = color.opposite();

        if self.is_attacked_by(&cross_over_pos, &enemy_color) {
            // If the enemy can attack the position being crossed over, castling is not legal.
            return;
        }

        // The position the king would end up at if the castling is performed.
        let king_pos = cross_over_pos.offset(dir, 0);
        let king_pos = match king_pos {
            Some(p) => p,
            None => return
        };

        let rook = self.find_rook(&king_pos, color, dir);
        if rook.is_some() {
            // A rook was found and there were no pieces between. Castling is possible.
            moveset.insert(king_pos);
        }
    }

    fn find_rook(&self, start: &BoardPos, color: &Color, dir: i8) -> Option<BoardPos> {
        // Traverse until we find a rook
        let mut pos = (*start).clone();
        loop {
            let tile = self.board.get_tile(&pos);
            if let Some(tile) = tile {
                if tile.color() != *color || tile.piece() != PieceType::Rook {
                    return None;
                }
                return Some(pos);
            }

            // A vacant slot, let's keep searching for a rook.
            pos = match pos.offset(dir, 0) {
                None => return None,
                Some(new_pos) => new_pos,
            };
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{board::{Tile, Board}, piece::PieceType, game::CastlingAvailability};
    use super::*;

    #[test]
    fn move_piece() {
        let mut game = Game::new();
        game.move_piece(&"e2".parse().unwrap(), &"e4".parse().unwrap()).unwrap();
        assert_eq!("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 1 1", game.to_fen());
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
            current_turn: COLOR,
            white_castling: CastlingAvailability { kingside: false, queenside: false },
            black_castling: CastlingAvailability { kingside: false, queenside: false },
            en_passant_target: None,
            promotion_required: None,
            halfmove_clock: 0,
            fullmove_number: 0,
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

    #[test]
    fn castling_possible() {
        let mut game = Game::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let white_king_pos = "e1".parse().unwrap();
        let black_king_pos = "e8".parse().unwrap();

        let white_moves = game.get_legal_moves(&white_king_pos).unwrap();
        game.current_turn = Color::Black;
        let black_moves = game.get_legal_moves(&black_king_pos).unwrap();

        assert_moves(&white_moves, "c1 d1 d2 e2 f2 f1 g1");
        assert_moves(&black_moves, "c8 d8 d7 e7 f7 f8 g8");
    }

    #[test]
    fn castling_not_possible() {
        let mut game = Game::new();

        game.move_piece(&"a2".parse().unwrap(), &"a3".parse().unwrap()).unwrap();
        game.move_piece(&"h7".parse().unwrap(), &"h6".parse().unwrap()).unwrap();
        game.move_piece(&"a1".parse().unwrap(), &"a2".parse().unwrap()).unwrap();
        game.move_piece(&"h8".parse().unwrap(), &"h7".parse().unwrap()).unwrap();

        assert_eq!(game.to_fen(), "rnbqkbn1/pppppppr/7p/8/8/P7/RPPPPPPP/1NBQKBNR w Kq - 4 3");
    }

    #[test]
    fn castling() {
        let mut game = Game::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();

        game.move_piece(&"e1".parse().unwrap(), & "g1".parse().unwrap()).unwrap();
        game.move_piece(&"e8".parse().unwrap(), & "c8".parse().unwrap()).unwrap();

        assert_eq!(game.to_fen(), "2kr3r/8/8/8/8/8/8/R4RK1 w - - 2 2");
    }

    #[test]
    fn en_passant() {
        let mut game = Game::from_fen("4k3/8/8/8/2p5/8/1P6/4K3 w - - 0 1").unwrap();

        game.move_piece(&"b2".parse().unwrap(), &"b4".parse().unwrap()).unwrap();

        let moves = game.get_legal_moves(&"c4".parse().unwrap()).unwrap();
        assert_moves(&moves, "c3 b3");

        game.move_piece(&"c4".parse().unwrap(), &"b3".parse().unwrap()).unwrap();

        assert_eq!(game.to_fen(), "4k3/8/8/8/8/1p6/8/4K3 w - - 0 2");
    }

    #[test]
    fn discovery_via_en_passant() {
        let mut game = Game::from_fen("8/8/8/8/1R2p1k1/8/3P4/4K3 w - - 0 1").unwrap();

        game.move_piece(&"d2".parse().unwrap(), &"d4".parse().unwrap()).unwrap();

        let moves = game.get_legal_moves(&"e4".parse().unwrap()).unwrap();

        // Taking en passant, which unblocks the rook to attack the king. This move is illegal.
        assert_moves_dont_exist(&moves, "d3");
    }

    #[test]
    fn undoing_performed_castling() {
        let mut game = Game::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();

        let performed_move = game.perform_move(&"e1".parse().unwrap(), &"c1".parse().unwrap());
        game.undo_performed_move(performed_move);

        assert_eq!(game.to_fen(), "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    }

    #[test]
    fn undoing_performed_en_passant() {
        let mut game = Game::from_fen("4k3/8/8/1pP5/8/8/8/4K3 w - b6 0 1").unwrap();

        let performed_move = game.perform_move(&"c5".parse().unwrap(), &"b6".parse().unwrap());
        game.undo_performed_move(performed_move);

        assert_eq!(game.to_fen(), "4k3/8/8/1pP5/8/8/8/4K3 w - b6 0 1");
    }

    #[test]
    fn check_must_move_to_non_check() {
        let mut game = Game::from_fen("4k3/8/8/8/2b5/8/3PK2P/8 w - - 0 1").unwrap();

        assert!(game.is_check(&Color::White));

        // When there is check, only moves that make the game exit check are legal.

        let moves1 = game.get_legal_moves(&"d2".parse().unwrap()).unwrap();
        assert_moves(&moves1, "d3");

        let moves2 = game.get_legal_moves(&"e2".parse().unwrap()).unwrap();
        assert_moves(&moves2, "e3 f3 f2 e1 d1");

        // Moving this pawn is usually legal, but it does not help the check situation.
        let moves3 = game.get_legal_moves(&"h2".parse().unwrap()).unwrap();
        assert_moves(&moves3, "");
    }
}