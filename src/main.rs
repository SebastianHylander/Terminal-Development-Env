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

            editor.draw(frame, editor_area);
            editor.draw_status_bar(frame, status_area);
            explorer.draw(frame, explorer_area);

            
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
