use anyhow::Result;
use ratatui::{
    layout::{Layout, Rect},
    prelude::Constraint,
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
        Widget, Wrap,
    },
    Frame,
};

use crate::component::{EditingInput, InputArena};
use crate::{
    app::{AppState, CurrentScreen},
    component::MainInput,
};

const FOCUSED_TEXT_COLOR: Color = Color::Green;
const UNFOCUSED_TEXT_COLOR: Color = Color::DarkGray;

const SELECTED_ROW_STYLE_FG: Color = tailwind::SLATE.c400;
const SELECTED_CELL_STYLE_FG: Color = tailwind::SLATE.c600;
const NORMAL_ROW_COLOR: Color = tailwind::SLATE.c950;
const ALT_ROW_COLOR: Color = tailwind::SLATE.c900;
const ROW_FG: Color = tailwind::SLATE.c200;
const BUFFER_BG: Color = tailwind::SLATE.c950;

const ACTIVE_STYLE: Style = Style::new().bg(ALT_ROW_COLOR).fg(Color::Black);
const INACTIVE_STYLE: Style = Style::new().bg(NORMAL_ROW_COLOR).fg(Color::Black);

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // cut r (the given Rect) in 3 parts
    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vectical piece into 3 width-wise pieces
    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(100 - percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // return the middle chunk
}

fn build_title() -> impl Widget {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    Paragraph::new(Text::styled(
        "Semantic Layer Builder",
        Style::default().fg(FOCUSED_TEXT_COLOR),
    ))
    .block(title_block)
}

fn build_search_proto_name(input: &InputArena, screen: &CurrentScreen) -> Result<impl Widget> {
    let mut search_block = Block::default()
        .title(Span::styled(
            "Filter",
            Style::default().fg(FOCUSED_TEXT_COLOR),
        ))
        .borders(Borders::ALL);

    if let CurrentScreen::Main(MainInput::Filter) = &screen {
        search_block = search_block.style(ACTIVE_STYLE);
    } else {
        search_block = search_block.style(INACTIVE_STYLE);
    }

    Ok(Paragraph::new(Text::styled(
        input.get_content(&MainInput::Filter.try_into()?)?.clone(),
        Style::default().fg(FOCUSED_TEXT_COLOR),
    ))
    .block(search_block))
}

fn build_list_protos<'a>(protos: &[&String]) -> Table<'a> {
    let selected_row_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(SELECTED_ROW_STYLE_FG);

    let selected_cell_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(SELECTED_CELL_STYLE_FG);

    let rows = protos.iter().map(|item| {
        let cell = Cell::from(Text::from(item.to_string()));
        Row::new([cell])
            .style(Style::new().fg(ROW_FG).bg(NORMAL_ROW_COLOR))
            .height(1)
    });

    let bar = " █ ";

    Table::new(rows, [Constraint::Min(10)])
        .row_highlight_style(selected_row_style)
        .cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(bar))
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
        .bg(BUFFER_BG)
}

fn build_scrollbar<'a>() -> Scrollbar<'a> {
    Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
}

fn build_proto_text(text: String) -> impl Widget {
    Paragraph::new(text).block(Block::default().borders(Borders::ALL))
}

fn build_mode_footer(screen: &CurrentScreen) -> impl Widget {
    let current_navigation_text = vec![
        // the first half of the text
        match screen {
            CurrentScreen::Main(MainInput::None) => {
                Span::styled("Normal Mode", Style::default().fg(FOCUSED_TEXT_COLOR))
            }
            CurrentScreen::Main(MainInput::Filter) => {
                Span::styled("Filter Mode", Style::default().fg(Color::White))
            }
            CurrentScreen::Editing(_) => {
                Span::styled("Editing Mode", Style::default().fg(Color::Yellow))
            }
            CurrentScreen::Exiting => {
                Span::styled("Exiting Mode", Style::default().fg(Color::LightRed))
            }
        }
        .to_owned(),
        // separator
        Span::styled(" | ", Style::default().fg(Color::White)),
        // the final section of the text, with hints
        {
            if let CurrentScreen::Editing(focused) = &screen {
                match focused {
                    EditingInput::Key => {
                        Span::styled("Editing Json Key", Style::default().fg(FOCUSED_TEXT_COLOR))
                    }
                    EditingInput::Value => Span::styled(
                        "Editing Json Value",
                        Style::default().fg(FOCUSED_TEXT_COLOR),
                    ),
                }
            } else if let CurrentScreen::Main(MainInput::Filter) = &screen {
                Span::styled("Editing filter", Style::default().fg(FOCUSED_TEXT_COLOR))
            } else {
                Span::styled("Not Editing", Style::default().fg(UNFOCUSED_TEXT_COLOR))
            }
        },
    ];

    Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL))
}

fn build_note_footer(screen: &CurrentScreen) -> impl Widget {
    let current_keys_hint = {
        match screen {
            CurrentScreen::Main(MainInput::None) => Span::styled(
                "(q) quit | (f) filter | (r) refresh | (↑) move up | (↓) move down ",
                Style::default().fg(FOCUSED_TEXT_COLOR),
            ),
            CurrentScreen::Main(MainInput::Filter) => Span::styled(
                "(ESC) / (Enter) quit search mode ",
                Style::default().fg(FOCUSED_TEXT_COLOR),
            ),
            CurrentScreen::Editing(_) => Span::styled(
                "(ESC) cancel | (Tab) switch boxes | (Enter) complete",
                Style::default().fg(FOCUSED_TEXT_COLOR),
            ),
            CurrentScreen::Exiting => Span::styled("", Style::default().fg(Color::Red)),
        }
    };

    Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL))
}

fn render_main_screen(frame: &mut Frame, state: &mut AppState, input: &InputArena) -> Result<()> {
    // redesign the main layout
    let layouts = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());
    let middle_layouts = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layouts[1]); // split last part into 2 elements
    let proto_name_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(middle_layouts[0]);

    let footer_layouts = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layouts[2]);

    let title = build_title();
    let search = build_search_proto_name(input, &state.current_screen)?;
    let names: Vec<&String> = state.get_filtered_data()?;
    let list = build_list_protos(&names);
    let scrollbar = build_scrollbar();

    let text = if let Some((_, cached_data)) = &state.cached {
        build_proto_text(cached_data.to_string())
    } else {
        build_proto_text(String::new())
    };
    let mode_footer = build_mode_footer(&state.current_screen);
    let notes_footer = build_note_footer(&state.current_screen);

    // first part will contain the title
    frame.render_widget(title, layouts[0]);
    // second one will contain the list of items
    frame.render_widget(search, proto_name_layout[0]);
    frame.render_stateful_widget(list, proto_name_layout[1], &mut state.state);
    frame.render_stateful_widget(scrollbar, proto_name_layout[1], &mut state.scroll_state);
    frame.render_widget(text, middle_layouts[1]);
    // third part left will contain the mode footer
    frame.render_widget(mode_footer, footer_layouts[0]);
    // third part right will contain the hotkeys footer
    frame.render_widget(notes_footer, footer_layouts[1]);

    Ok(())
}

pub fn render_editing_screen(
    frame: &mut Frame,
    input: &InputArena,
    editing: &EditingInput,
) -> Result<()> {
    let popup_block = Block::default()
        .title("Enter a new key-value pair")
        .borders(Borders::NONE)
        .style(Style::default().bg(UNFOCUSED_TEXT_COLOR));

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(popup_block, area);

    let popup_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let mut key_block = Block::default().title("Key").borders(Borders::ALL);
    let mut value_block = Block::default().title("Value").borders(Borders::ALL);

    match editing {
        EditingInput::Key => key_block = key_block.style(ACTIVE_STYLE),
        EditingInput::Value => value_block = value_block.style(ACTIVE_STYLE),
    };

    let key_ptr = input.get_content(&EditingInput::Key.into())?;
    let key_text = Paragraph::new(key_ptr.clone()).block(key_block);
    frame.render_widget(key_text, popup_chunks[0]);

    let value_ptr = input.get_content(&EditingInput::Value.into())?;
    let value_text = Paragraph::new(value_ptr.clone()).block(value_block);
    frame.render_widget(value_text, popup_chunks[1]);

    Ok(())
}

pub fn render_exit_screen(frame: &mut Frame) {
    frame.render_widget(Clear, frame.area());
    let popup_block = Block::default()
        .title("Y/N")
        .borders(Borders::NONE)
        .style(Style::default().bg(UNFOCUSED_TEXT_COLOR));

    let exit_text = Text::styled("Would you like to quit ?", Style::default().fg(Color::Red));

    let exit_paragraph = Paragraph::new(exit_text)
        .block(popup_block)
        .wrap(Wrap { trim: false });

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(exit_paragraph, area);
}

pub fn view(frame: &mut Frame, state: &mut AppState, input: &InputArena) -> Result<()> {
    render_main_screen(frame, state, input)?;
    // for editing ui
    if let CurrentScreen::Editing(focused) = &state.current_screen {
        render_editing_screen(frame, input, focused)?;
    }
    // exit popup
    if let CurrentScreen::Exiting = &state.current_screen {
        render_exit_screen(frame);
    }
    Ok(())
}
