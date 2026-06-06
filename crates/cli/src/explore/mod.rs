//! `uc explore` — an interactive TUI for browsing the catalog hierarchy.
//!
//! A two-pane explorer: a lazily-loaded tree (catalog → schema →
//! tables/volumes/functions) on the left and a detail view on the right.
//! The event loop multiplexes terminal input, a periodic tick, and the results
//! of async client calls over a single channel, so the UI stays responsive
//! while requests are in flight.

mod action;
mod app;
mod tui;

use clap::Args;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::{Duration, interval};

use self::action::Action;
use self::app::App;
use self::tui::Tui;
use crate::GlobalOpts;
use crate::error::{Error, Result};

#[derive(Debug, Args)]
pub struct ExploreCommand;

pub async fn handle_explore(_cmd: &ExploreCommand, opts: GlobalOpts) -> Result<()> {
    let client = opts.client()?;

    let (tx, rx) = unbounded_channel();
    let mut tui = Tui::new()?;
    let app = App::new(client, tx.clone());

    let result = run_loop(&mut tui, app, tx, rx).await;

    // Restore the terminal explicitly (Drop also does, but surface errors here).
    drop(tui);
    Tui::restore()?;
    result
}

async fn run_loop(
    tui: &mut Tui,
    mut app: App,
    tx: UnboundedSender<Action>,
    mut rx: UnboundedReceiver<Action>,
) -> Result<()> {
    let mut ticker = interval(Duration::from_millis(250));

    // Initial draw so the user sees the frame before any input.
    draw(tui, &mut app)?;

    while app.running {
        tokio::select! {
            _ = ticker.tick() => {
                let _ = tx.send(Action::Tick);
            }
            maybe_event = tui.events.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if let Some(action) = map_event(event) {
                            let _ = tx.send(action);
                        }
                    }
                    Some(Err(e)) => return Err(Error::Io(e)),
                    None => app.running = false,
                }
            }
            Some(action) = rx.recv() => {
                app.handle_action(action);
                // Drain any actions queued in the same wake-up before drawing.
                while let Ok(action) = rx.try_recv() {
                    app.handle_action(action);
                }
            }
        }
        draw(tui, &mut app)?;
    }
    Ok(())
}

fn draw(tui: &mut Tui, app: &mut App) -> Result<()> {
    tui.terminal
        .draw(|frame| app.draw(frame))
        .map_err(Error::Io)?;
    Ok(())
}

fn map_event(event: Event) -> Option<Action> {
    match event {
        Event::Resize(_, _) => Some(Action::Resize),
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => Some(Action::ExpandSelected),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::CollapseSelected),
            _ => None,
        },
        _ => None,
    }
}
