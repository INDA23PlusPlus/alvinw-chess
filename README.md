# alvinw-chess

## API Usage

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
1. the file is a `u8` integer between `[0-7]` (inclusive), where `0` is file `a`, `1` is rank `b`, ..., `7` is rank `h`.
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

## Getting valid moves
To find out the possible moves a piece could take during the next move, the `get_legal_moves` method on `Game` can be used.

Legal moves are defined as moves that:
1. follow the movement rules for the piece. Eg. a bishop can only walk diagonally.
2. respect the environment. Eg. not jumping over pieces unless the piece allows that.
3. do not move outside of the board.
4. do not move into a state of check.

This method is safe to call on any `BoardPos` instance, but be sure to handle the `Result` errors. The method will error with `NoTile` if there was no tile (no piece) at the square. It will error with `NotCurrentTurn` if there is a tile, but of the wrong color. The `get_legal_moves` method can only be used to see valid moves for the current team's pieces.

TODO more documentation!