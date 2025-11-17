use crate::app::{App, CurrentScreen};
use ratatui::{
    layout::{Alignment, Constraint}, prelude::{Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span, Text}, widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph}, Frame
};
use serde_json::{Map, Value};
use std::io::{Error, ErrorKind};

pub fn ui(
    frame: &mut Frame,
    app: &App,
    list_state: &mut ListState,
    json: &Value,
    json_map: &Map<String, Value>,
) -> Result<(), Error> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let title = Paragraph::new(Text::styled(
        "Interact with NN API",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, chunks[0]);

    let mut list_items = Vec::<ListItem>::new();

    for key in json_map.keys() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!(
                "{: <25} : {}",
                key,
                json_map
                    .get(key)
                    .ok_or(Error::new(ErrorKind::NotFound, "Couldn't Find Value"))?
                    .to_string()
            ),
            Style::default(),
        ))))
    }

    let list = List::new(list_items).highlight_style(Modifier::REVERSED);

    frame.render_stateful_widget(list, chunks[1], list_state);

    let current_navigation_text = vec![
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::DarkGray)),
            CurrentScreen::Editing => {
                Span::styled("Editing Mode", Style::default().fg(Color::Green))
            }
            CurrentScreen::Adding => Span::styled("Adding Mode", Style::default().fg(Color::Blue)),
            CurrentScreen::Deleting => {
                Span::styled("Deleting Mode", Style::default().fg(Color::Red))
            }
        }
        .to_owned(),
        Span::styled(" | ", Style::default().fg(Color::White)),
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Not Editing Anything", Style::default().fg(Color::Gray)),
            CurrentScreen::Editing => Span::styled("Editing Json Value", Style::default().fg(Color::LightGreen)),
            CurrentScreen::Adding => Span::styled("Adding Json Pair", Style::default().fg(Color::LightBlue)),
            CurrentScreen::Deleting => Span::styled("Deleting Json Pair", Style::default().fg(Color::LightRed)),
    }
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Main => match json {
                Value::Array(_) => Span::styled(
                    "(q)uit / (e)dit / (a)dd / (d)elete",
                    Style::default().fg(Color::Red),
                ),
                _ => Span::styled(
                    "(q)uit / (e)dit / (d)elete",
                    Style::default().fg(Color::Red),
                ),
            },
            CurrentScreen::Editing => Span::styled(
                "(ESC) to cancel / (Enter) to write value",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Adding => Span::styled(
                "(ESC) to cancel / (Enter) to push value",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Deleting => Span::styled(
                "(ESC) to cancel / (Tab) to switch / (Enter) to confirm",
                Style::default().fg(Color::Red),
            ),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);

    let mut popup_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::DarkGray))
        .title_alignment(Alignment::Center);
    match &app.current_screen {
        CurrentScreen::Editing | CurrentScreen::Adding => popup_block = popup_block.title("Enter a new value"),
        CurrentScreen::Deleting => popup_block = popup_block.title("Are you sure?"),
        _ => {},
    }

    let area = centered_rect(60, 25, frame.area());

    let popup_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let key_block = Block::default().title("Key").borders(Borders::ALL);
    let value_block = Block::default()
        .title("Value")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::LightYellow).fg(Color::Black));
    if let Some(editing) = &app.currently_editing {
        frame.render_widget(Clear, area);
        frame.render_widget(&popup_block, area);
        let trimmed = match editing.key.trim_start_matches('0') {
            "" => "0",
            s => s,
        };
        let key_text = Paragraph::new(trimmed).block(key_block);
        let value_text = Paragraph::new(editing.value.clone()).block(value_block);

        frame.render_widget(key_text, popup_chunks[0]);
        frame.render_widget(value_text, popup_chunks[1]);
    }

    if let Some(adding) = &app.currently_adding {
        frame.render_widget(Clear, area);
        frame.render_widget(&popup_block, area);
        let key_block = Block::default().title("Key").borders(Borders::ALL);
        let value_block = Block::default()
            .title("Value")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::LightYellow).fg(Color::Black));

        let key_text = Paragraph::new(format!(
            "{}",
            json.as_array()
                .ok_or(Error::new(ErrorKind::InvalidData, "Not an Array"))?
                .len()
        ))
        .block(key_block);
        let value_text = Paragraph::new(adding.value.clone()).block(value_block);

        frame.render_widget(key_text, popup_chunks[0]);
        frame.render_widget(value_text, popup_chunks[1]);
    }

    if let Some(deleting) = &app.currently_deleting {
        frame.render_widget(Clear, area);
        frame.render_widget(&popup_block, area);
        let mut yes_block = Block::default().borders(Borders::ALL);
        let mut no_block = Block::default().borders(Borders::ALL);
        let selected_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

        match deleting.are_you_sure {
            true => yes_block = yes_block.style(selected_style),
            false => no_block = no_block.style(selected_style),
        }

        let yes_text = Paragraph::new("Yes").block(yes_block).centered();
        let no_text = Paragraph::new("No").block(no_block).centered();

        frame.render_widget(yes_text, popup_chunks[0]);
        frame.render_widget(no_text, popup_chunks[1]);
    }
    Ok(())
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
