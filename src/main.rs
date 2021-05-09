mod addon_manager;
mod app;
mod curse;
#[allow(dead_code)]
mod event;
mod settings;

use crate::addon_manager::AddonManager;
use crate::app::{App, LogLevel, Mode};
use crate::event::{Event, Events};
use crate::settings::Settings;
use std::{
    error::Error,
    io::{self},
};
use termion::{
    event::Key, input::MouseTerminal, raw::IntoRawMode,
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    Terminal,
};
extern crate config;

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let settings = Settings::new();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup event handlers
    let mut events = Events::new();

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

        // write!(
        //     terminal.backend_mut(),
        //     "{}",
        //     Goto(3 + app.userInput.width() as u16, 3)
        // )?;
        if let Event::Input(input) = events.next()? {
            match app.mode {
                Mode::Normal => {
                    if input == settings.key_bindings.next_tab {
                        app.select_next_tab();
                    } else if input == settings.key_bindings.prev_tab {
                        app.select_prev_tab();
                    // app.mode = Mode::Editing;
                    // terminal.show_cursor()?;
                    // events.disable_exit_key();
                    } else if input == settings.key_bindings.download_addon {
                        app.download();
                    } else if input == settings.key_bindings.remove_addon {
                        dialog_callback = Some(App::remove_addon);
                        app.add_dialog(
                            "Do you want to delete this addon?".to_string(),
                            true,
                        );
                    } else if input == settings.key_bindings.search_addon {
                        // app.select_search_box();
                        app.select_search();
                        // terminal.show_cursor()?;
                        events.disable_exit_key();
                    } else if input == settings.key_bindings.next_table_item {
                        app.next_table_item();
                    } else if input == settings.key_bindings.prev_table_item {
                        app.prev_table_item();
                    } else if input
                        == settings.key_bindings.select_classic_version
                    {
                        app.select_classic();
                    } else if input
                        == settings.key_bindings.select_retail_version
                    {
                        app.select_retail();
                    } else if input
                        == settings.key_bindings.select_tbc_version
                    {
                        app.select_tbc();
                    } else if input == settings.key_bindings.scroll_down_log {
                        app.scroll_down_log();
                    } else if input == settings.key_bindings.scroll_up_log {
                        app.scroll_up_log();
                    } else if input == settings.key_bindings.quit {
                        break;
                    } else if input == settings.key_bindings.update_all_addons {
                        app.update_all();
                    } else if input == settings.key_bindings.update_addon {
                        app.update_addon();
                    }
                }
                Mode::Editing => match input {
                    Key::Char('\n') => {
                        app.mode = Mode::Normal;
                        app.search(app.user_input.clone());
                    }
                    Key::Char(c) => {
                        app.user_input.push(c);
                    }
                    Key::Backspace => {
                        app.user_input.pop();
                    }
                    Key::Esc => {
                        app.mode = Mode::Normal;
                        terminal.hide_cursor()?;
                        events.enable_exit_key();
                    }
                    _ => {}
                },
                Mode::Dialog => match input {
                    Key::Char('y') | Key::Char('Y') => {
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
