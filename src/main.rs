mod app;
mod config;
mod requests;
mod ui;

use crate::{
    app::{App, CurrentScreen, CurrentlyAdding, CurrentlyDeleting, CurrentlyEditing},
    config::Config,
    requests::api_get,
    ui::ui,
};
use color_eyre::eyre::OptionExt;
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::{Backend, CrosstermBackend},
    widgets::ListState,
};
use serde_json::{Map, Value};
use std::io::{self, Error, ErrorKind};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let json = api_get("")?;
    let config = Config::init()?;
    let mut app = App::new(json, config);
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> color_eyre::Result<()> {
    let mut list_state = ListState::default().with_selected(Some(0));
    loop {
        let json = app
            .json
            .pointer(
                &app.key_path
                    .iter()
                    .map(|v| format!("/{}", v))
                    .collect::<String>(),
            )
            .ok_or_eyre("Pointer DNE")?;
        let json_map = match json {
            Value::Object(j) => Some(j),
            Value::Array(j) => Some(&Map::from_iter(j.iter().enumerate().map(|(i, v)| {
                (
                    format!("{:0fill$}", i, fill = j.len().to_string().len()),
                    v.clone(),
                )
            }))),
            _ => None,
        };
        terminal.try_draw(|f| {
            ui(
                f,
                app,
                &mut list_state,
                json,
                json_map.ok_or(Error::new(ErrorKind::InvalidData, "Not an Array or Object"))?,
            )
        })?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('j') | KeyCode::Down => list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => list_state.select_previous(),
                    KeyCode::Char('[') => list_state.select_first(),
                    KeyCode::Char(']') => list_state.select_last(),
                    KeyCode::Char('l') | KeyCode::Right => {
                        if let Some(json) = json_map {
                            let selected = json
                                .keys()
                                .nth(list_state.selected().ok_or_eyre("No Item Selected")?)
                                .ok_or_eyre("Out of Range")?
                                .to_string();
                            if json
                                .get(&selected)
                                .ok_or_eyre("Couldn't Find Value")?
                                .is_object()
                                || json
                                    .get(&selected)
                                    .ok_or_eyre("Couldn't Find Value")?
                                    .is_array()
                            {
                                let trimmed = match selected.trim_start_matches('0') {
                                    "" => "0",
                                    s => s,
                                };
                                app.key_path.push(trimmed.to_string());
                                app.locations
                                    .push(list_state.selected().ok_or_eyre("No Item Selected")?);
                                list_state.select(Some(0));
                            }
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        if app.locations.len() > 0 {
                            app.key_path.pop();
                            list_state.select(app.locations.last().copied());
                            app.locations.pop();
                        }
                    }
                    KeyCode::Char('e') => {
                        app.current_screen = CurrentScreen::Editing;
                        if let Some(json) = json_map {
                            let selected = json
                                .keys()
                                .nth(list_state.selected().ok_or_eyre("No Item Selected")?)
                                .ok_or_eyre("Out of Range")?
                                .to_string();
                            app.currently_editing = Some(CurrentlyEditing {
                                key: selected.clone(),
                                value: json
                                    .get(&selected)
                                    .ok_or_eyre("Couldn't Find Value")?
                                    .to_string(),
                                changed: false,
                            })
                        }
                    }
                    KeyCode::Char('a') => {
                        if let Value::Array(_) = json {
                            app.current_screen = CurrentScreen::Adding;
                            app.currently_adding = Some(CurrentlyAdding {
                                value: String::from(""),
                            })
                        }
                    }
                    KeyCode::Char('d') => {
                        app.current_screen = CurrentScreen::Deleting;
                        if let Some(json) = json_map {
                            let selected = json
                                .keys()
                                .nth(list_state.selected().ok_or_eyre("No Item Selected")?)
                                .ok_or_eyre("Out of Range")?
                                .to_string();
                            app.currently_deleting = Some(CurrentlyDeleting {
                                key: selected.clone(),
                                are_you_sure: false,
                            })
                        }
                    }
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                },
                CurrentScreen::Editing if key.kind == KeyEventKind::Press => {
                    if let Some(editing) = &mut app.currently_editing {
                        match key.code {
                            KeyCode::Enter => {
                                app.save_edited_value()?;
                                let json = api_get("")?;
                                app.json = json;
                                app.currently_editing = None;
                                app.current_screen = CurrentScreen::Main;
                            }
                            KeyCode::Backspace => {
                                if !editing.changed {
                                    editing.value = String::from("");
                                    editing.changed = true;
                                } else {
                                    editing.value.pop();
                                }
                            }
                            KeyCode::Esc => {
                                app.current_screen = CurrentScreen::Main;
                                app.currently_editing = None;
                            }
                            KeyCode::Char(value) => {
                                if !editing.changed {
                                    editing.value = String::from("");
                                    editing.changed = true;
                                    editing.value.push(value);
                                } else {
                                    editing.value.push(value);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                CurrentScreen::Adding if key.kind == KeyEventKind::Press => {
                    if let Some(adding) = &mut app.currently_adding {
                        match key.code {
                            KeyCode::Enter => {
                                app.push_object_to_array()?;
                                let json = api_get("")?;
                                app.json = json;
                                app.currently_adding = None;
                                app.current_screen = CurrentScreen::Main;
                            }
                            KeyCode::Backspace => {
                                adding.value.pop();
                            }
                            KeyCode::Esc => {
                                app.current_screen = CurrentScreen::Main;
                                app.currently_adding = None;
                            }
                            KeyCode::Char(value) => {
                                adding.value.push(value);
                            }
                            _ => {}
                        }
                    }
                }
                CurrentScreen::Deleting if key.kind == KeyEventKind::Press => {
                    if let Some(deleting) = &mut app.currently_deleting {
                        match key.code {
                            KeyCode::Enter => {
                                if deleting.are_you_sure {
                                    app.delete_value()?;
                                }
                                let json = api_get("")?;
                                app.json = json;
                                app.currently_deleting = None;
                                app.current_screen = CurrentScreen::Main;
                            }
                            KeyCode::Esc => {
                                app.current_screen = CurrentScreen::Main;
                                app.currently_deleting = None;
                            }
                            KeyCode::Tab => match deleting.are_you_sure {
                                true => deleting.are_you_sure = false,
                                false => deleting.are_you_sure = true,
                            },
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
