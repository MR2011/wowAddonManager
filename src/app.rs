use crate::addon_manager::{Addon, AddonManager, Addons};
use crate::curse::CurseForgeAPI;
use std::collections::HashMap;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{
    Block, Borders, Clear, Paragraph, Row, Table, TableState, Tabs, Text,
};
use tui::Frame;

#[derive(Copy, Clone, PartialEq)]
pub enum Tab {
    Installed = 0,
    Search = 1,
}

impl Tab {
    pub fn from(i: usize) -> Option<Tab> {
        match i {
            0 => Some(Tab::Installed),
            1 => Some(Tab::Search),
            _ => None,
        }
    }

    pub fn len() -> usize {
        2
    }
}

#[derive(Copy, Clone)]
pub enum Version {
    Classic = 0,
    Tbc = 1,
    Retail = 2,
}

pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub enum Mode {
    Normal,
    Editing,
    Dialog,
}

pub struct TableItem {
    pub cells: Vec<String>,
    pub download_url: String,
    pub addon: Addon,
}

pub struct StatefulTable {
    state: TableState,
    items: Vec<TableItem>,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        StatefulTable {
            state: TableState::default(),
            items: Vec::new(),
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_selected(&self) -> Option<&TableItem> {
        match self.state.selected() {
            Some(i) => self.items.get(i),
            None => None,
        }
    }
}

pub struct Theme;

impl Theme {
    pub fn default() -> Style {
        Style::default().fg(Color::Gray)
    }

    pub fn hover() -> Style {
        Style::default().fg(Color::Magenta)
    }

    pub fn active() -> Style {
        Style::default().fg(Color::LightCyan)
    }
}
pub struct Dialog {
    text: String,
    confirmation: bool,
}

pub struct App {
    pub mode: Mode,
    pub user_input: String,
    pub tab_index: Tab,
    search_table: StatefulTable,
    installed_table: StatefulTable,
    selected_version: Version,
    log_scroll: u16,
    log_messages: Vec<(String, LogLevel)>,
    classic_path: String,
    retail_path: String,
    tbc_path: String,
    updates: Vec<Addon>,
    dialog: Option<Dialog>,
}

impl App {
    pub fn new(classic_path: String, retail_path: String, tbc_path: String) -> App {
        let mut app = App {
            mode: Mode::Normal,
            user_input: String::new(),
            tab_index: Tab::Installed,
            selected_version: Version::Classic,
            search_table: StatefulTable::new(),
            installed_table: StatefulTable::new(),
            log_scroll: 0,
            log_messages: Vec::new(),
            classic_path: classic_path,
            retail_path: retail_path,
            tbc_path: tbc_path,
            updates: Vec::new(),
            dialog: None,
        };
        app.load_installed_addons();
        app
    }

    pub fn draw_app<B>(&mut self, frame: &mut Frame<B>)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(frame.size());
        self.draw_header(frame, chunks[0]);
        match self.tab_index {
            Tab::Search => self.draw_search_tab(frame, chunks[1]),
            Tab::Installed => self.draw_installed_tab(frame, chunks[1]),
        };
        self.draw_footer(frame, chunks[2]);
        match &&self.dialog {
            Some(_) => self.draw_dialog(frame),
            None => (),
        }
    }

    pub fn draw_header<B>(&self, frame: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let tab_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [Constraint::Percentage(50), Constraint::Percentage(50)]
                    .as_ref(),
            )
            .split(area);
        let tab_index = vec!["Installed", "Search"];
        let tabs = Tabs::default()
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .titles(&tab_index)
            .select(self.tab_index as usize)
            .style(Theme::default())
            .highlight_style(Theme::active());
        frame.render_widget(tabs, tab_chunks[0]);
        let version_index = vec!["Classic", "Tbc", "Retail"];
        let versions = Tabs::default()
            .block(Block::default().borders(Borders::ALL).title("Version"))
            .titles(&version_index)
            .select(self.selected_version as usize)
            .style(Theme::default())
            .highlight_style(Theme::active());
        frame.render_widget(versions, tab_chunks[1]);
    }

    fn draw_installed_tab<B>(&mut self, frame: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let header = ["Status", "Name", "WoW", "Installed", "Available"];
        let rows = self
            .installed_table
            .items
            .iter()
            .map(|i| Row::StyledData(i.cells.iter(), Theme::default()));
        let table = Table::new(header.iter(), rows)
            .block(Block::default().title("Addons").borders(Borders::ALL))
            .header_style(Theme::active())
            .widths(&[
                Constraint::Percentage(15),
                Constraint::Percentage(35),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .style(Theme::default())
            .highlight_style(Theme::active())
            .highlight_symbol("> ");
        frame.render_stateful_widget(
            table,
            area,
            &mut self.installed_table.state,
        );
    }

    fn load_installed_addons(&mut self) {
        self.installed_table.items.clear();
        self.updates.clear();
        let path = match self.selected_version {
            Version::Classic => &self.classic_path,
            Version::Retail => &self.retail_path,
            Version::Tbc => &self.tbc_path,
        };
        let addons = match AddonManager::load_addon_db(&path) {
            Ok(a) => {
                self.log(
                    format!("Found {} installed addons.\n", a.addons.len()),
                    LogLevel::Info,
                );
                a
            }
            Err(err) => {
                self.log(
                    format!("Couldn't parse addons.\n{}\n", err),
                    LogLevel::Error,
                );
                Addons { addons: Vec::new() }
            }
        };
        let addon_ids: Vec<i32> = addons
            .addons
            .iter()
            .map(|a| a.addon_id.parse::<i32>().unwrap_or(0))
            .filter(|id| *id != 0)
            .collect();
        let mut updates: HashMap<String, Addon> = HashMap::new();
        if !addon_ids.is_empty() {
            updates = CurseForgeAPI::check_for_updates(
                addon_ids,
                self.selected_version,
            )
            .unwrap();
        }
        for addon in addons.addons.iter() {
            let download_url;
            let latest_version;
            let status;
            match updates.get(&addon.addon_id) {
                Some(update) => {
                    if update.file_id > addon.file_id {
                        download_url = update.download_url.clone();
                        latest_version = update.version.clone();
                        self.updates.push(update.clone());
                    } else {
                        download_url = addon.download_url.clone();
                        latest_version = addon.version.clone();
                    }
                }
                None => {
                    download_url = addon.download_url.clone();
                    latest_version = addon.version.clone();
                }
            }
            if latest_version == addon.version {
                status = "Up-to-date";
            } else {
                status = "Outdated";
            }
            self.installed_table.items.push(TableItem {
                cells: vec![
                    status.to_string(),
                    addon.name.clone(),
                    addon.game_version.clone(),
                    addon.version.clone(),
                    latest_version.clone(),
                ],
                download_url: download_url,
                addon: addon.clone(),
            })
        }
    }

    fn draw_search_tab<B>(&mut self, frame: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [Constraint::Percentage(10), Constraint::Percentage(90)]
                    .as_ref(),
            )
            .split(area);
        let text = [Text::raw(&self.user_input)];
        let input = Paragraph::new(text.iter()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(match self.mode {
                    Mode::Editing => Theme::active(),
                    _ => Theme::default(),
                })
                .title("Search"),
        );
        frame.render_widget(input, chunks[0]);
        let header = ["Name", "Game Version", "Date", "Downloads"];
        let rows = self
            .search_table
            .items
            .iter()
            .map(|i| Row::StyledData(i.cells.iter(), Theme::default()));
        let table = Table::new(header.iter(), rows)
            .block(Block::default().title("Addons").borders(Borders::ALL))
            .header_style(Theme::active())
            .widths(&[
                Constraint::Percentage(50),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
            ])
            .style(Theme::default())
            .highlight_style(Theme::active())
            .highlight_symbol("> ");
        frame.render_stateful_widget(
            table,
            chunks[1],
            &mut self.search_table.state,
        );
    }

    pub fn draw_footer<B>(&mut self, frame: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let mut text = self
            .log_messages
            .iter()
            .map(|(msg, log_level)| {
                let style;
                let prefix;
                match log_level {
                    LogLevel::Info => {
                        style = Style::default();
                        prefix = "[Info]";
                    }
                    LogLevel::Warning => {
                        style = Style::default().fg(Color::Yellow);
                        prefix = "[Warning]";
                    }
                    LogLevel::Error => {
                        style = Style::default().fg(Color::Red);
                        prefix = "[Error]";
                    }
                };
                Text::styled(format!("{} {}", prefix, msg), style)
            })
            .collect::<Vec<Text>>();
        text.reverse();
        let paragraph = Paragraph::new(text.iter())
            .block(Block::default().title("Log").borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(true)
            .scroll(self.log_scroll);
        frame.render_widget(paragraph, area);
    }

    pub fn draw_dialog<B>(&mut self, frame: &mut Frame<B>)
    where
        B: Backend,
    {
        let mut text =
            vec![Text::raw(self.dialog.as_ref().unwrap().text.clone())];
        if self.dialog.as_ref().unwrap().confirmation {
            text.push(Text::raw("\n(Y)es/(N)o"));
        }
        let paragraph = Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL).title("Warning"))
            .alignment(Alignment::Center);
        let area = self.centered_rect(50, 10, frame.size());
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
        self.mode = Mode::Dialog;
    }

    pub fn add_dialog(&mut self, text: String, confirmation: bool) {
        self.dialog = Some(Dialog {
            text: text,
            confirmation: confirmation,
        });
    }

    pub fn stop_dialog(&mut self) {
        self.mode = Mode::Normal;
        self.dialog = None;
    }

    pub fn search(&mut self, name: String) {
        let log_level;
        let msg;
        match CurseForgeAPI::search(&name, self.selected_version) {
            Ok(res) => {
                msg = format!("Found {} addons for {}.\n", res.len(), name);
                log_level = LogLevel::Info;
                self.search_table.items = res;
            }
            Err(err) => {
                msg = format!(
                    "Couldn't find any addons for {}.\n{}\n",
                    name, err
                );
                log_level = LogLevel::Error;
            }
        }
        self.log(msg, log_level);
    }

    pub fn download(&mut self) {
        if self.tab_index == Tab::Search {
            let item = self.search_table.get_selected().unwrap();
            let save_path = self.get_save_path();
            let log_level;
            let msg;
            if let Err(err) =
                CurseForgeAPI::download(&item.download_url, &save_path)
                    .and_then(|_| {
                        AddonManager::add_to_db(&save_path, item.addon.clone())
                    })
            {
                msg =
                    format!("Couldn't install {}.\n{}\n", &item.cells[0], err);
                log_level = LogLevel::Error;
            } else {
                msg = format!("{} successfully installed.\n", &item.cells[0]);
                log_level = LogLevel::Info;
            }
            self.log(msg, log_level);
        }
    }

    pub fn update_all(&mut self) {
        if self.tab_index == Tab::Installed {
            let save_path = self.get_save_path();
            for item in self.updates.clone().iter() {
                let msg;
                let log_level;
                if let Err(err) = AddonManager::delete(&save_path, &item)
                    .and_then(|_| {
                        CurseForgeAPI::download(&item.download_url, &save_path)
                    })
                    .and_then(|_| {
                        AddonManager::add_to_db(&save_path, item.clone())
                    })
                {
                    msg = format!(
                        "Couldn't update {}.\n{}\n",
                        item.name.clone(),
                        err
                    );
                    log_level = LogLevel::Error;
                } else {
                    msg = format!(
                        "{} successfully updated.\n",
                        item.name.clone()
                    );
                    log_level = LogLevel::Info;
                }
                self.log(msg, log_level);
            }
        }
    }

    pub fn update_addon(&mut self) {
        if self.tab_index == Tab::Installed {
            let save_path = self.get_save_path();
            let item = self.installed_table.get_selected().unwrap();
            let update = self
                .updates
                .iter()
                .find(|&u| u.addon_id == item.addon.addon_id)
                .unwrap();
            let name = &item.cells[1];
            let msg;
            let log_level;
            if let Err(err) = AddonManager::delete(&save_path, &item.addon)
                .and_then(|_| {
                    CurseForgeAPI::download(&item.download_url, &save_path)
                })
                .and_then(|_| {
                    AddonManager::add_to_db(&save_path, update.clone())
                })
            {
                msg = format!("Couldn't update {}.\n{}\n", name, err);
                log_level = LogLevel::Error;
            } else {
                msg = format!("{} successfully updated.\n", name);
                log_level = LogLevel::Info;
                self.load_installed_addons();
            }
            self.log(msg, log_level);
        }
    }

    pub fn remove_addon(&mut self) {
        if self.tab_index == Tab::Installed {
            let path = match self.selected_version {
                Version::Classic => &self.classic_path,
                Version::Retail => &self.retail_path,
                Version::Tbc => &self.tbc_path,
            };
            let item = self.installed_table.get_selected().unwrap();
            let msg;
            let log_level;
            match AddonManager::delete(&path, &item.addon) {
                Ok(_) => {
                    msg = format!("{} successfully deleted.\n", &item.cells[1]);
                    log_level = LogLevel::Info;
                }
                Err(err) => {
                    msg = format!(
                        "Couldn't delete {}.\n{}\n",
                        &item.cells[1], err
                    );
                    log_level = LogLevel::Error;
                }
            }
            self.load_installed_addons();
            self.log(msg, log_level);
        }
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    pub fn next_table_item(&mut self) {
        match self.tab_index {
            Tab::Search => {
                self.search_table.next();
            }
            Tab::Installed => {
                self.installed_table.next();
            }
        }
    }

    pub fn prev_table_item(&mut self) {
        match self.tab_index {
            Tab::Search => {
                self.search_table.previous();
            }
            Tab::Installed => {
                self.installed_table.previous();
            }
        }
    }

    pub fn select_classic(&mut self) {
        self.selected_version = Version::Classic;
        self.log("Switched to Classic.\n".to_string(), LogLevel::Info);
        self.refresh_view();
    }

    pub fn select_retail(&mut self) {
        self.selected_version = Version::Retail;
        self.log("Switched to Retail.\n".to_string(), LogLevel::Info);
        self.refresh_view();
    }

    pub fn select_tbc(&mut self) {
        self.selected_version = Version::Tbc;
        self.log("Switched to Tbc.\n".to_string(), LogLevel::Info);
        self.refresh_view();
    }

    pub fn scroll_up_log(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
    }

    pub fn scroll_down_log(&mut self) {
        if self.log_scroll < (self.log_messages.len() - 1) as u16 {
            self.log_scroll += 1;
        }
    }

    pub fn select_next_tab(&mut self) {
        let index = (self.tab_index as usize + 1).rem_euclid(Tab::len());
        self.tab_index = Tab::from(index).unwrap();
        match self.tab_index {
            Tab::Installed => self.load_installed_addons(),
            _ => (),
        };
    }

    pub fn select_prev_tab(&mut self) {
        let index =
            (self.tab_index as isize - 1).rem_euclid(Tab::len() as isize);
        self.tab_index = Tab::from(index as usize).unwrap();
        match self.tab_index {
            Tab::Installed => self.load_installed_addons(),
            _ => (),
        };
    }

    pub fn select_installed(&mut self) {
        self.load_installed_addons();
        self.tab_index = Tab::Installed;
    }

    pub fn select_search(&mut self) {
        self.mode = Mode::Editing;
        self.tab_index = Tab::Search;
    }

    pub fn log(&mut self, msg: String, log_level: LogLevel) {
        self.log_messages.push((msg, log_level));
    }

    pub fn refresh_view(&mut self) {
        match self.tab_index {
            Tab::Installed => self.load_installed_addons(),
            Tab::Search => self.search(self.user_input.clone()),
        };
    }

    pub fn get_save_path(&self) -> String {
        match self.selected_version {
            Version::Classic => self.classic_path.clone(),
            Version::Retail => self.retail_path.clone(),
            Version::Tbc => self.tbc_path.clone(),
        }
    }
}
