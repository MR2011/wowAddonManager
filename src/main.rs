mod addon_manager;
mod app;
mod curse;
mod settings;

use crate::addon_manager::AddonManager;
use crate::app::{App, LogLevel, Mode};
use crate::settings::Settings;
use crossterm::{
    cursor,
    event::{
        read, EnableMouseCapture, Event, KeyCode,
        KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{
        enable_raw_mode, EnterAlternateScreen,
    },
};
use std::{error::Error, io::stdout};
use tui::{backend::CrosstermBackend, Terminal};
extern crate config;

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let settings = Settings::new();
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(
        settings.paths.classic.clone(),
        settings.paths.retail.clone(),
        settings.paths.tbc.clone(),
    );

    match AddonManager::init_addon_db(&settings.paths.classic) {
        Ok(_) => app.log(
            "Classic addon directory successfully loaded.\n".to_string(),
            LogLevel::Info,
        ),
        Err(err) => app.log(
            format!("Couldn't load classic addon directory.\n{}\n", err),
            LogLevel::Error,
        ),
    }

    match AddonManager::init_addon_db(&settings.paths.retail) {
        Ok(_) => app.log(
            "Retail addon directory successfully loaded.\n".to_string(),
            LogLevel::Info,
        ),
        Err(err) => app.log(
            format!("Couldn't load retail addon directory.\n{}\n", err),
            LogLevel::Error,
        ),
    }

    let mut dialog_callback: Option<fn(app: &mut App)> = None;

    loop {
        terminal.draw(|mut f| {
            app.draw_app(&mut f);
        })?;

        if let Event::Key(code) = read()? {
            match app.mode {
                Mode::Normal => {
                    if code == settings.key_bindings.next_tab {
                        app.select_next_tab();
                    } else if code == settings.key_bindings.prev_tab {
                        app.select_prev_tab();
                    } else if code == settings.key_bindings.download_addon {
                        app.download();
                    } else if code == settings.key_bindings.remove_addon {
                        dialog_callback = Some(App::remove_addon);
                        app.add_dialog(
                            "Do you want to delete this addon?".to_string(),
                            true,
                        );
                    } else if code == settings.key_bindings.search_addon {
                        app.select_search();
                    } else if code == settings.key_bindings.next_table_item {
                        app.next_table_item();
                    } else if code == settings.key_bindings.prev_table_item {
                        app.prev_table_item();
                    } else if code
                        == settings.key_bindings.select_classic_version
                    {
                        app.select_classic();
                    } else if code
                        == settings.key_bindings.select_retail_version
                    {
                        app.select_retail();
                    } else if code == settings.key_bindings.select_tbc_version {
                        app.select_tbc();
                    } else if code == settings.key_bindings.scroll_down_log {
                        app.scroll_down_log();
                    } else if code == settings.key_bindings.scroll_up_log {
                        app.scroll_up_log();
                    } else if code == settings.key_bindings.quit {
                        break;
                    } else if code == settings.key_bindings.update_all_addons {
                        app.update_all();
                    } else if code == settings.key_bindings.update_addon {
                        app.update_addon();
                    }
                }
                Mode::Editing => match code {
                    KeyEvent {
                        code: KeyCode::Char('\n'),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        app.mode = Mode::Normal;
                        app.search(app.user_input.clone());
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        app.user_input.push(c);
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        app.user_input.pop();
                    }
                    KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        app.mode = Mode::Normal;
                        terminal.hide_cursor()?;
                    }
                    _ => {}
                },
                Mode::Dialog => match code {
                    KeyEvent {
                        code: KeyCode::Char('y'),
                        modifiers: KeyModifiers::NONE,
                    }
                    | KeyEvent {
                        code: KeyCode::Char('Y'),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        match dialog_callback {
                            Some(callback) => callback(&mut app),
                            None => (),
                        }
                        app.stop_dialog();
                    }
                    _ => {
                        app.stop_dialog();
                    }
                },
            }
        }
    }
    Ok(())
}
