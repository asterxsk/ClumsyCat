use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use std::io::Write;
use std::sync::mpsc::Receiver;
use vte::{Parser, Perform};

const SCROLLBACK_LIMIT: usize = 1000;

#[derive(Clone, Copy)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::White,
            bg: Color::Black,
            bold: false,
        }
    }
}

pub struct TerminalBuffer {
    cells: Vec<Vec<TerminalCell>>,
    cursor: (u16, u16),
    width: u16,
    height: u16,
    parser: Parser,
    scrollback: Vec<Vec<TerminalCell>>,
    current_fg: Color,
    current_bg: Color,
    current_bold: bool,
}

impl TerminalBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![vec![TerminalCell::default(); width as usize]; height as usize];
        let scrollback = Vec::new();

        Self {
            cells,
            cursor: (0, 0),
            width,
            height,
            parser: Parser::new(),
            scrollback,
            current_fg: Color::White,
            current_bg: Color::Black,
            current_bold: false,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if width == self.width && height == self.height {
            return;
        }

        self.width = width;
        self.height = height;
        self.cells = vec![vec![TerminalCell::default(); width as usize]; height as usize];

        if self.cursor.0 >= width {
            self.cursor.0 = width.saturating_sub(1);
        }
        if self.cursor.1 >= height {
            self.cursor.1 = height.saturating_sub(1);
        }
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            let mut parser = std::mem::replace(&mut self.parser, Parser::new());
            parser.advance(self, byte);
            self.parser = parser;
        }
    }

    fn scroll_up(&mut self) {
        if let Some(top_line) = self.cells.first().cloned() {
            self.scrollback.push(top_line);
            if self.scrollback.len() > SCROLLBACK_LIMIT {
                self.scrollback.remove(0);
            }
        }

        self.cells.remove(0);
        self.cells
            .push(vec![TerminalCell::default(); self.width as usize]);
    }

    fn set_cell(&mut self, x: u16, y: u16, ch: char) {
        if let Some(row) = self.cells.get_mut(y as usize) {
            if let Some(cell) = row.get_mut(x as usize) {
                cell.ch = ch;
                cell.fg = self.current_fg;
                cell.bg = self.current_bg;
                cell.bold = self.current_bold;
            }
        }
    }

    fn advance_cursor(&mut self) {
        self.cursor.0 += 1;
        if self.cursor.0 >= self.width {
            self.cursor.0 = 0;
            self.cursor.1 += 1;
            if self.cursor.1 >= self.height {
                self.cursor.1 = self.height - 1;
                self.scroll_up();
            }
        }
    }
}

impl Perform for TerminalBuffer {
    fn print(&mut self, c: char) {
        if c == '\n' {
            self.cursor.0 = 0;
            self.cursor.1 += 1;
            if self.cursor.1 >= self.height {
                self.cursor.1 = self.height - 1;
                self.scroll_up();
            }
        } else if c == '\r' {
            self.cursor.0 = 0;
        } else if c.is_control() {
            // Skip other control characters
        } else {
            self.set_cell(self.cursor.0, self.cursor.1, c);
            self.advance_cursor();
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.print('\n'),
            b'\r' => self.print('\r'),
            b'\t' => {
                let spaces = 8 - (self.cursor.0 % 8);
                for _ in 0..spaces {
                    if self.cursor.0 < self.width {
                        self.print(' ');
                    }
                }
            }
            b'\x08' => {
                // Backspace
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        c: char,
    ) {
        match c {
            'A' => {
                // Cursor up
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1);
                self.cursor.1 = self.cursor.1.saturating_sub(n);
            }
            'B' => {
                // Cursor down
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1);
                self.cursor.1 = (self.cursor.1 + n).min(self.height - 1);
            }
            'C' => {
                // Cursor right
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1);
                self.cursor.0 = (self.cursor.0 + n).min(self.width - 1);
            }
            'D' => {
                // Cursor left
                let n = params.iter().next().map(|p| p[0]).unwrap_or(1).max(1);
                self.cursor.0 = self.cursor.0.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor position
                let mut iter = params.iter();
                let row = iter.next().map(|p| p[0]).unwrap_or(1).max(1) - 1;
                let col = iter.next().map(|p| p[0]).unwrap_or(1).max(1) - 1;
                self.cursor.0 = col.min(self.width - 1);
                self.cursor.1 = row.min(self.height - 1);
            }
            'J' => {
                // Clear screen
                let n = params.iter().next().map(|p| p[0]).unwrap_or(0);
                match n {
                    0 => {
                        // Clear from cursor to end
                        for y in self.cursor.1..self.height {
                            let start_x = if y == self.cursor.1 { self.cursor.0 } else { 0 };
                            for x in start_x..self.width {
                                self.set_cell(x, y, ' ');
                            }
                        }
                    }
                    1 => {
                        // Clear from start to cursor
                        for y in 0..=self.cursor.1 {
                            let end_x = if y == self.cursor.1 {
                                self.cursor.0
                            } else {
                                self.width - 1
                            };
                            for x in 0..=end_x {
                                self.set_cell(x, y, ' ');
                            }
                        }
                    }
                    2 => {
                        // Clear entire screen
                        self.cells = vec![
                            vec![TerminalCell::default(); self.width as usize];
                            self.height as usize
                        ];
                        self.cursor = (0, 0);
                    }
                    _ => {}
                }
            }
            'K' => {
                // Clear line
                let n = params.iter().next().map(|p| p[0]).unwrap_or(0);
                let y = self.cursor.1;
                match n {
                    0 => {
                        // Clear from cursor to end of line
                        for x in self.cursor.0..self.width {
                            self.set_cell(x, y, ' ');
                        }
                    }
                    1 => {
                        // Clear from start of line to cursor
                        for x in 0..=self.cursor.0 {
                            self.set_cell(x, y, ' ');
                        }
                    }
                    2 => {
                        // Clear entire line
                        for x in 0..self.width {
                            self.set_cell(x, y, ' ');
                        }
                    }
                    _ => {}
                }
            }
            'm' => {
                // Set graphic rendition (colors, bold, etc.)
                if params.is_empty() {
                    // Reset to defaults
                    self.current_fg = Color::White;
                    self.current_bg = Color::Black;
                    self.current_bold = false;
                } else {
                    for param_slice in params.iter() {
                        for &param in param_slice {
                            match param {
                                0 => {
                                    self.current_fg = Color::White;
                                    self.current_bg = Color::Black;
                                    self.current_bold = false;
                                }
                                1 => self.current_bold = true,
                                22 => self.current_bold = false,
                                30 => self.current_fg = Color::Black,
                                31 => self.current_fg = Color::Red,
                                32 => self.current_fg = Color::Green,
                                33 => self.current_fg = Color::Yellow,
                                34 => self.current_fg = Color::Blue,
                                35 => self.current_fg = Color::Magenta,
                                36 => self.current_fg = Color::Cyan,
                                37 => self.current_fg = Color::White,
                                39 => self.current_fg = Color::White, // Default
                                40 => self.current_bg = Color::Black,
                                41 => self.current_bg = Color::Red,
                                42 => self.current_bg = Color::Green,
                                43 => self.current_bg = Color::Yellow,
                                44 => self.current_bg = Color::Blue,
                                45 => self.current_bg = Color::Magenta,
                                46 => self.current_bg = Color::Cyan,
                                47 => self.current_bg = Color::White,
                                49 => self.current_bg = Color::Black, // Default
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

impl Widget for &TerminalBuffer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in 0..area.height.min(self.height) {
            for x in 0..area.width.min(self.width) {
                if let Some(row) = self.cells.get(y as usize) {
                    if let Some(cell) = row.get(x as usize) {
                        let style = Style::default().fg(cell.fg).bg(cell.bg);

                        let style = if cell.bold {
                            style.add_modifier(ratatui::style::Modifier::BOLD)
                        } else {
                            style
                        };

                        if let Some(buf_cell) = buf.cell_mut((area.x + x, area.y + y)) {
                            buf_cell.set_char(cell.ch).set_style(style);
                        }
                    }
                }
            }
        }
    }
}

pub struct ProxyTerminal {
    pub buffer: TerminalBuffer,
    pub output_rx: Receiver<Vec<u8>>,
    pub writer: Box<dyn Write + Send>,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub visible: bool,
    pub focused: bool,
}

impl ProxyTerminal {
    pub fn new(
        width: u16,
        height: u16,
        output_rx: Receiver<Vec<u8>>,
        writer: Box<dyn Write + Send>,
        child: Box<dyn portable_pty::Child + Send + Sync>,
    ) -> Self {
        Self {
            buffer: TerminalBuffer::new(width, height),
            output_rx,
            writer,
            child,
            visible: false,
            focused: false,
        }
    }

    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Ok(bytes) = self.output_rx.try_recv() {
            self.buffer.process_bytes(&bytes);
        }
        Ok(())
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer.resize(width, height);
    }

    pub fn write_input(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Still running
            Err(_) => false,      // Error checking status, assume dead
        }
    }
}
