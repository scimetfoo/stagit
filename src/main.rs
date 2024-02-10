use git2::{Repository, Status, StatusOptions};
use std::error::Error;
use std::io::{self, stdout};
use std::path::PathBuf;

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};



use ratatui::{prelude::*};
mod events;

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
    terminal.show_cursor()?;
    let path = PathBuf::from(r".");

    match CurrentGitRepository::new(path).fetch_index() {
        Err(e) => {
            println!("Error: {e}");
        }
        Ok(git_index) => events::AppState::new().run(&mut terminal, &git_index)?,
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
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

            if entry.status().intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE,
            ) {
                update_file_states(&mut staged_files, file_path.clone());
            }

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
        Some(_file_state) => (),
        None => file_states.push(FileState {
            path: file_path,
            expanded: false,
            changes: vec![],
        }),
    }
}
