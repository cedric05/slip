use crate::WorkOrPersonal;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use super::repolist::*;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
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

    fn select_0(&mut self) {
        self.state.select(Some(0));
    }

    fn go_ten_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.repolist.repos.len() - 10 {
                    self.repolist.repos.len() - 1
                } else {
                    i + 10
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn go_ten_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < 10 {
                    0
                } else {
                    i - 10
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn end(&mut self) {
        self.state.select(Some(self.repolist.repos.len() - 1));
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

pub fn main(
    category: Option<WorkOrPersonal>,
) -> Result<std::option::Option<Launch>, Box<dyn Error>> {
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

    Ok(res?)
}

pub enum LaunchType {
    LaunchShell,
    LaunchCode,
}

pub struct Launch {
    pub directory: String,
    pub launch_type: LaunchType,
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<Option<Launch>> {
    let last_tick = Instant::now();
    let matcher = SkimMatcherV2::default();
    loop {
        terminal.draw(|f| ui(f, &mut app, &matcher))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(None),
                    KeyCode::Left | KeyCode::Home => app.items.select_0(),
                    KeyCode::Right | KeyCode::End => app.items.end(),
                    KeyCode::Down | KeyCode::Tab => app.items.next(),
                    KeyCode::Up | KeyCode::BackTab => app.items.previous(),
                    KeyCode::PageDown => app.items.go_ten_down(),
                    KeyCode::PageUp => app.items.go_ten_up(),
                    KeyCode::Insert => {
                        let index = app.items.state.selected();
                        if let Some(index) = index {
                            if let Some(selected) = find_matches_in_order(
                                &app.items.repolist,
                                &app.search_text,
                                &matcher,
                            )
                            .into_iter()
                            .nth(index)
                            .map(|(repo, _)| repo)
                            {
                                return Ok(Some(Launch {
                                    directory: selected.location.clone(),
                                    launch_type: LaunchType::LaunchCode,
                                }));
                            }
                        }
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        let index = app.items.state.selected();
                        if let Some(index) = index {
                            if let Some(selected) = find_matches_in_order(
                                &app.items.repolist,
                                &app.search_text,
                                &matcher,
                            )
                            .into_iter()
                            .nth(index)
                            .map(|(repo, _)| repo)
                            {
                                return Ok(Some(Launch {
                                    directory: selected.location.clone(),
                                    launch_type: LaunchType::LaunchShell,
                                }));
                            }
                        }
                        return Ok(None);
                    }
                    KeyCode::Char(a) => {
                        app.search_text.push(a);
                        app.items.select_0();
                    }
                    KeyCode::Backspace => {
                        app.search_text.pop();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn find_matches_in_order<'a>(
    repolist: &'a RepoList,
    search_text: &'a str,
    matcher: &SkimMatcherV2,
) -> Vec<(&'a Repo, i64)> {
    let mut matched = repolist
        .repos
        .iter()
        .map(|repo| {
            let out = matcher.fuzzy_match(&repo.name, search_text);
            (repo, out)
        })
        .filter(|x| x.1.is_some())
        .map(|x| (x.0, x.1.unwrap()))
        .collect::<Vec<_>>();
    matched.sort_by(|a, b| b.1.cmp(&a.1));
    matched
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, matcher: &SkimMatcherV2) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(85),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .split(f.size());

    let input = Paragraph::new(app.search_text.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Filter repos"));
    f.render_widget(input, chunks[0]);

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> =
        find_matches_in_order(&app.items.repolist, &app.search_text, matcher)
            .into_iter()
            .map(|(repo, _order)| {
                let mut lines = vec![Spans::from(Span::styled(
                    format!("{} {}  {}", _order, repo.name, repo.category),
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
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let cheatsheet = Paragraph::new(Text::from(vec![
        Spans::from(vec![
            //Esc
            Span::styled("E", bold),
            Span::from("sc --> exit        "),
            // Enter
            Span::styled("E", bold),
            Span::from("nter --> change directory and shell    "),
            // Insert
            Span::styled("I", bold),
            Span::from("nsert --> open in vscode       "),
            // Any char
            Span::styled("A", bold),
            Span::from("ny char --> to search      "),
        ]),
        Spans::from(vec![
            // left
            Span::styled("L", bold),
            Span::from("eft --> go to top  "),
            // down
            Span::styled("D", bold),
            Span::from("own --> next                           "),
            // up
            Span::styled("U", bold),
            Span::from("p --> previous                 "),
            // backspace
            Span::styled("B", bold),
            Span::from("ackspace --> delete last character "),
        ]),
        // Spans::from(vec![
        //     // left
        //     Span::styled("P", bold),
        //     Span::from("age Down --> current + 10"),
        //     // down
        //     Span::styled("P", bold),
        //     Span::from("age Up --> current - 10"),
        //     // up
        //     Span::styled("H", bold),
        //     Span::from("ome --> get to top"),
        //     // backspace
        //     Span::styled("E", bold),
        //     Span::from("end --> get to bottom "),
        // ]),
    ]));
    f.render_widget(cheatsheet, chunks[2]);
}
