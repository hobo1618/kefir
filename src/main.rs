use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{prelude::*, widgets::*};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    fn del_selected(&mut self) {
        if let Some(i) = self.state.selected() {
            self.items.remove(i);
            self.state.select(Some(i.saturating_sub(1)));
        }
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is
/// a wrapper around `ListState`. Keeping track of the items state let us render the associated
/// widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App<'a> {
    items: StatefulList<(&'a str, usize, Status)>,
    events: Vec<(&'a str, &'a str)>,
}

#[derive(Copy, Clone, PartialEq)]
enum Status {
    ToDo,
    UpNext,
    InProgress,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(vec![
                ("Item0", 1, Status::ToDo),
                ("Item1", 2, Status::ToDo),
                ("Item2", 1, Status::ToDo),
                ("Item3", 3, Status::ToDo),
                ("Item4", 1, Status::ToDo),
                ("Item5", 4, Status::ToDo),
                ("Item6", 1, Status::ToDo),
                ("Item7", 3, Status::ToDo),
                ("Item8", 1, Status::ToDo),
                ("Item9", 6, Status::ToDo),
                ("Item10", 1, Status::InProgress),
                ("Item11", 3, Status::InProgress),
                ("Item12", 1, Status::InProgress),
                ("Item13", 2, Status::InProgress),
                ("Item14", 1, Status::InProgress),
                ("Item15", 1, Status::InProgress),
                ("Item16", 4, Status::InProgress),
                ("Item17", 1, Status::InProgress),
                ("Item18", 5, Status::InProgress),
                ("Item19", 4, Status::InProgress),
                ("Item20", 1, Status::InProgress),
                ("Item21", 2, Status::UpNext),
                ("Item22", 1, Status::UpNext),
                ("Item23", 3, Status::UpNext),
            ]),
            events: vec![
                ("Event1", "INFO"),
                ("Event2", "INFO"),
                ("Event3", "CRITICAL"),
                ("Event4", "ERROR"),
                ("Event5", "INFO"),
                ("Event6", "INFO"),
                ("Event7", "WARNING"),
                ("Event8", "INFO"),
                ("Event9", "INFO"),
                ("Event10", "INFO"),
                ("Event11", "CRITICAL"),
                ("Event12", "INFO"),
                ("Event13", "INFO"),
                ("Event14", "INFO"),
                ("Event15", "INFO"),
                ("Event16", "INFO"),
                ("Event17", "ERROR"),
                ("Event18", "ERROR"),
                ("Event19", "INFO"),
                ("Event20", "INFO"),
                ("Event21", "WARNING"),
                ("Event22", "INFO"),
                ("Event23", "INFO"),
                ("Event24", "WARNING"),
                ("Event25", "INFO"),
                ("Event26", "INFO"),
            ],
        }
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    fn on_tick(&mut self) {
        let event = self.events.remove(0);
        self.events.push(event);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
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
        println!("{err:?}");
    }

    Ok(())
}

fn next_status(status: Status) -> Status {
    match status {
        Status::ToDo => Status::UpNext,
        Status::UpNext => Status::InProgress,
        Status::InProgress => Status::ToDo,
    }
}

fn prev_status(status: Status) -> Status {
    match status {
        Status::ToDo => Status::InProgress,
        Status::InProgress => Status::UpNext,
        Status::UpNext => Status::ToDo,
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut active_column: Status = Status::ToDo;

    loop {
        terminal.draw(|f| ui(f, &mut app, &mut active_column))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Left => app.items.unselect(),
                        KeyCode::Down => app.items.next(),
                        KeyCode::Char('j') => app.items.next(),
                        KeyCode::Char('k') => app.items.previous(),
                        KeyCode::Char('l') => {
                            active_column = next_status(active_column);
                        }
                        KeyCode::Char('h') => {
                            active_column = prev_status(active_column);
                        }
                        KeyCode::Char('x') => app.items.del_selected(),
                        KeyCode::Up => app.items.previous(),
                        _ => {}
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, active_column: &mut Status) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let to_do: Vec<ListItem> = app
        .items
        .items
        .iter()
        .filter(|i| i.2 == Status::UpNext)
        .map(|i| {
            let mut lines = vec![Line::from(i.0)];
            for _ in 0..i.1 {
                lines.push("Something important to do".italic().into());
            }
            ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let to_do =
        List::new(to_do)
            .block(Block::default().borders(Borders::ALL).title("To Do").style(
                Style::default().bg(if *active_column == Status::ToDo {
                    Color::Yellow
                } else {
                    Color::Black
                }),
            ))
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(to_do, chunks[0], &mut app.items.state);

    // Iterate through all elements in the `items` app and append some debug text to it.
    let up_next: Vec<ListItem> = app
        .items
        .items
        .iter()
        .filter(|i| i.2 == Status::ToDo)
        .map(|i| {
            let mut lines = vec![Line::from(i.0)];
            for _ in 0..i.1 {
                lines.push("Something important to do".italic().into());
            }
            ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let up_next = List::new(up_next)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Up Next")
                .style(Style::default().bg(if *active_column == Status::UpNext {
                    Color::Yellow
                } else {
                    Color::Black
                })),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(up_next, chunks[1], &mut app.items.state);

    // Iterate through all elements in the `items` app and append some debug text to it.
    let in_progress: Vec<ListItem> = app
        .items
        .items
        .iter()
        .filter(|i| i.2 == Status::InProgress)
        .map(|i| {
            let mut lines = vec![Line::from(i.0)];
            for _ in 0..i.1 {
                lines.push("Something important to do".italic().into());
            }
            ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
        })
        .collect();

    let in_progress = List::new(in_progress)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("In Progress")
                .style(
                    Style::default().bg(if *active_column == Status::InProgress {
                        Color::Yellow
                    } else {
                        Color::Black
                    }),
                ),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("&& ");
    f.render_stateful_widget(in_progress, chunks[2], &mut app.items.state);
}
