# Rust Tetris

A simple terminal-based Tetris game written in Rust. It leverages [crossterm](https://crates.io/crates/crossterm) for terminal I/O, [ratatui](https://crates.io/crates/ratatui) for UI rendering, and [clap](https://crates.io/crates/clap) for command-line argument parsing.

## Features

- **Classic Tetris Gameplay:** Play the classic game in the terminal including hold, ghost pieces, and a score counter.
- **Configurable Controls:** Customize key bindings via command-line arguments.
- **Adjustable Tick Speed:** Set the game tick speed to control the falling speed of the tetrominoes.
- **Save-State Support:** Load a saved game state from a text file.

## Requirements

- **Rust** (latest stable version recommended)
- **Cargo** (Rust’s package manager)

## Installation

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/NiclasMcClymont/Tuitris.git
   cd Tuitris
   ```

2. **Build the Project:**

   ```bash
   cargo build --release
   ```

## Running the Game

You can run the game directly with Cargo:

```bash
cargo run --release
```

### Command-Line Options

Configure various aspects of the game using flags:

- **Set Key Bindings:**

  ```bash
  cargo run --release -- --move_left a --move_right d --move_down s --rotate_cw w --rotate_ccw q --hold e --hard_drop Space --pause p --restart r --quit x
  ```

- **Load Key Bindings From File:**

  ```bash
  cargo run --release -- --kb-file kb-examples/vim.txt
  ```

- **Set Tick Speed:**

  ```bash
  cargo run --release -- --tick_speed 300
  ```

- **Load a Save State:**

  ```bash
  cargo run --release -- --load_state path/to/save.txt
  ```

- **Create a Sample Save File:**

  ```bash
  cargo run --release -- --create_sample sample_save.txt
  ```

View all options:

```bash
cargo run --release -- --help
```

## Controls

Default key bindings:

- **Left Arrow:** Move Left
- **Right Arrow:** Move Right
- **Down Arrow:** Move Down
- **Up Arrow:** Rotate Clockwise
- **Z:** Rotate Counterclockwise
- **H:** Hold Piece
- **Space:** Hard Drop
- **P:** Pause/Resume
- **R:** Restart
- **Q:** Quit

> [!TIP]
> Key bindings can be customized via command-line options

### Keybind File

You can create a `.txt` file with custom keybinds and load it with the `--kb-file` flag. Example keybind files are in [./kb-examples/](kb-examples/). 

## Save-State File Format

The save-state file is a simple text file:

```
score: 0
lines: 0
level: 1
current_piece: T 0 5 0
next_piece: I 0 5 0
hold_piece: none
board:
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
..........
```

Generate the sample save file above:

```bash
cargo run --release -- --create_sample sample_save.txt
```

## Contributing

Feel free to open issues or submit pull requests. When contributing, please follow the existing code style and document your changes.

## License

This project is licensed under the MIT License.

