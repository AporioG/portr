mod app;
mod process;
mod scanner;
mod services;
mod ui;

use app::{App, InputMode};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    panic,
    time::{Duration, Instant},
};

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
}

fn main() -> Result<()> {
    // Регистрация хука для защиты TTY при аварийном завершении рантайма
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('j') | KeyCode::Down => app.next(),
                            KeyCode::Char('k') | KeyCode::Up => app.previous(),
                            KeyCode::Char('r') => {
                                app.refresh();
                                app.status_message = "Refreshed".to_string();
                            }
                            KeyCode::Char('/') => {
                                app.input_mode = InputMode::Filtering;
                            }
                            KeyCode::Char('t') => {
                                app.kill_selected(libc::SIGTERM);
                            }
                            KeyCode::Char('K') => {
                                app.kill_selected(libc::SIGKILL);
                            }
                            _ => {}
                        },
                        InputMode::Filtering => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Backspace => {
                                app.filter.pop();
                            }
                            KeyCode::Char(c) => {
                                app.filter.push(c);
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.refresh();
            last_tick = Instant::now();
        }
    }

    restore_terminal();
    terminal.show_cursor()?;

    Ok(())
}
