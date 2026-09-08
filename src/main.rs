use std::{
    env, fs::{self}, io::{self, stdout}, path::{PathBuf},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use ratatui::widgets::FrameExt as _;

use ratatui_explorer::{FileExplorerBuilder, FileExplorer, Theme};

struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    scroll_y: usize,
    file_path: Option<PathBuf>,
    dirty: bool,
}

impl Editor {
    fn new(file_path: Option<PathBuf>) -> io::Result<Self> {
        let lines = match &file_path {
            Some(path) => {
                if path.exists() {
                    let contents = fs::read_to_string(path)?;

                    let mut lines: Vec<String> =
                        contents.lines().map(String::from).collect();

                    if lines.is_empty() {
                        lines.push(String::new());
                    }

                    lines
                } else {
                    let mut lines = Vec::new();
                    lines.push(String::new());
                    lines
                }
            }
            None => {
                let mut lines = Vec::new();
                lines.push(String::new());
                lines
            }
        };

        Ok(Self {
            lines,
            cursor_x: 0,
            cursor_y: 0,
            scroll_y: 0,
            file_path,
            dirty: false,
        })
    }

    fn insert_char(&mut self, character: char) {
        self.lines[self.cursor_y].insert(self.cursor_x, character);
        self.cursor_x += 1;
        self.dirty = true;
    }

    fn insert_newline(&mut self) {
        let remainder = self.lines[self.cursor_y].split_off(self.cursor_x);

        self.cursor_y += 1;
        self.cursor_x = 0;

        self.lines.insert(self.cursor_y, remainder);
        self.dirty = true;
    }

    fn backspace(&mut self) {
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

    fn move_cursor(&mut self, direction: KeyCode) {
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

    fn save(&mut self) -> io::Result<()> {
        let Some(path) = &self.file_path else {
            return Ok(());
        };

        let contents = self.lines.join("\n");
        fs::write(path, contents)?;

        self.dirty = false;

        Ok(())
    }

    fn ensure_cursor_visible(&mut self, height: usize) {
        if self.cursor_y < self.scroll_y {
            self.scroll_y = self.cursor_y;
        }

        if self.cursor_y >= self.scroll_y + height {
            self.scroll_y = self.cursor_y - height + 1;
        }
    }
}

struct Explorer {
    root_dir: PathBuf,
    file_explorer: FileExplorer
} 
impl Explorer {
    fn new(dir_path : PathBuf, mut file_explorer : FileExplorer) -> io::Result<Self> {
        let title = String::from(dir_path.clone().file_stem().unwrap().to_str().unwrap());

        let theme = Theme::default().with_title_top(move |_| {format!("{}/",  title.clone()).into()});
        file_explorer.set_theme(theme);
        

        Ok(Self{
            root_dir: dir_path,
            file_explorer: file_explorer
        })
    }

    fn new_from_root_dir(dir_path: PathBuf) -> io::Result<Self> {
        if !dir_path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "path is not a directory",
            ));
        }
        Self::new(dir_path.clone(), FileExplorerBuilder::build_with_working_dir(dir_path).unwrap())
    }

    fn new_from_file(file_path: PathBuf) -> io::Result<Self> {
        if !file_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "path is not a file",
            ));
        }
        Self::new(file_path.clone().parent().unwrap().to_path_buf(), FileExplorerBuilder::build_with_working_file(file_path).unwrap())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let path = env::args().nth(1).map(PathBuf::from);

    match path {
        None => {
            Ok(())
        }
        
        Some(path) => {
            if path.is_dir() {
                let mut editor = Editor::new(None)?;

                let mut explorer = Explorer::new_from_root_dir(path)?;

                enable_raw_mode()?;

                let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen
                )?;

                terminal.clear()?;

                let result = run_editor(&mut terminal, &mut editor, &mut explorer);

                disable_raw_mode()?;

                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen
                )?;

                terminal.show_cursor()?;

                result?;
            } else {
                let mut editor = Editor::new(Some(path.clone()))?;

                let mut explorer = Explorer::new_from_file(path)?;

                enable_raw_mode()?;

                let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen
                )?;

                terminal.clear()?;

                let result = run_editor(&mut terminal, &mut editor, &mut explorer);

                disable_raw_mode()?;

                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen
                )?;

                terminal.show_cursor()?;

                result?;
            }
            

            Ok(())
        }

    }


}

fn run_editor(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    editor: &mut Editor,
    explorer: &mut Explorer
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| {
            let area = frame.area();


            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(15),
                    Constraint::Percentage(85)
                ])
                .split(area);

            let editor_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(layout[1]);
            let explorer_area = layout[0];
            let editor_area = editor_layout[0];
            let status_area = editor_layout[1];



            let visible_height = editor_area.height.saturating_sub(2) as usize;

            editor.ensure_cursor_visible(visible_height);

            draw_editor(frame, editor, editor_area);
            draw_status_bar(frame, editor, status_area);
            draw_explorer(frame, explorer, explorer_area);

            let cursor_screen_y =
                editor_area.y + 1 + (editor.cursor_y - editor.scroll_y) as u16;

            let cursor_screen_x =
                editor_area.x + 6 + editor.cursor_x as u16;

            frame.set_cursor_position((cursor_screen_x+1, cursor_screen_y));
        })?;

        if let Event::Key(key) = event::read()? {
            if handle_key_event(editor, key)? {
                break;
            }
        }
    }

    Ok(())
}

fn handle_key_event(
    editor: &mut Editor,
    key: KeyEvent,
) -> Result<bool, Box<dyn std::error::Error>> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('q') => {
                return Ok(true);
            }

            KeyCode::Char('s') => {
                editor.save()?;
                return Ok(false);
            }

            _ => {}
        }
    }

    match key.code {
        KeyCode::Char(character) => {
            editor.insert_char(character);
        }

        KeyCode::Enter => {
            editor.insert_newline();
        }

        KeyCode::Backspace => {
            editor.backspace();
        }

        KeyCode::Left
        | KeyCode::Right
        | KeyCode::Up
        | KeyCode::Down => {
            editor.move_cursor(key.code);
        }

        _ => {}
    }

    Ok(false)
}

fn draw_editor(
    frame: &mut ratatui::Frame,
    editor: &Editor,
    area: ratatui::layout::Rect,
) {
    let visible_height = area.height.saturating_sub(2) as usize;

    let text = editor
        .lines
        .iter()
        .enumerate()
        .skip(editor.scroll_y)
        .take(visible_height)
        .map(|(index, line)| {
            format!("{:>4}  {}", index + 1, line)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(
    frame: &mut ratatui::Frame,
    editor: &Editor,
    area: ratatui::layout::Rect,
) {
    let filename = editor
        .file_path
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("[No Name]");

    let modified = if editor.dirty { " [+]" } else { "" };

    let status = format!(
        " {}{}    Ln {}, Col {}    Ctrl-S Save | Ctrl-Q Quit",
        filename,
        modified,
        editor.cursor_y + 1,
        editor.cursor_x + 1,
    );

    let paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

fn draw_explorer(
    frame: &mut ratatui::Frame,
    explorer: &Explorer,
    area: ratatui::layout::Rect,
) {
    frame.render_widget_ref(explorer.file_explorer.widget(), area);
}
