mod editor;
use std::{io, path::PathBuf, format};

use crossterm::event::{Event, KeyCode, KeyModifiers};
use editor::Editor;
use ratatui::{layout::{Constraint, Direction, Layout}, style::{Color, Style}, symbols, widgets::{Block, Borders, Tabs}};

pub struct TabsWindow {
    tabs : Vec<Editor>,
    current : usize
}

impl TabsWindow {
    pub fn new () -> io::Result<Self> {
        Ok (
            Self{
                tabs : Vec::new(),
                current : 0
            }
        )
    }

    pub fn new_with_file (file_path : PathBuf) -> io::Result<Self> {
        let mut tabs = Self::new()?;
        tabs.tabs.push(Editor::new_with_file(file_path)?);
        Ok(tabs)
    }

    pub fn new_tab(self : &mut TabsWindow, file_path : PathBuf) -> io::Result<()> {
        self.tabs.push(Editor::new_with_file(file_path)?);
        self.current = self.tabs.len() - 1;
        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        self.tabs[self.current].save()
    }

    pub fn draw(self : &mut TabsWindow, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        if self.tabs.len() == 0 {
            return
        }

        let tabs_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        let tabs = Tabs::new(self.tabs.iter().map(
            |e : &Editor| {
                let s = e.get_file_name().unwrap_or_else(
                    || {String::from("[No Name]")}
                );
                if e.dirty {
                    s+"*"
                }
                else {s}
            })
        .map(format_tab_title))
        .style(Color::White)
        .highlight_style(Style::default().underlined().on_dark_gray().bold())
        .select(self.current)
        .divider(symbols::DOT)
        .padding(" ", " ")
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(tabs, tabs_layout[0]);
        
        self.tabs[self.current].draw(frame, tabs_layout[1]);
    }

    pub fn handle_event(self: &mut TabsWindow, e : Event){
        match e {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::PageUp | KeyCode::Tab => { 
                            if self.tabs.len() > 0 {self.current = (self.current + 1) % self.tabs.len()}
                        }
                        KeyCode::PageDown => { 
                            if self.tabs.len() > 0 {self.current = ((self.current + self.tabs.len()) - 1) % self.tabs.len()}
                        }
                        KeyCode::Char('w') => {
                            if self.tabs.len() > 0 {
                                self.tabs.remove(self.current);
                                if self.tabs.len() > 0 {self.current = ((self.current + self.tabs.len()) - 1) % self.tabs.len()}
                            }
                        }
                        _ => {}
                    }
                } else {
                    self.tabs[self.current].handle_event(e);
                }
            }
            _ => {}
        }
    }
}

fn format_tab_title(filename: String) -> String {
    let max_len = 15;
    if filename.len() > max_len {
        let (name, ext) = filename.rsplit_once('.').unwrap_or((&filename, ""));
        let suffix = format!("..{}", ext);

        format!("{}{}", &name[..max_len - suffix.len()], suffix)
    } else {
        format!("{:^width$}", filename, width = max_len)
    }
}