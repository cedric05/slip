use crate::{repolist::Repo, WorkOrPersonal};

use super::repolist::RepoList;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    process::Command,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

struct StatefulList {
    state: ListState,
    repolist: RepoList,
}

impl StatefulList {
    fn with_items(repolist: RepoList) -> StatefulList {
        let mut state = ListState::default();
        state.select(Some(0));
        StatefulList { state, repolist }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.repolist.repos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.repolist.repos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App {
    items: StatefulList,
    search_text: String,
    category: Option<WorkOrPersonal>,
}

impl App {
    fn new(repolist: RepoList, category: Option<WorkOrPersonal>) -> App {
        App {
            items: StatefulList::with_items(repolist),
            search_text: String::new(),
            category,
        }
    }
}

pub fn main(category: Option<WorkOrPersonal>) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let repolist = RepoList::get_config()?;
    let app = App::new(repolist, category);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Enter => {
                        let index = app.items.state.selected();
                        if let Some(index) = index {
                            let selected = app
                                .items
                                .repolist
                                .repos
                                .iter()
                                .filter(|repo| filter_category(repo, &app.category))
                                .filter(|repo| filter_search_text(repo, &app.search_text))
                                .nth(index);
                            if let Some(selected) = selected {
                                let mut execute_command;
                                let clone_command_str = format!("code {}", selected.location);
                                if cfg!(target_os = "windows") {
                                    execute_command = Command::new("cmd");
                                    execute_command.args(["/C", &clone_command_str]);
                                } else {
                                    execute_command = Command::new("sh");
                                    execute_command.args(["-c", &clone_command_str]);
                                };
                                execute_command.spawn()?;
                                return Ok(());
                            }
                        }
                        return Ok(());
                    }
                    KeyCode::Char(a) => app.search_text.push(a),
                    KeyCode::Backspace => {
                        app.search_text.pop();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn filter_search_text(repo: &Repo, search_text: &str) -> bool {
    repo.name
        .to_lowercase()
        .contains(&search_text.to_lowercase())
}

fn filter_category(repo: &Repo, category: &Option<WorkOrPersonal>) -> bool {
    match category {
        Some(category) => category == &repo.category,
        None => true,
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(f.size());

    let input = Paragraph::new(app.search_text.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Filter repos"));
    f.render_widget(input, chunks[0]);

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .repolist
        .repos
        .iter()
        .filter(|repo| filter_category(repo, &app.category))
        .filter(|repo| filter_search_text(repo, &app.search_text))
        .map(|repo| {
            let mut lines = vec![Spans::from(Span::styled(
                format!("{}  {}", repo.name, repo.category),
                Style::default().add_modifier(Modifier::BOLD),
            ))];
            lines.push(Spans::from(Span::styled(
                repo.location.clone(),
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::UNDERLINED)
                    .add_modifier(Modifier::ITALIC),
            )));
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Repos"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[1], &mut app.items.state);

    // Let's do the same for the events.
    // The event list doesn't have any state and only displays the current state of the list.
}
