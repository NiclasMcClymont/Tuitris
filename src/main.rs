use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use clap::Parser;
use std::io;
use std::time::{Duration, Instant};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const BOARD_BACKGROUND: Color = Color::Rgb(30, 30, 30);

#[derive(Clone, Copy, PartialEq)]
enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RotationDirection {
    CW,  // Clockwise
    CCW, // Counterclockwise
}

#[derive(Clone, Copy)]
struct Piece {
    tetromino: TetrominoType,
    rotation: usize,
    x: i32,
    y: i32,
}

/// Returns the list of (x,y) coordinates for the given piece based on its type and rotation.
fn get_piece_coords(piece: &Piece) -> Vec<(i32, i32)> {
    match piece.tetromino {
        TetrominoType::I => {
            let rotations = [
                vec![(-1, 0), (0, 0), (1, 0), (2, 0)],
                vec![(0, -1), (0, 0), (0, 1), (0, 2)],
            ];
            let shape = &rotations[piece.rotation % 2];
            shape.iter().map(|(dx, dy)| (piece.x + dx, piece.y + dy)).collect()
        }
        TetrominoType::O => {
            let shape = vec![(0, 0), (1, 0), (0, 1), (1, 1)];
            shape.iter().map(|(dx, dy)| (piece.x + dx, piece.y + dy)).collect()
        }
        TetrominoType::T => {
            let rotations = [
                vec![(-1, 0), (0, 0), (1, 0), (0, 1)],
                vec![(0, -1), (0, 0), (0, 1), (1, 0)],
                vec![(-1, 0), (0, 0), (1, 0), (0, -1)],
                vec![(0, -1), (0, 0), (0, 1), (-1, 0)],
            ];
            let shape = &rotations[piece.rotation % 4];
            shape.iter().map(|(dx, dy)| (piece.x + dx, piece.y + dy)).collect()
        }
        TetrominoType::S => {
            let rotations = [
                vec![(0, 0), (1, 0), (-1, 1), (0, 1)],
                vec![(0, -1), (0, 0), (1, 0), (1, 1)],
            ];
            let shape = &rotations[piece.rotation % 2];
            shape.iter().map(|(dx, dy)| (piece.x + dx, piece.y + dy)).collect()
        }
        TetrominoType::Z => {
            let rotations = [
                vec![(-1, 0), (0, 0), (0, 1), (1, 1)],
                vec![(1, -1), (0, 0), (1, 0), (0, 1)],
            ];
            let shape = &rotations[piece.rotation % 2];
            shape.iter().map(|(dx, dy)| (piece.x + dx, piece.y + dy)).collect()
        }
        TetrominoType::J => {
            let rotations = [
                vec![(-1, 0), (0, 0), (1, 0), (1, 1)],
                vec![(0, -1), (0, 0), (0, 1), (1, -1)],
                vec![(-1, -1), (-1, 0), (0, 0), (1, 0)],
                vec![(-1, 1), (0, -1), (0, 0), (0, 1)],
            ];
            let shape = &rotations[piece.rotation % 4];
            shape.iter().map(|(dx, dy)| (piece.x + dx, piece.y + dy)).collect()
        }
        TetrominoType::L => {
            let rotations = [
                vec![(-1, 0), (0, 0), (1, 0), (-1, 1)],
                vec![(0, -1), (0, 0), (0, 1), (1, 1)],
                vec![(-1, 0), (0, 0), (1, 0), (1, -1)],
                vec![(0, -1), (0, 0), (0, 1), (-1, -1)],
            ];
            let shape = &rotations[piece.rotation % 4];
            shape.iter().map(|(dx, dy)| (piece.x + dx, piece.y + dy)).collect()
        }
    }
}

/// Checks whether the given (x,y) coordinate is within the board bounds.
fn in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < BOARD_WIDTH as i32 && y < BOARD_HEIGHT as i32
}

/// The main game state structure.
struct Game {
    board: [[Option<TetrominoType>; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Piece,
    next_piece: Piece,
    last_tick: Instant,
    is_game_over: bool,
    paused: bool,
    score: u32,
    lines_cleared: u32,
    level: u32,
    hold_piece: Option<Piece>,
    hold_used: bool,
    start_time: Instant,
    tick_duration: Duration,
}

impl Game {
    /// Creates a new game.
    fn new(tick_duration: Duration) -> Self {
        let current_piece = Game::generate_piece();
        let next_piece = Game::generate_piece();
        Game {
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece,
            next_piece,
            last_tick: Instant::now(),
            is_game_over: false,
            paused: false,
            score: 0,
            lines_cleared: 0,
            level: 1,
            hold_piece: None,
            hold_used: false,
            start_time: Instant::now(),
            tick_duration,
        }
    }

    /// Randomly generates a new tetromino piece.
    fn generate_piece() -> Piece {
        let mut rng = rand::thread_rng();
        let tetromino = match rng.gen_range(0..7) {
            0 => TetrominoType::I,
            1 => TetrominoType::O,
            2 => TetrominoType::T,
            3 => TetrominoType::S,
            4 => TetrominoType::Z,
            5 => TetrominoType::J,
            _ => TetrominoType::L,
        };
        Piece {
            tetromino,
            rotation: 0,
            x: (BOARD_WIDTH / 2) as i32,
            y: 0,
        }
    }

    /// Returns true if the piece is in a valid position.
    fn is_valid_position(&self, piece: &Piece) -> bool {
        for (x, y) in get_piece_coords(piece) {
            if !in_bounds(x, y) || (y >= 0 && self.board[y as usize][x as usize].is_some()) {
                return false;
            }
        }
        true
    }

    /// Locks the current piece into the board and spawns a new one.
    /// If the new piece cannot be placed, the game is over.
    fn lock_piece(&mut self) {
        for (x, y) in get_piece_coords(&self.current_piece) {
            if in_bounds(x, y) && y >= 0 {
                self.board[y as usize][x as usize] = Some(self.current_piece.tetromino);
            }
        }
        self.clear_full_lines();
        self.current_piece = self.next_piece;
        self.next_piece = Game::generate_piece();
        if !self.is_valid_position(&self.current_piece) {
            self.is_game_over = true;
        }
        self.hold_used = false;
    }

    /// Clears full lines from the board and moves the remaining rows down.
    fn clear_full_lines(&mut self) {
        let mut new_board: Vec<[Option<TetrominoType>; BOARD_WIDTH]> = Vec::new();
        for row in self.board.iter() {
            if row.iter().all(|&cell| cell.is_some()) {
                // Skip full row.
            } else {
                new_board.push(*row);
            }
        }
        let cleared = BOARD_HEIGHT - new_board.len();
        for _ in 0..cleared {
            new_board.insert(0, [None; BOARD_WIDTH]);
        }
        // Copy new board rows back.
        for (i, row) in new_board.into_iter().enumerate() {
            self.board[i] = row;
        }
        if cleared > 0 {
            self.lines_cleared += cleared as u32;
            let points = match cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800,
                _ => cleared * 200,
            };
            self.score += points as u32;
            self.level = self.lines_cleared / 10 + 1;
        }
    }

    /// Moves the current piece by (dx, dy). If moving down is not possible, locks the piece.
    fn move_piece(&mut self, dx: i32, dy: i32) {
        let mut new_piece = self.current_piece;
        new_piece.x += dx;
        new_piece.y += dy;
        if self.is_valid_position(&new_piece) {
            self.current_piece = new_piece;
        } else if dy > 0 {
            // The piece cannot move down – lock it.
            self.lock_piece();
        }
    }

    /// Rotates the current piece using the Super Rotation System (SRS).
    fn rotate_piece(&mut self, direction: RotationDirection) {
        // O tetromino does not rotate.
        if self.current_piece.tetromino == TetrominoType::O {
            return;
        }

        let original = self.current_piece;
        let new_rotation =
            (original.rotation + if direction == RotationDirection::CW { 1 } else { 3 }) % 4;

        let kicks = if original.tetromino == TetrominoType::I {
            // SRS kick table for I tetromino (for CCW rotations)
            match (original.rotation, new_rotation) {
                (0, 3) => vec![(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                (3, 2) => vec![(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (2, 1) => vec![(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (1, 0) => vec![(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                _ => vec![(0, 0)],
            }
        } else {
            // SRS kick table for J, L, T, S, Z tetrominoes (for CCW rotations)
            match (original.rotation, new_rotation) {
                (0, 3) => vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (3, 2) => vec![(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (2, 1) => vec![(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (1, 0) => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                _ => vec![(0, 0)],
            }
        };

        for (dx, dy) in kicks.iter() {
            let candidate = Piece {
                tetromino: original.tetromino,
                rotation: new_rotation,
                x: original.x + dx,
                y: original.y + dy,
            };
            if self.is_valid_position(&candidate) {
                self.current_piece = candidate;
                return;
            }
        }
    }

    /// Computes the ghost piece (where the current piece would land).
    fn ghost_piece(&self) -> Piece {
        let mut ghost = self.current_piece;
        while self.is_valid_position(&Piece {
            x: ghost.x,
            y: ghost.y + 1,
            rotation: ghost.rotation,
            tetromino: ghost.tetromino,
        }) {
            ghost.y += 1;
        }
        ghost
    }

    /// Executes a hard drop: immediately moves the piece to its ghost position and locks it.
    fn hard_drop(&mut self) {
        let ghost = self.ghost_piece();
        self.current_piece = ghost;
        self.lock_piece();
    }

    /// Holds the current piece (hold mechanic). If a piece is already held, swaps with it.
    fn hold_current_piece(&mut self) {
        if self.hold_used {
            return;
        }
        self.hold_used = true;
        let temp = self.current_piece;
        if let Some(hold) = self.hold_piece.take() {
            self.current_piece = hold;
            self.hold_piece = Some(temp);
        } else {
            self.hold_piece = Some(temp);
            self.current_piece = self.next_piece;
            self.next_piece = Game::generate_piece();
        }
        // Reset the spawn position for the new piece.
        self.current_piece.x = (BOARD_WIDTH / 2) as i32;
        self.current_piece.y = 0;
        if !self.is_valid_position(&self.current_piece) {
            self.is_game_over = true;
        }
    }

    /// Updates the game state by moving the piece down if the tick duration has passed.
    fn update(&mut self) {
        if self.paused {
            return;
        }
        if self.last_tick.elapsed() >= self.tick_duration {
            self.move_piece(0, 1);
            self.last_tick = Instant::now();
        }
    }

    /// Draws the game board and UI.
    fn draw<B: ratatui::backend::Backend>(
        &self,
        terminal: &mut Terminal<B>,
        key_config: &KeyConfig,
    ) -> Result<(), io::Error> {
        terminal.draw(|f| {
            let size = f.size();
            // Split into game area (left) and info area (right)
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(size);

            // Game board area with a border.
            let game_block = Block::default().title("Tetris").borders(Borders::ALL);
            f.render_widget(game_block, main_chunks[0]);
            let board_area = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(1)
                .split(main_chunks[0]);

            if self.paused {
                let empty_board = Paragraph::new("").style(Style::default().bg(BOARD_BACKGROUND));
                f.render_widget(empty_board, board_area[0]);
            } else {
                let mut lines: Vec<Spans> = Vec::new();
                let current_coords = get_piece_coords(&self.current_piece);
                let ghost = self.ghost_piece();
                let ghost_coords = get_piece_coords(&ghost);

                for y in 0..BOARD_HEIGHT {
                    let mut spans = Vec::new();
                    for x in 0..BOARD_WIDTH {
                        let pos = (x as i32, y as i32);
                        let (cell_str, cell_style) = if current_coords.contains(&pos) {
                            ("[]", Style::default().bg(Game::color_for(self.current_piece.tetromino)))
                        } else if let Some(t) = self.board[y][x] {
                            ("[]", Style::default().bg(Game::color_for(t)))
                        } else if ghost_coords.contains(&pos) {
                            ("[]", Style::default().add_modifier(Modifier::DIM))
                        } else {
                            ("  ", Style::default().bg(BOARD_BACKGROUND))
                        };
                        spans.push(Span::styled(cell_str, cell_style));
                    }
                    lines.push(Spans::from(spans));
                }
                let board_paragraph = Paragraph::new(lines);
                f.render_widget(board_paragraph, board_area[0]);
            }

            // Right info area: split into previews (Hold & Next), stats, and controls.
            let info_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(8),
                        Constraint::Min(3),
                        Constraint::Length(12),
                    ]
                    .as_ref(),
                )
                .split(main_chunks[1]);

            // Previews: Hold (left) and Next Piece (right)
            let preview_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(info_chunks[0]);

            let hold_block = Block::default().title("Hold").borders(Borders::ALL);
            let hold_preview = if let Some(hold) = self.hold_piece {
                create_preview(hold)
            } else {
                create_empty_preview()
            };
            let hold_paragraph = Paragraph::new(hold_preview).block(hold_block);
            f.render_widget(hold_paragraph, preview_chunks[0]);

            let next_block = Block::default().title("Next Piece").borders(Borders::ALL);
            let next_preview = create_preview(self.next_piece);
            let next_paragraph = Paragraph::new(next_preview).block(next_block);
            f.render_widget(next_paragraph, preview_chunks[1]);

            // Stats display.
            let stats_block = Block::default().title("Stats").borders(Borders::ALL);
            let mut stats_text = vec![
                Spans::from(format!("Score: {}", self.score)),
                Spans::from(format!("Lines: {}", self.lines_cleared)),
                Spans::from(format!("Level: {}", self.level)),
                Spans::from(format!("Time: {}s", self.start_time.elapsed().as_secs())),
            ];
            if self.paused {
                stats_text.insert(0, Spans::from("Paused"));
            }
            let stats_paragraph = Paragraph::new(stats_text).block(stats_block);
            f.render_widget(stats_paragraph, info_chunks[1]);

            // Controls display.
            let controls_block = Block::default().title("Controls").borders(Borders::ALL);
            let controls_text = vec![
                Spans::from(format!("{} : Move Left", key_name(key_config.move_left))),
                Spans::from(format!("{} : Move Right", key_name(key_config.move_right))),
                Spans::from(format!("{} : Move Down", key_name(key_config.move_down))),
                Spans::from(format!("{} : Rotate CW", key_name(key_config.rotate_cw))),
                Spans::from(format!("{} : Rotate CCW", key_name(key_config.rotate_ccw))),
                Spans::from(format!("{} : Hold", key_name(key_config.hold))),
                Spans::from(format!("{} : Hard Drop", key_name(key_config.hard_drop))),
                Spans::from(format!("{} : Pause/Resume", key_name(key_config.pause))),
                Spans::from(format!("{} : Restart", key_name(key_config.restart))),
                Spans::from(format!("{} : Quit", key_name(key_config.quit))),
            ];
            let controls_paragraph = Paragraph::new(controls_text).block(controls_block);
            f.render_widget(controls_paragraph, info_chunks[2]);

            // If paused, display a PAUSED banner.
            if self.paused {
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(f.size());
                let game_area = main_chunks[0];
                let area = centered_rect(30, 10, game_area);
                let pause_banner = Paragraph::new("PAUSED")
                    .style(Style::default().fg(Color::Red).bg(Color::Black))
                    .block(Block::default().borders(Borders::ALL).title("Paused"));
                f.render_widget(pause_banner, area);
            }
        })?;
        Ok(())
    }

    /// Returns a color associated with the given tetromino.
    fn color_for(t: TetrominoType) -> Color {
        match t {
            TetrominoType::I => Color::Cyan,
            TetrominoType::O => Color::Yellow,
            TetrominoType::T => Color::Magenta,
            TetrominoType::S => Color::Green,
            TetrominoType::Z => Color::Red,
            TetrominoType::J => Color::Blue,
            TetrominoType::L => Color::Rgb(255, 165, 0),
        }
    }

    /// Toggles the pause state.
    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Loads a game state from a simple text file.
    /// File format (line by line):
    /// score: <u32>
    /// lines: <u32>
    /// level: <u32>
    /// current_piece: <char> <rotation> <x> <y>
    /// next_piece: <char> <rotation> <x> <y>
    /// hold_piece: none  OR  hold_piece: <char> <rotation> <x> <y>
    /// board:
    /// <BOARD_HEIGHT> lines with <BOARD_WIDTH> characters ('.' for empty)
    fn load_from_file(path: &str, tick_duration: Duration) -> Result<Self, Box<dyn std::error::Error>> {
        use std::io::BufRead;
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut score = 0;
        let mut lines_cleared = 0;
        let mut level = 1;
        let mut current_piece: Option<Piece> = None;
        let mut next_piece: Option<Piece> = None;
        let mut hold_piece: Option<Piece> = None;
        let mut board = [[None; BOARD_WIDTH]; BOARD_HEIGHT];
        let mut in_board = false;
        let mut board_row = 0;
        for line_res in reader.lines() {
            let line = line_res?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if in_board {
                if board_row < BOARD_HEIGHT {
                    let chars: Vec<char> = line.chars().collect();
                    for x in 0..BOARD_WIDTH {
                        let c = *chars.get(x).unwrap_or(&'.');
                        board[board_row][x] = match c {
                            '.' => None,
                            'I' => Some(TetrominoType::I),
                            'O' => Some(TetrominoType::O),
                            'T' => Some(TetrominoType::T),
                            'S' => Some(TetrominoType::S),
                            'Z' => Some(TetrominoType::Z),
                            'J' => Some(TetrominoType::J),
                            'L' => Some(TetrominoType::L),
                            _ => None,
                        };
                    }
                    board_row += 1;
                }
                continue;
            }
            let lower = line.to_lowercase();
            if lower.starts_with("score:") {
                if let Some(val) = line.split(':').nth(1) {
                    score = val.trim().parse().unwrap_or(0);
                }
            } else if lower.starts_with("lines:") {
                if let Some(val) = line.split(':').nth(1) {
                    lines_cleared = val.trim().parse().unwrap_or(0);
                }
            } else if lower.starts_with("level:") {
                if let Some(val) = line.split(':').nth(1) {
                    level = val.trim().parse().unwrap_or(1);
                }
            } else if lower.starts_with("current_piece:") {
                if let Some(data) = line.split(':').nth(1) {
                    let tokens: Vec<&str> = data.trim().split_whitespace().collect();
                    if tokens.len() >= 4 {
                        let tetro = parse_tetromino(tokens[0].chars().next().unwrap_or('T'));
                        let rotation = tokens[1].parse().unwrap_or(0);
                        let x = tokens[2].parse().unwrap_or((BOARD_WIDTH / 2) as i32);
                        let y = tokens[3].parse().unwrap_or(0);
                        current_piece = Some(Piece { tetromino: tetro, rotation, x, y });
                    }
                }
            } else if lower.starts_with("next_piece:") {
                if let Some(data) = line.split(':').nth(1) {
                    let tokens: Vec<&str> = data.trim().split_whitespace().collect();
                    if tokens.len() >= 4 {
                        let tetro = parse_tetromino(tokens[0].chars().next().unwrap_or('I'));
                        let rotation = tokens[1].parse().unwrap_or(0);
                        let x = tokens[2].parse().unwrap_or((BOARD_WIDTH / 2) as i32);
                        let y = tokens[3].parse().unwrap_or(0);
                        next_piece = Some(Piece { tetromino: tetro, rotation, x, y });
                    }
                }
            } else if lower.starts_with("hold_piece:") {
                if let Some(data) = line.split(':').nth(1) {
                    let data = data.trim();
                    if data.to_lowercase() == "none" {
                        hold_piece = None;
                    } else {
                        let tokens: Vec<&str> = data.split_whitespace().collect();
                        if tokens.len() >= 4 {
                            let tetro = parse_tetromino(tokens[0].chars().next().unwrap_or('I'));
                            let rotation = tokens[1].parse().unwrap_or(0);
                            let x = tokens[2].parse().unwrap_or((BOARD_WIDTH / 2) as i32);
                            let y = tokens[3].parse().unwrap_or(0);
                            hold_piece = Some(Piece { tetromino: tetro, rotation, x, y });
                        }
                    }
                }
            } else if lower.starts_with("board:") {
                in_board = true;
            }
        }
        let current_piece = current_piece.unwrap_or_else(|| Game::generate_piece());
        let next_piece = next_piece.unwrap_or_else(|| Game::generate_piece());
        Ok(Game {
            board,
            current_piece,
            next_piece,
            last_tick: Instant::now(),
            is_game_over: false,
            paused: false,
            score,
            lines_cleared,
            level,
            hold_piece,
            hold_used: false,
            start_time: Instant::now(),
            tick_duration,
        })
    }
}

/// Helper: returns the relative coordinates of a piece (assuming an origin of (0,0)).
fn get_relative_coords(piece: &Piece) -> Vec<(i32, i32)> {
    let temp = Piece {
        x: 0,
        y: 0,
        tetromino: piece.tetromino,
        rotation: piece.rotation,
    };
    get_piece_coords(&temp)
}

/// Creates a preview grid for the given piece.
fn create_preview(piece: Piece) -> Vec<Spans<'static>> {
    let preview_width = 6;
    let preview_height = 6;
    let shape = get_relative_coords(&piece);
    let min_x = shape.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x = shape.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let min_y = shape.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y = shape.iter().map(|(_, y)| *y).max().unwrap_or(0);
    let shape_width = max_x - min_x + 1;
    let shape_height = max_y - min_y + 1;
    let offset_x = (preview_width as i32 - shape_width) / 2 - min_x;
    let offset_y = (preview_height as i32 - shape_height) / 2 - min_y;

    let mut grid = vec![vec!["  "; preview_width]; preview_height];
    for (x, y) in shape {
        let px = (x + offset_x) as usize;
        let py = (y + offset_y) as usize;
        if px < preview_width && py < preview_height {
            grid[py][px] = "[]";
        }
    }

    let color = Game::color_for(piece.tetromino);
    grid.into_iter()
        .map(|row| {
            Spans::from(
                row.into_iter()
                    .map(|cell| {
                        if cell == "[]" {
                            Span::styled(cell, Style::default().bg(color))
                        } else {
                            Span::styled(cell, Style::default().bg(BOARD_BACKGROUND))
                        }
                    })
                    .collect::<Vec<Span>>(),
            )
        })
        .collect()
}

/// Creates an empty preview grid.
fn create_empty_preview() -> Vec<Spans<'static>> {
    let preview_width = 6;
    let preview_height = 6;
    let grid = vec![vec!["  "; preview_width]; preview_height];
    grid.into_iter()
        .map(|row| {
            Spans::from(
                row.into_iter()
                    .map(|cell| Span::styled(cell, Style::default().bg(BOARD_BACKGROUND)))
                    .collect::<Vec<Span>>(),
            )
        })
        .collect()
}

/// Returns a centered rectangle with the given percentage width and height.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Command-line arguments for key bindings and other options.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Key to move left.
    #[clap(long, default_value = "Left")]
    move_left: String,

    /// Key to move right.
    #[clap(long, default_value = "Right")]
    move_right: String,

    /// Key to move down.
    #[clap(long, default_value = "Down")]
    move_down: String,

    /// Key to rotate clockwise.
    #[clap(long, default_value = "Up")]
    rotate_cw: String,

    /// Key to rotate counterclockwise.
    #[clap(long, default_value = "z")]
    rotate_ccw: String,

    /// Key to hold the current piece.
    #[clap(long, default_value = "h")]
    hold: String,

    /// Key for hard drop.
    #[clap(long, default_value = "Space")]
    hard_drop: String,

    /// Key to pause/resume.
    #[clap(long, default_value = "p")]
    pause: String,

    /// Key to restart the game.
    #[clap(long, default_value = "r")]
    restart: String,

    /// Key to quit the game.
    #[clap(long, default_value = "q")]
    quit: String,

    /// Path to load key bindings from a config file.
    #[clap(long)]
    kb_file: Option<String>,

    /// Tick speed in milliseconds.
    #[clap(short, long, default_value_t = 500)]
    tick_speed: u64,

    /// Path to a saved game state.
    #[clap(short, long)]
    load_state: Option<String>,

    /// Path to create a sample save file.
    #[clap(long)]
    create_sample: Option<String>,
}

/// Holds key bindings as crossterm KeyCodes.
#[derive(Debug, Clone)]
struct KeyConfig {
    move_left: KeyCode,
    move_right: KeyCode,
    move_down: KeyCode,
    rotate_cw: KeyCode,
    rotate_ccw: KeyCode,
    hold: KeyCode,
    hard_drop: KeyCode,
    pause: KeyCode,
    restart: KeyCode,
    quit: KeyCode,
}

impl KeyConfig {
    fn from_args(args: &Args) -> Self {
        KeyConfig {
            move_left: parse_key(&args.move_left),
            move_right: parse_key(&args.move_right),
            move_down: parse_key(&args.move_down),
            rotate_cw: parse_key(&args.rotate_cw),
            rotate_ccw: parse_key(&args.rotate_ccw),
            hold: parse_key(&args.hold),
            hard_drop: parse_key(&args.hard_drop),
            pause: parse_key(&args.pause),
            restart: parse_key(&args.restart),
            quit: parse_key(&args.quit),
        }
    }
    fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use std::io::BufRead;
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        let mut config_map = std::collections::HashMap::new();
        for line_res in reader.lines() {
            let line = line_res?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                config_map.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        Ok(KeyConfig {
            move_left: parse_key(config_map.get("move_left").unwrap_or(&"Left".to_string())),
            move_right: parse_key(config_map.get("move_right").unwrap_or(&"Right".to_string())),
            move_down: parse_key(config_map.get("move_down").unwrap_or(&"Down".to_string())),
            rotate_cw: parse_key(config_map.get("rotate_cw").unwrap_or(&"Up".to_string())),
            rotate_ccw: parse_key(config_map.get("rotate_ccw").unwrap_or(&"z".to_string())),
            hold: parse_key(config_map.get("hold").unwrap_or(&"h".to_string())),
            hard_drop: parse_key(config_map.get("hard_drop").unwrap_or(&"Space".to_string())),
            pause: parse_key(config_map.get("pause").unwrap_or(&"p".to_string())),
            restart: parse_key(config_map.get("restart").unwrap_or(&"r".to_string())),
            quit: parse_key(config_map.get("quit").unwrap_or(&"q".to_string())),
        })
    }
}

/// Parses a string to a KeyCode. Recognizes "left", "right", "up", "down", "space", or a single character.
fn parse_key(s: &str) -> KeyCode {
    match s.to_lowercase().as_str() {
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "space" => KeyCode::Char(' '),
        other if other.len() == 1 => KeyCode::Char(other.chars().next().unwrap()),
        other => KeyCode::Char(other.chars().next().unwrap()),
    }
}

/// Returns a human-readable name for the given KeyCode.
fn key_name(key: KeyCode) -> String {
    match key {
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        other => format!("{:?}", other),
    }
}

/// Writes a sample save state file with default values.
fn write_sample_save_state(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "score: {}", 0)?;
    writeln!(file, "lines: {}", 0)?;
    writeln!(file, "level: {}", 1)?;
    // Example values: current_piece: T 0 <center> 0, next_piece: I 0 <center> 0
    writeln!(file, "current_piece: {} {} {} {}", 'T', 0, BOARD_WIDTH / 2, 0)?;
    writeln!(file, "next_piece: {} {} {} {}", 'I', 0, BOARD_WIDTH / 2, 0)?;
    writeln!(file, "hold_piece: none")?;
    writeln!(file, "board:")?;
    for _ in 0..BOARD_HEIGHT {
        writeln!(file, "{}", ".".repeat(BOARD_WIDTH))?;
    }
    Ok(())
}

/// Parses a character into a TetrominoType.
fn parse_tetromino(c: char) -> TetrominoType {
    match c {
        'I' => TetrominoType::I,
        'O' => TetrominoType::O,
        'T' => TetrominoType::T,
        'S' => TetrominoType::S,
        'Z' => TetrominoType::Z,
        'J' => TetrominoType::J,
        'L' => TetrominoType::L,
        _ => TetrominoType::T,
    }
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    // If the sample save file creation flag is used, create the file and exit.
    if let Some(ref sample_path) = args.create_sample {
        match write_sample_save_state(sample_path) {
            Ok(()) => println!("Sample save file created at '{}'.", sample_path),
            Err(e) => eprintln!("Error creating sample save file: {}", e),
        }
        return Ok(());
    }

    //
    // Load key bindings configuration
    //

    // Find the OS specific config dir
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("tuitris").join("keybinds.txt");
    let config_key_path = config_path.to_str().unwrap_or("");
    // An empty KeyConfig which will be filled in 
    let key_config: KeyConfig;
    // TODO:
    // Keybinds are currently checked in this order:
    // 1. From the file specified by `--kb-file` argument (if provided)
    // 2. From the default config file at ~/.config/tuitris/keybinds.txt (if it exists)
    // 3. From command-line arguments (default values if not provided)
    // Instead they should be checked in this order:
    // 1. Load default keybinds
    // 2. Override with keybinds from the config file (if it exists)
    // 3. Override with keybinds from command-line arguments (if provided)
    //
    // Try to load from `args.kb_file`
    if let Some(ref kb_file) = args.kb_file {
        match KeyConfig::from_file(kb_file) {
            Ok(config) => key_config = config,
            Err(e) => {
                eprintln!("Error loading key bindings from file: {}", e);
                key_config = KeyConfig::from_args(&args);
            }
        }
    // Try to load from `config_path/keybinds.txt`
    } else if std::path::Path::new(config_key_path).exists() {
        match KeyConfig::from_file(config_key_path) {
            Ok(config) => key_config = config,
            Err(e) => {
                eprintln!("Error loading key bindings from config file: {}", e);
                key_config = KeyConfig::from_args(&args);
            }
        }
    // Fallback to command-line arguments
    } else {
        key_config = KeyConfig::from_args(&args);
    }

    let tick_duration = Duration::from_millis(args.tick_speed);
    let mut game = if let Some(ref path) = args.load_state {
        match Game::load_from_file(path, tick_duration) {
            Ok(loaded_game) => loaded_game,
            Err(e) => {
                eprintln!("Error loading save state: {}", e);
                Game::new(tick_duration)
            }
        }
    } else {
        Game::new(tick_duration)
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    'mainloop: loop {
        if game.is_game_over {
            terminal.draw(|f| {
                let area = centered_rect(50, 20, f.size());
                let game_over_text = Paragraph::new(format!(
                    "GAME OVER\nPress ({} : Restart) or ({} : Quit)",
                    key_name(key_config.restart),
                    key_name(key_config.quit)
                ))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Game Over"));
                f.render_widget(game_over_text, area);
            })?;
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key_event.code {
                        code if code == key_config.quit => break 'mainloop,
                        code if code == key_config.restart => game = Game::new(tick_duration),
                        _ => {}
                    }
                }
            }
        } else {
            game.update();
            game.draw(&mut terminal, &key_config)?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key_event.code {
                        code if code == key_config.quit => break 'mainloop,
                        code if code == key_config.pause => game.toggle_pause(),
                        code if code == key_config.restart => game = Game::new(tick_duration),
                        code if code == key_config.hold => game.hold_current_piece(),
                        code if code == key_config.hard_drop => game.hard_drop(),
                        code if code == key_config.move_left => game.move_piece(-1, 0),
                        code if code == key_config.move_right => game.move_piece(1, 0),
                        code if code == key_config.move_down => game.move_piece(0, 1),
                        code if code == key_config.rotate_cw => game.rotate_piece(RotationDirection::CW),
                        code if code == key_config.rotate_ccw => game.rotate_piece(RotationDirection::CCW),
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
