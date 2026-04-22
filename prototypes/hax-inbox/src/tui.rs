use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap};
use ratatui::{Frame, Terminal};

use crate::inbox::{BadgeVariant, InboxItem, InboxKind};

const POLL_INTERVAL: Duration = Duration::from_millis(200);
const TAB_TITLES: [&str; 3] = ["CEREMONY PENDING", "NEEDS ATTENTION", "HANDLED TODAY"];

#[derive(Debug, Clone)]
pub struct TuiState {
    items: Vec<InboxItem>,
    active_tab: usize,
    selected: usize,
    open_item: Option<InboxItem>,
}

impl TuiState {
    pub fn new(items: Vec<InboxItem>) -> Self {
        Self {
            items,
            active_tab: 0,
            selected: 0,
            open_item: None,
        }
    }

    pub fn visible_items(&self) -> Vec<&InboxItem> {
        self.items
            .iter()
            .filter(|item| item_belongs_to_tab(item, self.active_tab))
            .collect()
    }

    fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % TAB_TITLES.len();
        self.selected = 0;
    }

    fn next_item(&mut self) {
        let visible_count = self.visible_items().len();
        if visible_count > 0 {
            self.selected = next_index(self.selected, visible_count);
        }
    }

    fn prev_item(&mut self) {
        let visible_count = self.visible_items().len();
        if visible_count > 0 {
            self.selected = previous_index(self.selected, visible_count);
        }
    }

    fn open_selected(&mut self) {
        if let Some(item) = self.visible_items().get(self.selected) {
            self.open_item = Some((*item).clone());
        }
    }

    fn close_item(&mut self) {
        self.open_item = None;
    }
}

pub fn run(items: Vec<InboxItem>) -> Result<()> {
    let mut terminal = enter_terminal()?;
    let mut state = TuiState::new(items);
    let result = event_loop(&mut terminal, &mut state);
    leave_terminal(&mut terminal)?;
    result
}

pub fn render_snapshot(items: Vec<InboxItem>) -> Result<String> {
    let state = TuiState::new(items);
    let mut output = String::new();

    output.push_str("=== HAX INBOX SNAPSHOT ===\n");
    output.push_str(&format!("Active tab: {}\n", TAB_TITLES[state.active_tab]));
    output.push_str("Tabs: CEREMONY PENDING | NEEDS ATTENTION | HANDLED TODAY\n");

    let visible = state.visible_items();
    output.push_str(&format!("Items in tab: {}\n", visible.len()));

    for (index, item) in visible.iter().enumerate() {
        let marker = if index == state.selected { ">" } else { " " };
        output.push_str(&format!(
            "{} [{}] {} - {} ({})\n",
            marker, item.badge, item.title, item.snippet, item.source_agent
        ));
    }

    output.push_str("Ctrl-I tab | Ctrl-J/Enter open | Up/Down select | Esc close\n");
    Ok(output)
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut TuiState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            render_ui(frame, state);
        })?;

        if event::poll(POLL_INTERVAL)? {
            if let Event::Key(key) = event::read()? {
                if is_ctrl_i(key) {
                    state.next_tab();
                } else if is_open_key(key) {
                    state.open_selected();
                } else if key.code == KeyCode::Esc {
                    if state.open_item.is_none() {
                        break;
                    } else {
                        state.close_item();
                    }
                } else if key.code == KeyCode::Up {
                    state.prev_item();
                } else if key.code == KeyCode::Down {
                    state.next_item();
                }
            }
        }
    }

    Ok(())
}

fn render_ui(frame: &mut Frame<'_>, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_tabs(frame, chunks[0], state);
    render_items(frame, chunks[1], state);
    render_footer(frame, chunks[2], state);
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let tabs = Tabs::new(TAB_TITLES.iter().copied())
        .select(state.active_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

fn render_items(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let visible = state.visible_items();
    let rows = visible
        .iter()
        .enumerate()
        .map(|(index, item)| row_for_item(index, state.selected, item))
        .collect::<Vec<_>>();
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Queue"))
        .highlight_symbol(">> ");
    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let text = state
        .open_item
        .as_ref()
        .map(open_item_text)
        .unwrap_or_else(default_footer_text);
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Action"))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn row_for_item(index: usize, selected: usize, item: &InboxItem) -> ListItem<'static> {
    let marker = if index == selected { ">" } else { " " };
    let line = Line::from(vec![
        Span::styled(marker, Style::default().fg(Color::LightCyan)),
        Span::raw(" "),
        Span::styled(format!("[{}] ", item.badge), badge_style(item.badge)),
        Span::styled(item.title.clone(), Style::default().fg(Color::White)),
        Span::raw(format!(" - {} ({})", item.snippet, item.source_agent)),
    ]);
    ListItem::new(line)
}

fn open_item_text(item: &InboxItem) -> String {
    format!(
        "OPEN {} | {} | {} | {}",
        item.id, item.kind, item.title, item.snippet
    )
}

fn default_footer_text() -> String {
    "Ctrl-I tab | Ctrl-J/Enter open | Up/Down select | Esc close".to_string()
}

fn badge_style(badge: BadgeVariant) -> Style {
    match badge {
        BadgeVariant::NeedsYou => Style::default().fg(Color::LightRed),
        BadgeVariant::Review => Style::default().fg(Color::LightYellow),
        BadgeVariant::Handled => Style::default().fg(Color::LightGreen),
        BadgeVariant::Ceremony => Style::default().fg(Color::LightMagenta),
        BadgeVariant::Anchor => Style::default().fg(Color::LightCyan),
    }
}

fn item_belongs_to_tab(item: &InboxItem, tab: usize) -> bool {
    match tab {
        0 => item.kind == InboxKind::CeremonyPending,
        1 => item.kind == InboxKind::NeedsAttention || item.kind == InboxKind::Informational,
        _ => item.badge == BadgeVariant::Handled,
    }
}

fn next_index(selected: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (selected + 1) % len
    }
}

fn previous_index(selected: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        selected.checked_sub(1).unwrap_or(len - 1)
    }
}

fn is_ctrl_i(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('i') && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn is_open_key(key: KeyEvent) -> bool {
    key.code == KeyCode::Enter
        || (key.code == KeyCode::Char('j') && key.modifiers.contains(KeyModifiers::CONTROL))
}

fn enter_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(io::stdout())).map_err(Into::into)
}

fn leave_terminal<B: Backend>(terminal: &mut Terminal<B>) -> Result<()>
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
