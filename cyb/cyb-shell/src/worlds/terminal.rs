use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::sync::{Arc, Mutex};
use std::thread;

use bevy::color::LinearRgba;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy_ascii_terminal::{color, Terminal, TerminalCamera};
use teletypewriter::ProcessReadWrite;

use super::WorldState;

const TERM_COLS: usize = 120;
const TERM_ROWS: usize = 40;

pub struct TerminalWorldPlugin;

#[derive(Component)]
struct TerminalMarker;

impl Plugin for TerminalWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerminalStateRes>()
            .add_systems(OnEnter(WorldState::Terminal), setup_terminal)
            .add_systems(OnExit(WorldState::Terminal), destroy_terminal)
            .add_systems(
                Update,
                sync_grid_to_terminal.run_if(in_state(WorldState::Terminal)),
            )
            .add_systems(
                Update,
                forward_keyboard_input.run_if(in_state(WorldState::Terminal)),
            );
    }
}

// --- Terminal cell with ANSI colors ---

#[derive(Clone, Copy)]
struct Cell {
    c: char,
    fg: LinearRgba,
    bg: LinearRgba,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: color::WHITE,
            bg: color::BLACK,
        }
    }
}

// --- SGR state for tracking current text attributes ---

#[derive(Clone)]
struct SgrState {
    fg: LinearRgba,
    bg: LinearRgba,
    bold: bool,
}

impl Default for SgrState {
    fn default() -> Self {
        Self {
            fg: color::WHITE,
            bg: color::BLACK,
            bold: false,
        }
    }
}

// --- Terminal grid shared between reader thread and Bevy ---

struct TermGrid {
    cells: Vec<Vec<Cell>>,
    cursor_row: usize,
    cursor_col: usize,
    cols: usize,
    rows: usize,
    sgr: SgrState,
}

impl TermGrid {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cells: vec![vec![Cell::default(); cols]; rows],
            cursor_row: 0,
            cursor_col: 0,
            cols,
            rows,
            sgr: SgrState::default(),
        }
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.newline();
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            // bevy_ascii_terminal only supports ASCII glyphs in its font map
            // Replace non-printable-ASCII with space to avoid panic
            let safe_c = if c.is_ascii_graphic() || c == ' ' { c } else { ' ' };
            self.cells[self.cursor_row][self.cursor_col] = Cell {
                c: safe_c,
                fg: self.sgr.fg,
                bg: self.sgr.bg,
            };
            self.cursor_col += 1;
        }
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        if self.cursor_row >= self.rows {
            self.cells.remove(0);
            self.cells.push(vec![Cell::default(); self.cols]);
            self.cursor_row = self.rows - 1;
        }
    }

    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn clear_line_from_cursor(&mut self) {
        for col in self.cursor_col..self.cols {
            self.cells[self.cursor_row][col] = Cell::default();
        }
    }

    fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }
}

// --- ANSI 256-color palette ---

fn ansi_standard_color(idx: u8, bold: bool) -> LinearRgba {
    match idx {
        0 => if bold { LinearRgba::new(0.33, 0.33, 0.33, 1.0) } else { color::BLACK },
        1 => if bold { LinearRgba::new(1.0, 0.33, 0.33, 1.0) } else { LinearRgba::new(0.8, 0.0, 0.0, 1.0) },
        2 => if bold { LinearRgba::new(0.33, 1.0, 0.33, 1.0) } else { LinearRgba::new(0.0, 0.8, 0.0, 1.0) },
        3 => if bold { color::YELLOW } else { LinearRgba::new(0.8, 0.8, 0.0, 1.0) },
        4 => if bold { LinearRgba::new(0.33, 0.33, 1.0, 1.0) } else { LinearRgba::new(0.0, 0.0, 0.8, 1.0) },
        5 => if bold { LinearRgba::new(1.0, 0.33, 1.0, 1.0) } else { LinearRgba::new(0.8, 0.0, 0.8, 1.0) },
        6 => if bold { LinearRgba::new(0.33, 1.0, 1.0, 1.0) } else { LinearRgba::new(0.0, 0.8, 0.8, 1.0) },
        7 => if bold { color::WHITE } else { LinearRgba::new(0.75, 0.75, 0.75, 1.0) },
        _ => color::WHITE,
    }
}

fn ansi_256_color(idx: u8) -> LinearRgba {
    match idx {
        0..=7 => ansi_standard_color(idx, false),
        8..=15 => ansi_standard_color(idx - 8, true),
        16..=231 => {
            // 6x6x6 color cube
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            let to_f = |v: u8| if v == 0 { 0.0 } else { (55.0 + 40.0 * v as f32) / 255.0 };
            LinearRgba::new(to_f(r), to_f(g), to_f(b), 1.0)
        }
        232..=255 => {
            // Grayscale ramp
            let v = (8 + 10 * (idx - 232) as u32) as f32 / 255.0;
            LinearRgba::new(v, v, v, 1.0)
        }
    }
}

// --- VTE performer ---

struct GridPerformer<'a> {
    grid: &'a mut TermGrid,
}

impl vte::Perform for GridPerformer<'_> {
    fn print(&mut self, c: char) {
        self.grid.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.grid.newline(),
            b'\r' => self.grid.carriage_return(),
            b'\x08' => self.grid.backspace(),
            b'\t' => {
                let next_tab = (self.grid.cursor_col / 8 + 1) * 8;
                self.grid.cursor_col = next_tab.min(self.grid.cols - 1);
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let first = params.iter().next().and_then(|p| p.first().copied()).unwrap_or(0);
        match action {
            'A' => {
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_row = self.grid.cursor_row.saturating_sub(n);
            }
            'B' => {
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_row = (self.grid.cursor_row + n).min(self.grid.rows - 1);
            }
            'C' => {
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_col = (self.grid.cursor_col + n).min(self.grid.cols - 1);
            }
            'D' => {
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_col = self.grid.cursor_col.saturating_sub(n);
            }
            'H' | 'f' => {
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first().copied()).unwrap_or(1) as usize;
                let col = iter.next().and_then(|p| p.first().copied()).unwrap_or(1) as usize;
                self.grid.cursor_row = (row.saturating_sub(1)).min(self.grid.rows - 1);
                self.grid.cursor_col = (col.saturating_sub(1)).min(self.grid.cols - 1);
            }
            'J' => {
                match first {
                    2 | 3 => self.grid.clear_screen(),
                    0 => {
                        self.grid.clear_line_from_cursor();
                        for row in (self.grid.cursor_row + 1)..self.grid.rows {
                            for col in 0..self.grid.cols {
                                self.grid.cells[row][col] = Cell::default();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                match first {
                    0 => self.grid.clear_line_from_cursor(),
                    2 => {
                        let row = self.grid.cursor_row;
                        for col in 0..self.grid.cols {
                            self.grid.cells[row][col] = Cell::default();
                        }
                    }
                    _ => {}
                }
            }
            'm' => {
                // SGR — Set Graphic Rendition
                self.process_sgr(params);
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

impl GridPerformer<'_> {
    fn process_sgr(&mut self, params: &vte::Params) {
        let mut iter = params.iter();

        // Handle empty params (reset)
        let mut had_params = false;

        while let Some(param) = iter.next() {
            had_params = true;
            let code = param.first().copied().unwrap_or(0);

            match code {
                0 => {
                    self.grid.sgr = SgrState::default();
                }
                1 => {
                    self.grid.sgr.bold = true;
                }
                22 => {
                    self.grid.sgr.bold = false;
                }
                // Standard foreground colors
                30..=37 => {
                    self.grid.sgr.fg = ansi_standard_color((code - 30) as u8, self.grid.sgr.bold);
                }
                // Standard background colors
                40..=47 => {
                    self.grid.sgr.bg = ansi_standard_color((code - 40) as u8, false);
                }
                // Default foreground
                39 => {
                    self.grid.sgr.fg = color::WHITE;
                }
                // Default background
                49 => {
                    self.grid.sgr.bg = color::BLACK;
                }
                // Bright foreground colors
                90..=97 => {
                    self.grid.sgr.fg = ansi_standard_color((code - 90) as u8, true);
                }
                // Bright background colors
                100..=107 => {
                    self.grid.sgr.bg = ansi_standard_color((code - 100) as u8, true);
                }
                // Extended foreground: 38;5;N (256-color) or 38;2;R;G;B (truecolor)
                38 => {
                    if let Some(sub) = iter.next() {
                        match sub.first().copied().unwrap_or(0) {
                            5 => {
                                if let Some(idx) = iter.next() {
                                    self.grid.sgr.fg = ansi_256_color(idx.first().copied().unwrap_or(0) as u8);
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as f32 / 255.0;
                                let g = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as f32 / 255.0;
                                let b = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as f32 / 255.0;
                                self.grid.sgr.fg = LinearRgba::new(r, g, b, 1.0);
                            }
                            _ => {}
                        }
                    }
                }
                // Extended background: 48;5;N or 48;2;R;G;B
                48 => {
                    if let Some(sub) = iter.next() {
                        match sub.first().copied().unwrap_or(0) {
                            5 => {
                                if let Some(idx) = iter.next() {
                                    self.grid.sgr.bg = ansi_256_color(idx.first().copied().unwrap_or(0) as u8);
                                }
                            }
                            2 => {
                                let r = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as f32 / 255.0;
                                let g = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as f32 / 255.0;
                                let b = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as f32 / 255.0;
                                self.grid.sgr.bg = LinearRgba::new(r, g, b, 1.0);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if !had_params {
            // CSI m with no params = reset
            self.grid.sgr = SgrState::default();
        }
    }
}

// --- Bevy resources & systems ---

struct TerminalState {
    grid: Arc<Mutex<TermGrid>>,
    writer: Arc<Mutex<File>>,
    _reader_handle: thread::JoinHandle<()>,
    _pty: teletypewriter::Pty,
}

fn setup_terminal(mut commands: Commands, mut state_res: ResMut<TerminalStateRes>) {
    let cols = TERM_COLS;
    let rows = TERM_ROWS;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

    // Shell needs TERM to render prompt and handle input properly
    unsafe { std::env::set_var("TERM", "xterm-256color"); }

    let mut pty = teletypewriter::create_pty(&shell, cols as u16, rows as u16);

    let reader_fd = pty.reader().as_raw_fd();
    let writer_fd = pty.writer().as_raw_fd();

    let reader_file = unsafe { File::from_raw_fd(libc::dup(reader_fd)) };
    let writer_file = unsafe { File::from_raw_fd(libc::dup(writer_fd)) };

    // teletypewriter sets fd to non-blocking — we need blocking for our reader thread
    let dup_reader_fd = reader_file.as_raw_fd();
    unsafe {
        let flags = libc::fcntl(dup_reader_fd, libc::F_GETFL);
        libc::fcntl(dup_reader_fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
    }

    let grid = Arc::new(Mutex::new(TermGrid::new(cols, rows)));
    let grid_clone = Arc::clone(&grid);
    let writer = Arc::new(Mutex::new(writer_file));

    let reader_handle = thread::spawn(move || {
        let mut reader = reader_file;
        let mut parser = vte::Parser::new();
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let mut grid = grid_clone.lock().unwrap();
                    let mut performer = GridPerformer { grid: &mut grid };
                    for byte in &buf[..n] {
                        parser.advance(&mut performer, *byte);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(_) => break,
            }
        }
    });

    // Spawn bevy_ascii_terminal
    commands.spawn((
        Terminal::new([cols as u32, rows as u32]),
        TerminalMarker,
    ));
    commands.spawn((TerminalCamera::new(), TerminalMarker));

    state_res.state = Some(TerminalState {
        grid,
        writer,
        _reader_handle: reader_handle,
        _pty: pty,
    });

    info!("Terminal world created ({}x{}) shell={}", cols, rows, shell);
}

#[derive(Resource, Default)]
struct TerminalStateRes {
    state: Option<TerminalState>,
}

fn sync_grid_to_terminal(
    state_res: Res<TerminalStateRes>,
    mut term_query: Query<&mut Terminal, With<TerminalMarker>>,
) {
    let Some(ref state) = state_res.state else { return };
    let Ok(mut terminal) = term_query.single_mut() else { return };

    let grid = state.grid.lock().unwrap();

    let term_size = terminal.size();
    let display_cols = (grid.cols as u32).min(term_size.x) as usize;
    let display_rows = (grid.rows as u32).min(term_size.y) as usize;

    for row in 0..display_rows {
        let y = (display_rows - 1 - row) as i32;
        for col in 0..display_cols {
            let cell = &grid.cells[row][col];
            let tile = terminal.tile_mut([col as i32, y]);
            tile.glyph = cell.c;
            tile.fg_color = cell.fg;
            tile.bg_color = cell.bg;
        }
    }

    // Show cursor as inverse block
    let crow = grid.cursor_row;
    let ccol = grid.cursor_col;
    if crow < display_rows && ccol < display_cols {
        let y = (display_rows - 1 - crow) as i32;
        let tile = terminal.tile_mut([ccol as i32, y]);
        let tmp = tile.fg_color;
        tile.fg_color = tile.bg_color;
        tile.bg_color = tmp;
    }
}

fn forward_keyboard_input(
    state_res: Res<TerminalStateRes>,
    mut key_events: MessageReader<KeyboardInput>,
) {
    let Some(ref state) = state_res.state else { return };

    for event in key_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        debug!("Terminal key: {:?}", event.logical_key);

        let bytes: Option<Vec<u8>> = match &event.logical_key {
            Key::Character(c) => {
                let s = c.as_str();
                // Handle Ctrl+C, Ctrl+D, etc
                if s.len() == 1 {
                    let ch = s.bytes().next().unwrap();
                    // Characters below 0x20 are already control chars from Ctrl+key
                    Some(vec![ch])
                } else {
                    Some(s.as_bytes().to_vec())
                }
            }
            Key::Enter => Some(b"\r".to_vec()),
            Key::Backspace => Some(b"\x7f".to_vec()),
            Key::Tab => Some(b"\t".to_vec()),
            Key::Escape => Some(b"\x1b".to_vec()),
            Key::Space => Some(b" ".to_vec()),
            Key::ArrowUp => Some(b"\x1b[A".to_vec()),
            Key::ArrowDown => Some(b"\x1b[B".to_vec()),
            Key::ArrowRight => Some(b"\x1b[C".to_vec()),
            Key::ArrowLeft => Some(b"\x1b[D".to_vec()),
            Key::Home => Some(b"\x1b[H".to_vec()),
            Key::End => Some(b"\x1b[F".to_vec()),
            Key::Delete => Some(b"\x1b[3~".to_vec()),
            _ => None,
        };

        if let Some(bytes) = bytes {
            if let Ok(mut writer) = state.writer.lock() {
                let _ = writer.write_all(&bytes);
                let _ = writer.flush();
            }
        }
    }
}

fn destroy_terminal(
    mut commands: Commands,
    mut state_res: ResMut<TerminalStateRes>,
    query: Query<Entity, With<TerminalMarker>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    state_res.state = None;
    info!("Terminal world destroyed");
}
