//! Terminal setup/teardown for the explorer.
//!
//! [`Tui`] enters raw mode + the alternate screen on construction and restores
//! the terminal on `Drop`, so the user's shell is left clean even if the app
//! panics mid-frame.

use std::io::{Stdout, stdout};

use crossterm::event::EventStream;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::error::{Error, Result};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Async stream of crossterm input events.
    pub events: EventStream,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode().map_err(map_io)?;
        let mut out = stdout();
        out.execute(EnterAlternateScreen).map_err(map_io)?;
        let terminal = Terminal::new(CrosstermBackend::new(out)).map_err(map_io)?;
        Ok(Self {
            terminal,
            events: EventStream::new(),
        })
    }

    /// Explicitly restore the terminal. Also runs on `Drop`, but calling it
    /// directly lets errors surface during normal shutdown.
    pub fn restore() -> Result<()> {
        disable_raw_mode().map_err(map_io)?;
        execute!(stdout(), LeaveAlternateScreen).map_err(map_io)?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        // Best-effort restore; ignore errors since we may be unwinding.
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

fn map_io(e: std::io::Error) -> Error {
    Error::Io(e)
}
