use std::error::Error;
use std::io;

use clap::Parser;
use ratatui::{
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::CrosstermBackend,
    Terminal,
};

use color_eyre::Result;

mod app;
mod component;
mod model;
mod ui;

use app::App;
use ui::view;

#[derive(Parser)]
struct Cli {
    sql_path: std::path::PathBuf,
    layer_path: std::path::PathBuf,
}

type Tui = Terminal<CrosstermBackend<io::Stdout>>;

fn run_app(terminal: &mut Tui, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| {
            view(f, &mut app.state, &app.input_arena).expect("View should always work")
        })?;

        app.handle_events()?;
        if app.exit {
            return Ok(());
        }
    }
}

fn init() -> io::Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    set_panic_hook();
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        hook(panic_info);
    }));
}

fn restore() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let args: Cli = Cli::parse();
    let mut terminal = init()?;

    // create app and run it
    let mut app = App::new(args.sql_path, args.layer_path)?;
    run_app(&mut terminal, &mut app)?;

    // restore terminal
    if let Err(err) = restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {err}"
        );
    };
    terminal.show_cursor()?;

    Ok(())
}
