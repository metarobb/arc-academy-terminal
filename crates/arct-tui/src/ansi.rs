//! ANSI escape code parsing for colored output

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use vte::{Params, Parser, Perform};

/// Parse ANSI escape codes and convert to styled spans
pub fn parse_ansi(text: &str) -> Vec<Line<'static>> {
    let mut parser = AnsiParser::new();
    let mut vte_parser = Parser::new();

    for byte in text.bytes() {
        vte_parser.advance(&mut parser, byte);
    }

    parser.finish()
}

struct AnsiParser {
    current_style: Style,
    current_text: String,
    lines: Vec<Line<'static>>,
    current_line_spans: Vec<Span<'static>>,
}

impl AnsiParser {
    fn new() -> Self {
        Self {
            current_style: Style::default(),
            current_text: String::new(),
            lines: Vec::new(),
            current_line_spans: Vec::new(),
        }
    }

    fn finish(mut self) -> Vec<Line<'static>> {
        self.flush_text();
        if !self.current_line_spans.is_empty() {
            self.lines.push(Line::from(self.current_line_spans));
        }
        self.lines
    }

    fn flush_text(&mut self) {
        if !self.current_text.is_empty() {
            let text = std::mem::take(&mut self.current_text);
            self.current_line_spans
                .push(Span::styled(text, self.current_style));
        }
    }

    fn newline(&mut self) {
        self.flush_text();
        let spans = std::mem::take(&mut self.current_line_spans);
        self.lines.push(Line::from(spans));
    }
}

impl Perform for AnsiParser {
    fn print(&mut self, c: char) {
        self.current_text.push(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => {} // Ignore carriage return
            b'\t' => self.current_text.push('\t'),
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        if c == 'm' {
            // SGR - Select Graphic Rendition
            self.flush_text();

            if params.is_empty() {
                // Reset
                self.current_style = Style::default();
                return;
            }

            // Flatten params into a vec for easier lookahead
            let mut param_values: Vec<u16> = Vec::new();
            for param in params.iter() {
                param_values.extend_from_slice(param);
            }

            let mut i = 0;
            while i < param_values.len() {
                let n = param_values[i];
                match n {
                    0 => self.current_style = Style::default(),
                    1 => self.current_style = self.current_style.add_modifier(Modifier::BOLD),
                    2 => self.current_style = self.current_style.add_modifier(Modifier::DIM),
                    3 => self.current_style = self.current_style.add_modifier(Modifier::ITALIC),
                    4 => self.current_style = self.current_style.add_modifier(Modifier::UNDERLINED),
                    7 => self.current_style = self.current_style.add_modifier(Modifier::REVERSED),
                    22 => {
                        self.current_style = self
                            .current_style
                            .remove_modifier(Modifier::BOLD | Modifier::DIM);
                    }
                    23 => {
                        self.current_style =
                            self.current_style.remove_modifier(Modifier::ITALIC);
                    }
                    24 => {
                        self.current_style =
                            self.current_style.remove_modifier(Modifier::UNDERLINED);
                    }
                    27 => {
                        self.current_style =
                            self.current_style.remove_modifier(Modifier::REVERSED);
                    }
                    // 256-color and true color foreground
                    38 => {
                        if i + 1 < param_values.len() {
                            match param_values[i + 1] {
                                // 256-color mode: ESC[38;5;Nm
                                5 if i + 2 < param_values.len() => {
                                    let color_idx = param_values[i + 2] as u8;
                                    self.current_style = self.current_style.fg(Color::Indexed(color_idx));
                                    i += 2;
                                }
                                // True color mode: ESC[38;2;R;G;Bm
                                2 if i + 4 < param_values.len() => {
                                    let r = param_values[i + 2] as u8;
                                    let g = param_values[i + 3] as u8;
                                    let b = param_values[i + 4] as u8;
                                    self.current_style = self.current_style.fg(Color::Rgb(r, g, b));
                                    i += 4;
                                }
                                _ => {}
                            }
                        }
                    }
                    // 256-color and true color background
                    48 => {
                        if i + 1 < param_values.len() {
                            match param_values[i + 1] {
                                // 256-color mode: ESC[48;5;Nm
                                5 if i + 2 < param_values.len() => {
                                    let color_idx = param_values[i + 2] as u8;
                                    self.current_style = self.current_style.bg(Color::Indexed(color_idx));
                                    i += 2;
                                }
                                // True color mode: ESC[48;2;R;G;Bm
                                2 if i + 4 < param_values.len() => {
                                    let r = param_values[i + 2] as u8;
                                    let g = param_values[i + 3] as u8;
                                    let b = param_values[i + 4] as u8;
                                    self.current_style = self.current_style.bg(Color::Rgb(r, g, b));
                                    i += 4;
                                }
                                _ => {}
                            }
                        }
                    }
                    // Foreground colors (16-color)
                    30 => self.current_style = self.current_style.fg(Color::Black),
                    31 => self.current_style = self.current_style.fg(Color::Red),
                    32 => self.current_style = self.current_style.fg(Color::Green),
                    33 => self.current_style = self.current_style.fg(Color::Yellow),
                    34 => self.current_style = self.current_style.fg(Color::Blue),
                    35 => self.current_style = self.current_style.fg(Color::Magenta),
                    36 => self.current_style = self.current_style.fg(Color::Cyan),
                    37 => self.current_style = self.current_style.fg(Color::Gray),
                    39 => self.current_style = self.current_style.fg(Color::Reset),
                    // Bright foreground colors
                    90 => self.current_style = self.current_style.fg(Color::DarkGray),
                    91 => self.current_style = self.current_style.fg(Color::LightRed),
                    92 => self.current_style = self.current_style.fg(Color::LightGreen),
                    93 => self.current_style = self.current_style.fg(Color::LightYellow),
                    94 => self.current_style = self.current_style.fg(Color::LightBlue),
                    95 => self.current_style = self.current_style.fg(Color::LightMagenta),
                    96 => self.current_style = self.current_style.fg(Color::LightCyan),
                    97 => self.current_style = self.current_style.fg(Color::White),
                    // Background colors (16-color)
                    40 => self.current_style = self.current_style.bg(Color::Black),
                    41 => self.current_style = self.current_style.bg(Color::Red),
                    42 => self.current_style = self.current_style.bg(Color::Green),
                    43 => self.current_style = self.current_style.bg(Color::Yellow),
                    44 => self.current_style = self.current_style.bg(Color::Blue),
                    45 => self.current_style = self.current_style.bg(Color::Magenta),
                    46 => self.current_style = self.current_style.bg(Color::Cyan),
                    47 => self.current_style = self.current_style.bg(Color::Gray),
                    49 => self.current_style = self.current_style.bg(Color::Reset),
                    // Bright background colors
                    100 => self.current_style = self.current_style.bg(Color::DarkGray),
                    101 => self.current_style = self.current_style.bg(Color::LightRed),
                    102 => self.current_style = self.current_style.bg(Color::LightGreen),
                    103 => self.current_style = self.current_style.bg(Color::LightYellow),
                    104 => self.current_style = self.current_style.bg(Color::LightBlue),
                    105 => self.current_style = self.current_style.bg(Color::LightMagenta),
                    106 => self.current_style = self.current_style.bg(Color::LightCyan),
                    107 => self.current_style = self.current_style.bg(Color::White),
                    _ => {}
                }
                i += 1;
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
