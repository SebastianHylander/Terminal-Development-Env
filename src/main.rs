mod explorer;
use explorer::Explorer;

mod editor;
use editor::Editor;

use std::{
    env, io::{stdout}, path::{PathBuf},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},

    Terminal,
};

use crate::input_modes::{EDITOR, EXPLORER};

enum input_modes {
    EDITOR,
    EXPLORER
}

struct IDE {
    editor : Editor,
    explorer : Explorer,
    input_mode : input_modes
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let path = env::args().nth(1).map(PathBuf::from);

    match path {
        None => {
            Ok(())
        }
        
        Some(path) => {
            if path.is_dir() {
                let mut editor = Editor::new()?;

                let mut explorer = Explorer::new_from_root_dir(path)?;

                enable_raw_mode()?;

                let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen
                )?;

                terminal.clear()?;

                let mut ide = IDE {editor : editor, explorer : explorer, input_mode : EXPLORER};

                let result = run_editor(&mut terminal, &mut ide);

                disable_raw_mode()?;

                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen
                )?;

                terminal.show_cursor()?;

                result?;
            } else {
                let mut editor = Editor::new_with_file(path.clone())?;

                let mut explorer = Explorer::new_from_file(path)?;

                enable_raw_mode()?;

                let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen
                )?;

                terminal.clear()?;

                let mut ide = IDE {editor : editor, explorer : explorer, input_mode : EDITOR};

                let result = run_editor(&mut terminal, &mut ide);

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
    ide : &mut IDE
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

            ide.editor.ensure_cursor_visible(visible_height);

            ide.editor.draw(frame, editor_area);
            ide.editor.draw_status_bar(frame, status_area);
            ide.explorer.draw(frame, explorer_area);

            
        })?;

        let e= event::read()?;
        
        match e{
            Event::Key(key) => {
                match handle_key_event(ide, key) {
                    Some(stop) => {
                        if stop {
                            break;
                        } 
                    }
                    None => {
                        match ide.input_mode {
                            EXPLORER => {
                                ide.explorer.handle_event(e);
                            }
                            EDITOR => {
                                ide.editor.handle_event(e);
                            }
                        }
                    }
                }
            }
            _ => {}
        }        
    }

    Ok(())
}

fn handle_key_event(
    ide: &mut IDE,
    key: KeyEvent,
) -> Option<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('q') => {
                return Some(true);
            }

            KeyCode::Char('s') => {
                ide.editor.save().ok()?;
                return Some(false);
            }

            KeyCode::Char('e') => {
                match ide.input_mode {
                    EXPLORER => {
                        ide.input_mode = EDITOR;
                    }

                    EDITOR => {
                        ide.input_mode = EXPLORER;
                    }
                }
                return Some(false)
            }
            _ => {}
        }
    }
    None
}
