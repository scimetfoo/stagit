use git2::{Repository, Status, StatusOptions, StatusShow};
use std::error::Error;
use std::io::{self, stdout};
use std::path::PathBuf;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::text::{Line, Span};
use ratatui::layout::Rect;
use ratatui::widgets::ListItem;
use ratatui::{prelude::*, widgets::*, Frame};

struct GitIndex {
    staged: Staged,
    unstaged: Unstaged,
}

struct FileState {
    path: String,
    expanded: bool,
    changes: Vec<Change>,
}

struct Unstaged {
    files: Vec<FileState>,
}

struct Staged {
    files: Vec<FileState>,
}

enum ChangeType {
    Addition,
    Deletion,
}

struct Change {
    line_number: usize,
    content: String,
}

trait GitRepository {
    fn new(path: PathBuf) -> Self;
    fn fetch_index(&self) -> Result<GitIndex, Box<dyn Error>>;
}

struct CurrentGitRepository {
    path: PathBuf,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let path = PathBuf::from(r".");

    match CurrentGitRepository::new(path).fetch_index() {
        Err(e) => {
            println!("Error: {e}");
        }
        Ok(git_index) => {
            let mut should_quit = false;
            while !should_quit {
                draw_ui(&mut terminal, &git_index)?;
                should_quit = handle_events()?;
            }
        }
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

fn draw_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    git_index: &GitIndex,
) -> Result<(), std::io::Error> {
    terminal.draw(|frame| {
        let size = frame.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(size);

        // Staged files section
        draw_files_section(frame, &chunks[0], "Staged Files", &git_index.staged.files);

        // Unstaged files section
        draw_files_section(frame, &chunks[1], "Unstaged Files", &git_index.unstaged.files);
    });
    Ok(())
}

fn draw_files_section(frame: &mut Frame, area: &Rect, title: &str, files: &[FileState]) {
    let files_list: Vec<ListItem> = files.iter().map(|file| {
        let content = if file.expanded {
            file.changes.iter().map(|change| {
                Span::from(Span::raw(format!(
                                "{}: {}",
                                change.line_number, change.content
                            )))
            }).collect::<Vec<Span>>()
        } else {
            vec![Span::from(Span::raw(&file.path))]
        };

        ListItem::new(Text::from(Line::from(content)))
    }).collect();

    let list = List::new(files_list)
        .block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(list, *area);
}

impl GitRepository for CurrentGitRepository {
    fn new(path: PathBuf) -> Self {
        CurrentGitRepository { path }
    }

    fn fetch_index(&self) -> Result<GitIndex, Box<dyn Error>> {
        let repo = Repository::open(&self.path)?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_ignored(false)
            .show(git2::StatusShow::IndexAndWorkdir);

        let statuses = repo.statuses(Some(&mut opts))?;

        let mut staged_files: Vec<FileState> = Vec::new();
        let mut unstaged_files: Vec<FileState> = Vec::new();

        for entry in statuses.iter() {
            let file_path = entry.path().unwrap_or_default().to_string();


            // Determine if the file is staged
            if entry.status().intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE,
            ) {
                update_file_states(&mut staged_files, file_path.clone());
            }

            // Determine if the file is unstaged
            if entry.status().intersects(
                Status::WT_MODIFIED
                    | Status::WT_DELETED
                    | Status::WT_NEW
                    | Status::WT_RENAMED
                    | Status::WT_TYPECHANGE,
            ) {
                update_file_states(&mut unstaged_files, file_path);
            }
        }

        Ok(GitIndex {
            staged: Staged {
                files: staged_files,
            },
            unstaged: Unstaged {
                files: unstaged_files,
            },
        })
    }
}

fn update_file_states(file_states: &mut Vec<FileState>, file_path: String) {
    match file_states.iter_mut().find(|fs| fs.path == file_path) {
        Some(file_state) => (),
        None => file_states.push(FileState {
            path: file_path,
            expanded: false,
            changes: vec![],
        }),
    }
}
