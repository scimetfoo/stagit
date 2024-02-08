use crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::ListState;

struct App {
    staged_files_state: ListState,
    unstaged_files_state: ListState,
    staged_expanded: bool,
    unstaged_expanded: bool,
    active_section_index: usize,
}

impl App {
    fn new() -> App {
        let mut staged_files_state = ListState::default();
        let mut unstaged_files_state = ListState::default();
        staged_files_state.select(Some(0));
        unstaged_files_state.select(Some(0));

        App {
            staged_files_state,
            unstaged_files_state,
            staged_expanded: true,
            unstaged_expanded: true,
            active_section_index: 0,
        }
    }

    fn toggle_expanded(&mut self) {
        if self.active_section_index == 0 {
            self.staged_expanded = !self.staged_expanded;
        } else {
            self.unstaged_expanded = !self.unstaged_expanded;
        }
    }

    fn toggle_active_section(&mut self) {
        self.active_section_index = (self.active_section_index + 1) % 2;
    }
}

fn handle_events(app: &App) {
    match key.code {
        KeyCode::Char('q') => break,
        KeyCode::Char('e') => app.toggle_expanded(),
        KeyCode::Tab => app.toggle_active_section(),
        KeyCode::Down => {
            if app.active_section_index == 0 && app.staged_expanded
                || app.active_section_index == 1 && app.unstaged_expanded
            {
                app.next();
            }
        }
        KeyCode::Up => {
            if app.active_section_index == 0 && app.staged_expanded
                || app.active_section_index == 1 && app.unstaged_expanded
            {
                app.previous();
            }
        }
        _ => {}
    }
}
