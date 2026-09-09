use std::{
    fs::{self, File}, io, path::PathBuf,
};

use crossterm::event::{Event, KeyCode};

use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph}
};

pub struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    scroll_y: usize,
    file_path: Option<PathBuf>,
    dirty: bool,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        let mut lines = Vec::new();
        lines.push(String::new());
        
        Ok(Self {
            lines,
            cursor_x: 0,
            cursor_y: 0,
            scroll_y: 0,
            file_path: None,
            dirty: false,
        })
    }

    pub fn new_with_file(file_path: PathBuf) -> io::Result<Self> {
        if !file_path.exists() {
            File::create_new(file_path.as_path())?;
        }

        if !file_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "path is not a file",
            ));
        }
        
        let contents = fs::read_to_string(file_path.clone())?;

        let mut lines: Vec<String> =
            contents.split("\n").map(String::from).collect();

        if lines.is_empty() {
            lines.push(String::new());
        }

        Ok(Self {
            lines,
            cursor_x: 0,
            cursor_y: 0,
            scroll_y: 0,
            file_path: Some(file_path),
            dirty: false,
        })
    }

    pub fn insert_char(&mut self, character: char) {
        self.lines[self.cursor_y].insert(self.cursor_x, character);
        self.cursor_x += 1;
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let remainder = self.lines[self.cursor_y].split_off(self.cursor_x);

        self.cursor_y += 1;
        self.cursor_x = 0;

        self.lines.insert(self.cursor_y, remainder);
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.lines[self.cursor_y].remove(self.cursor_x);
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);

            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();

            self.lines[self.cursor_y].push_str(&current_line);
        } else {
            return;
        }

        self.dirty = true;
    }

    pub fn move_cursor(&mut self, direction: KeyCode) {
        match direction {
            KeyCode::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                } else if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    self.cursor_x = self.lines[self.cursor_y].len();
                }
            }

            KeyCode::Right => {
                if self.cursor_x < self.lines[self.cursor_y].len() {
                    self.cursor_x += 1;
                } else if self.cursor_y + 1 < self.lines.len() {
                    self.cursor_y += 1;
                    self.cursor_x = 0;
                }
            }

            KeyCode::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    self.cursor_x =
                        self.cursor_x.min(self.lines[self.cursor_y].len());
                }
            }

            KeyCode::Down => {
                if self.cursor_y + 1 < self.lines.len() {
                    self.cursor_y += 1;
                    self.cursor_x =
                        self.cursor_x.min(self.lines[self.cursor_y].len());
                }
            }

            _ => {}
        }
    }

    pub fn save(&mut self) -> io::Result<()> {
        let Some(path) = &self.file_path else {
            return Ok(());
        };

        let contents = self.lines.join("\n");
        fs::write(path, contents)?;

        self.dirty = false;

        Ok(())
    }

    pub fn ensure_cursor_visible(&mut self, height: usize) {
        if self.cursor_y < self.scroll_y {
            self.scroll_y = self.cursor_y;
        }

        if self.cursor_y >= self.scroll_y + height {
            self.scroll_y = self.cursor_y - height + 1;
        }
    }

    pub fn draw(self: &Editor, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let visible_height = area.height.saturating_sub(2) as usize;

        let text = self.lines
            .iter()
            .enumerate()
            .skip(self.scroll_y)
            .take(visible_height)
            .map(|(index, line)| {
                format!("{:>4}  {}", index + 1, line)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);

        let cursor_screen_y =
                area.y + 1 + (self.cursor_y - self.scroll_y) as u16;

            let cursor_screen_x =
                area.x + 6 + self.cursor_x as u16;

            frame.set_cursor_position((cursor_screen_x+1, cursor_screen_y));
    }

    pub fn draw_status_bar(self: &Editor, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let filename = self.file_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");

        let modified = if self.dirty { " [+]" } else { "" };

        let status = format!(
            " {}{}    Ln {}, Col {}    Ctrl-S Save | Ctrl-Q Quit",
            filename,
            modified,
            self.cursor_y + 1,
            self.cursor_x + 1,
        );

        let paragraph = Paragraph::new(status)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }

    pub fn handle_event(self: &mut Editor, e : Event){
        match e {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char(character) => {
                        self.insert_char(character);
                    }

                    KeyCode::Enter => {
                        self.insert_newline();
                    }

                    KeyCode::Backspace => {
                        self.backspace();
                    }

                    KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down => {
                        self.move_cursor(key.code);
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }

}
