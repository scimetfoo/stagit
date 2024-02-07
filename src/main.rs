use std::io::{self, stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::text::{Span};
use ratatui::widgets::{ListItem};
use ratatui::{prelude::*, widgets::*};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    
    let app_state = initialize_app_state();
    let mut should_quit = false;
    while !should_quit {
        draw_ui(&mut terminal, &app_state)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn draw_ui<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app_state: &AppState) -> Result<(), std::io::Error> {
    let _ = terminal.draw(|frame| {
        let files: Vec<ListItem> = app_state.files.iter().map(|file| {
            let content = if file.expanded {
                file.changes.iter().map(|change| {
                    Span::from(Span::raw(format!("{}: {}", change.line_number, change.content)))
                }).collect::<Vec<Span>>()
            } else {
                vec![Span::from(Span::raw(&file.path))]
            };

            let text = Text::from(Line::from(content));

            ListItem::new(text)  
        }).collect();  

        let files_list = List::new(files)
            .block(Block::default().title("Files to Stage").borders(Borders::ALL));
        
        frame.render_widget(files_list, frame.size());
    });
    Ok(())
}

fn initialize_app_state() -> AppState {
    let files = vec![
        FileState {
            path: "src/main.rs".to_string(),
            expanded: false,
            changes: vec![
                Change {
                    line_number: 10,
                    content: "+ println!(\"Hello, world!\");".to_string(),
                },
            ],
        },
    ];

    AppState { files }
}

struct AppState {
    files: Vec<FileState>,
}

struct FileState {
    path: String,
    expanded: bool,
    changes: Vec<Change>,
}

struct Change {
    line_number: usize,
    content: String,
}

