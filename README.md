# alvinw-chess

# API Usage
This section describes an overview of important API methods.

Before using a method, consult the method documentation to ensure you are always passing valid arguments to avoid unexpected panics. This is especially important in cases where the arguments come from user input.

## The Game instance
Start by creating the game instance, usually this is done by
```rust
let mut game = Game::new();
```
to create a game with pieces in the standard starting position.

It is also possible to create a game instance from a FEN-string by using
```rust
let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
```
and handling the error accordingly.

## Board Positions
The `BoardPos` struct is used to represent **valid** positions on the board. For example `e4`, `b2`, and `h7`.

There are primarily two ways of creating `BoardPos` instances.

### `BoardPos::new`

You can create a `BoardPos` instance from the file and rank where
1. the file is a `u8` integer between `[0-7]` (inclusive), where `0` is file a, `1` is rank b, ..., `7` is rank h.
2. the rank is a `u8` integer between `[0-7]` (inclsuive), where `0` is rank 1, `1` is rank 2, ..., `7` is rank 8.

It is important to distinguish between the human-readable file and ranks and the internal representation, since the rank is zero indexed in the code, while starting at rank 1 for humans.

```rust
let pos = BoardPos::new(1, 1); // = b2
```

> **Note: Panics!** The `BoardPos::new` method **will panic** if the file or rank is outside the range `[0-7]`. It is therefore very important that in all cases where the values passed to the `new` functions the values lie in the correct range. If the values come from user input, it must be validated before being passed to the method to avoid panics.
> 
> Panicing ensures that all instances of `BoardPos` are **valid** positions on the board. It is not possible to represent a position outside of the board.

### Parsing from strings
`BoardPos` instances can also be created by parsing them from a string in algebraic notation. This means the string `"e4"` can become the corresponding `BoardPos` instance.

```rust
let pos: BoardPos = "e4".parse().unwrap();
```

Be sure to handle errors occordingly.

## Getting pieces on the board
Pieces on the board are represented using the curiously named `Tile` struct. This struct holds information about the piece stored on a a square/tile on the board. `Tile` instances are not tied to a `BoardPos` though.

A `Tile` instance holds the `PieceType` and `Color`, so a a tile instance could for example be a black king or a white knight.

To get the `Tile` currently on a `BoardPos`, use the `get_tile` method on `Game`.

For example, to read the entire board, you can use:
```rs
for file in 0..=7 {
    for rank in 0..=7 {
        let pos = BoardPos::new(file, rank);
        let tile = game.get_tile(&pos);
        println!("tile = {:?}", tile);
    }
}
```

## Getting valid moves
To find out the possible moves a piece could take during the next move, the `get_legal_moves` method on `Game` can be used.

Legal moves are defined as moves that:
1. follow the movement rules for the piece. Eg. a bishop can only walk diagonally.
2. respect the environment. Eg. not jumping over pieces unless the piece allows that.
3. do not move outside of the board.
4. do not move into a state of check.

This method is safe to call on any `BoardPos` instance, but be sure to handle the `Result` errors. The method will error with `NoTile` if there was no tile (no piece) at the square. It will error with `NotCurrentTurn` if there is a tile, but of the wrong color. The `get_legal_moves` method can only be used to see valid moves for the current team's pieces.

These moves can be used to display the possible moves a piece can take.

## Moving
To perform a move, use the the `move_piece` method.

This method will move the piece, increment the move counter and change the turn to the opposite color.

The method returns a `Result<(), MovePieceError>`. In other words, nothing is returned for successful moves. It is though important to handle errors.

This method should always be immediately followed by `get_state` since a move might result in the player needing to promote a piece.

The following example showcases a typical usage of this method.

```rust
match game.move_piece(from, to) {
    Ok(_) => println!("Moved {from} to {to}"),
    Err(MovePieceError::NoTile) => panic!("The tile {from} is empty!"),
    Err(MovePieceError::NotCurrentTurn) => panic!("You can not move your opponent's pieces!"),
    Err(MovePieceError::InvalidMove) => panic!("That is not a valid move."),
};

match game.get_state() {
    GameState::Normal => println!("Ok, next player to move."),
    GameState::Check(color) => println!("The next player to play is in check!"),
    GameState::Checkmate(color) => println!("You win!"),
    GameState::PromotionRequired(pos) => {
        println!("The pawn at {pos} needs to be promoted, choose a piece:");
        // [...] user input stuff
        game.promote(PieceType::Queen);
    },
};

// Next player's turn.
```

The move will only have been performed if `move_piece` returns `Ok`. If `move_piece` returns an error, you need to handle the error and the player must instead make another move that is valid.

After a successful move, check the status to see if the player needs to choose a piece to promote if the move in question was a promotion. In other cases it is time for the opponent to play.

You can always use `game.current_turn()` to get the `Color` who should play (using `move_piece`) next.

## Castling and en passant
Castling and en passant are implemented like any other move, and nothing special needs to be done by the consumer of the library.

When en passant is applicable, the square the piece will end up at will appear as legal in `get_legal_moves`, and if the piece moves to that square using `move_piece`, the opponent's pawn being taken en passant will be captured.

When castling is applicable, the square the king will end up at after castling will appear as legal in `get_legal_moves`, and if the king moves to that square using `move_piece`, the rook will also be moved to the correct square when `move_piece` is called.

## Promotion
As seen in the example before, after calling `move_piece`, there is a possibility that `get_state` returns `PromotionRequired` if the player moved a pawn to the final rank. The `promote` method must be called directly after (before the next move) to let the player choose which piece to promote the rook to. Players usually choose the queen, but the player can choose other pieces, except for the king or a pawn.

The `promote` method must only by used after getting a `PromotionRequired` state. Attempting to call this method in other cases will result in a panic. Passing incorrect piece types (a king or a pawn) will also result in a panic, so make sure user input is validated before being passed to the method. Consult the method documentation for more information.

## Low-level board access
The `Game` struct provides method to interact with the game according to Chess rules. You can use the `board()` method to get access to the `Board` instance that stores tiles. There you can get, set and remove tiles directly without validation.

# Feature requests
Open an issue to request a feature!