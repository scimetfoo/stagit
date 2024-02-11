use crate::FileState;
use crate::GitIndex;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;
use ratatui::{prelude::*, widgets::*, Frame};

struct Header {
    title: String,
    expanded: bool,
}

pub trait Toggle {
    fn construct_title(&self) -> String;
    fn toggle_section(&self) -> Header;
}

impl Toggle for Header {
    fn construct_title(&self) -> String {
        let indicator = if self.expanded { "▼" } else { "▶" };
        return format!("{} {}", indicator, self.title);
    }

    fn toggle_section(&self) -> Header {
        return Header {
            title: self.construct_title(),
            expanded: !self.expanded,
        };
    }
}

pub struct AppState {
    headers: Vec<Header>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            headers: vec![
                Header {
                    title: String::from("untracked files"),
                    expanded: false,
                }
                .toggle_section(),
                Header {
                    title: String::from("unstaged files"),
                    expanded: false,
                }
                .toggle_section(),
                Header {
                    title: String::from("staged files"),
                    expanded: false,
                }
                .toggle_section(),
            ],
        }
    }

    pub fn run<B: Backend>(
        self,
        terminal: &mut Terminal<B>,
        git_index: &GitIndex,
    ) -> Result<(), std::io::Error> {
        loop {
            draw_ui(terminal, &git_index, &self)?;
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn draw_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    _git_index: &GitIndex,
    app_state: &AppState,
) -> Result<(), std::io::Error> {
    terminal.draw(|frame| {
        frame.set_cursor(0, 0);
        let _header_state = draw_headers(frame, frame.size(), app_state);
    });
    Ok(())
}

fn draw_files_section(frame: &mut Frame, area: &Rect, files: &[FileState]) {
    let files_list: Vec<ListItem> = files
        .iter()
        .map(|file| {
            let content = if file.expanded {
                file.changes
                    .iter()
                    .map(|change| {
                        Span::from(Span::raw(format!(
                            "{}: {}",
                            change.line_number, change.content
                        )))
                    })
                    .collect::<Vec<Span>>()
            } else {
                vec![Span::from(Span::raw(&file.path))]
            };

            ListItem::new(Text::from(Line::from(content)))
        })
        .collect();

    let list = List::new(files_list);

    frame.render_widget(list, *area);
}

fn draw_headers(frame: &mut Frame, area: Rect, app_state: &AppState) -> ListState {
    let header_state: &mut ListState = &mut ListState::default();
    let headers: Vec<ListItem> = app_state
        .headers
        .iter()
        .map(|header| {
            let text: Text = Text::from(header.title.clone());
            ListItem::new(text)
        })
        .collect();

    let header_list = List::new(headers)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(header_list, area, header_state);
    return header_state.clone();
}
