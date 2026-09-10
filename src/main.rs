mod explorer;
use explorer::Explorer;

mod tabs;
use tabs::TabsWindow;

use std::{
    env, io::{self, stdout}, path::PathBuf,
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
    tabs : TabsWindow,
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
                let mut tabs = TabsWindow::new()?;

                let mut explorer = Explorer::new_from_root_dir(path)?;

                enable_raw_mode()?;

                let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen
                )?;

                terminal.clear()?;

                let mut ide = IDE {tabs : tabs, explorer : explorer, input_mode : EXPLORER};

                let result = run_editor(&mut terminal, &mut ide);

                disable_raw_mode()?;

                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen
                )?;

                terminal.show_cursor()?;

                result?;
            } else {
                let mut tabs = TabsWindow::new_with_file(path.clone())?;

                let mut explorer = Explorer::new_from_file(path)?;

                enable_raw_mode()?;

                let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen
                )?;

                terminal.clear()?;

                let mut ide = IDE {tabs : tabs, explorer : explorer, input_mode : EDITOR};

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


            let explorer_area = layout[0];
            let tabs_area = layout[1];

            ide.explorer.draw(frame, explorer_area);
            ide.tabs.draw(frame, tabs_area);            
        })?;

        let e= event::read()?;
        
        match e{
            Event::Key(key) => {
                match handle_key_event(ide, key)? {
                    Some(stop) => {
                        if stop {
                            break;
                        } 
                    }
                    None => {
                        match ide.input_mode {
                            EXPLORER => {
                                match ide.explorer.handle_event(e)? {
                                    Some(path) => {
                                        ide.tabs.new_tab(path)?;
                                        ide.input_mode = EDITOR;
                                    }

                                    None => {}
                                };
                            }
                            EDITOR => {
                                ide.tabs.handle_event(e);
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
) -> io::Result<Option<bool>> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('q') => {
                return Ok(Some(true));
            }

            KeyCode::Char('s') => {
                ide.tabs.save()?;
                return Ok(Some(false));
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
                return Ok(Some(false))
            }
            _ => {}
        }
    }
    Ok(None)
}
