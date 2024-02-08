use crate::FileState;
use crate::GitIndex;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;
use ratatui::{prelude::*, widgets::*, Frame};

struct Staged<'a> {
    title: &'a str,
    expanded: bool,
}

struct Unstaged<'a> {
    title: &'a str,
    expanded: bool,
}

impl Default for Staged<'_> {
    fn default() -> Self {
        Staged {
            title: "staged files",
            expanded: false,
        }
    }
}

impl Default for Unstaged<'_> {
    fn default() -> Self {
        Unstaged {
            title: "unstaged files",
            expanded: false,
        }
    }
}

pub struct AppState<'a> {
    staged: Staged<'a>,
    unstaged: Unstaged<'a>,
}

impl AppState<'_> {
    pub fn new() -> AppState<'static> {
        AppState {
            staged: Staged::default(),
            unstaged: Unstaged::default(),
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

    fn toggle_section(&mut self) {
        self.staged.expanded = !self.staged.expanded;
        self.unstaged.expanded = !self.unstaged.expanded;
    }
}

pub fn draw_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    git_index: &GitIndex,
    app_state: &AppState,
) -> Result<(), std::io::Error> {
    terminal.draw(|frame| {
        frame.set_cursor(0, 0);
        let size = frame.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                app_state
                    .staged
                    .expanded
                    .then(|| Constraint::Percentage(50))
                    .unwrap_or(Constraint::Length(0)),
                Constraint::Length(1),
                app_state
                    .unstaged
                    .expanded
                    .then(|| Constraint::Percentage(50))
                    .unwrap_or(Constraint::Length(0)),
            ])
            .split(size);

        draw_header(
            frame,
            &chunks[0],
            app_state.staged.title,
            app_state.staged.expanded,
        );
        if app_state.staged.expanded {
            draw_files_section(frame, &chunks[1], &git_index.staged.files);
        }

        draw_header(
            frame,
            &chunks[2],
            app_state.unstaged.title,
            app_state.unstaged.expanded,
        );
        if app_state.unstaged.expanded {
            draw_files_section(frame, &chunks[3], &git_index.unstaged.files);
        }
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

fn draw_header(frame: &mut Frame, area: &Rect, title: &str, expanded: bool) {
    let symbol = if expanded { "▼" } else { "▶" };
    let text = format!("{} {}", symbol, title);
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
    frame.render_widget(paragraph, *area);
}
