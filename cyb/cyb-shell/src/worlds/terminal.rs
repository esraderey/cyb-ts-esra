use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::sync::{Arc, Mutex};
use std::thread;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use teletypewriter::ProcessReadWrite;

use super::WorldState;

const TERM_COLS: usize = 120;
const TERM_ROWS: usize = 40;

pub struct TerminalWorldPlugin;

#[derive(Resource, Default)]
struct TerminalCreated(bool);

impl Plugin for TerminalWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerminalCreated>()
            .add_systems(OnExit(WorldState::Terminal), destroy_terminal)
            .add_systems(
                Update,
                ensure_terminal_created.run_if(in_state(WorldState::Terminal)),
            )
            .add_systems(
                Update,
                render_terminal.run_if(in_state(WorldState::Terminal)),
            )
            .add_systems(
                Update,
                forward_keyboard_input.run_if(in_state(WorldState::Terminal)),
            );
    }
}

/// Simple terminal grid cell
#[derive(Clone, Copy)]
struct Cell {
    c: char,
}

impl Default for Cell {
    fn default() -> Self {
        Self { c: ' ' }
    }
}

/// Terminal grid state shared between reader thread and Bevy systems
struct TermGrid {
    cells: Vec<Vec<Cell>>,
    cursor_row: usize,
    cursor_col: usize,
    cols: usize,
    rows: usize,
}

impl TermGrid {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cells: vec![vec![Cell::default(); cols]; rows],
            cursor_row: 0,
            cursor_col: 0,
            cols,
            rows,
        }
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.newline();
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.cells[self.cursor_row][self.cursor_col].c = c;
            self.cursor_col += 1;
        }
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        if self.cursor_row >= self.rows {
            // Scroll up
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

    fn to_string(&self) -> String {
        let mut s = String::with_capacity((self.cols + 1) * self.rows);
        for (i, row) in self.cells.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            let line: String = row.iter().map(|c| c.c).collect();
            s.push_str(line.trim_end());
        }
        s
    }
}

/// VTE performer that updates our grid
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
                // Cursor up
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_row = self.grid.cursor_row.saturating_sub(n);
            }
            'B' => {
                // Cursor down
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_row = (self.grid.cursor_row + n).min(self.grid.rows - 1);
            }
            'C' => {
                // Cursor forward
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_col = (self.grid.cursor_col + n).min(self.grid.cols - 1);
            }
            'D' => {
                // Cursor backward
                let n = if first == 0 { 1 } else { first as usize };
                self.grid.cursor_col = self.grid.cursor_col.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor position
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first().copied()).unwrap_or(1) as usize;
                let col = iter.next().and_then(|p| p.first().copied()).unwrap_or(1) as usize;
                self.grid.cursor_row = (row.saturating_sub(1)).min(self.grid.rows - 1);
                self.grid.cursor_col = (col.saturating_sub(1)).min(self.grid.cols - 1);
            }
            'J' => {
                // Erase display
                match first {
                    2 | 3 => self.grid.clear_screen(),
                    0 => {
                        // Clear from cursor to end
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
                // Erase line
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
                // SGR (colors/attributes) — ignore for now
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

struct TerminalState {
    grid: Arc<Mutex<TermGrid>>,
    writer: Arc<Mutex<File>>,
    _reader_handle: thread::JoinHandle<()>,
    _pty: teletypewriter::Pty,
}

fn ensure_terminal_created(world: &mut World) {
    if world.resource::<TerminalCreated>().0 {
        return;
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

    let mut pty = teletypewriter::create_pty(&shell, TERM_COLS as u16, TERM_ROWS as u16);

    // Duplicate file descriptors for separate reader/writer handles
    let reader_fd = pty.reader().as_raw_fd();
    let writer_fd = pty.writer().as_raw_fd();

    let reader_file = unsafe { File::from_raw_fd(libc::dup(reader_fd)) };
    let writer_file = unsafe { File::from_raw_fd(libc::dup(writer_fd)) };

    let grid = Arc::new(Mutex::new(TermGrid::new(TERM_COLS, TERM_ROWS)));
    let grid_clone = Arc::clone(&grid);
    let writer = Arc::new(Mutex::new(writer_file));

    // Spawn reader thread
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
                Err(_) => break,
            }
        }
    });

    world.insert_non_send_resource(TerminalState {
        grid,
        writer,
        _reader_handle: reader_handle,
        _pty: pty,
    });

    info!("Terminal world created ({}x{}) shell={}", TERM_COLS, TERM_ROWS, shell);
    world.resource_mut::<TerminalCreated>().0 = true;
}

fn render_terminal(
    terminal_state: Option<NonSend<TerminalState>>,
    mut contexts: EguiContexts,
) {
    let Some(terminal_state) = terminal_state else { return };
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let text = {
        let grid = terminal_state.grid.lock().unwrap();
        grid.to_string()
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(26, 26, 46)))
        .show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .lock_focus(false),
                );
            });
        });
}

fn forward_keyboard_input(
    terminal_state: Option<NonSend<TerminalState>>,
    mut key_events: MessageReader<KeyboardInput>,
) {
    let Some(terminal_state) = terminal_state else { return };

    for event in key_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        let bytes: Option<Vec<u8>> = match &event.logical_key {
            Key::Character(c) => Some(c.as_str().as_bytes().to_vec()),
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
            if let Ok(mut writer) = terminal_state.writer.lock() {
                let _ = writer.write_all(&bytes);
                let _ = writer.flush();
            }
        }
    }
}

fn destroy_terminal(world: &mut World) {
    if let Some(_state) = world.remove_non_send_resource::<TerminalState>() {
        // Reader thread will exit when PTY is dropped
        drop(_state);
    }
    world.resource_mut::<TerminalCreated>().0 = false;
    info!("Terminal world destroyed");
}
