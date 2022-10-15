use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use cs::{Event as CsEvent, State};
use std::{
    error::Error,
    io,
    os::unix::process::CommandExt,
    process::Command,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let mut state = State::new();
    let res = run_app(&mut terminal, &mut state, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
        return Ok(());
    }
    let default = if cfg!(target_os = "linux") {
        "bash"
    } else if cfg!(target_os = "macos") {
        "zsh"
    } else {
        panic!("Unsupported OS");
    };
    Command::new(std::env::var("CS_SHELL").unwrap_or(default.to_owned())).exec();
    // std::process::exit(0);
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut State,
    tick_rate: Duration,
) -> io::Result<()> {
    let last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Left => {
                        state.update(CsEvent::Left);
                    }
                    KeyCode::Down => {
                        state.update(CsEvent::Down);
                    }
                    KeyCode::Up => {
                        state.update(CsEvent::Up);
                    }
                    KeyCode::Right => {
                        state.update(CsEvent::Right);
                    }
                    KeyCode::Enter => {
                        state.update(CsEvent::Right);
                        std::env::set_current_dir(state.get_current_dir())?;
                        break;
                    }
                    KeyCode::Char(c) => {
                        state.search.push(c);
                        state.update(CsEvent::Search);
                    }
                    KeyCode::Backspace => {
                        state.search.pop();
                        state.update(CsEvent::Search);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        // .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    // We can now render the item list
    ui_search(f, chunks[0], state);
    ui_list(f, chunks[1], state);
    ui_help(f, chunks[2], state);
}

fn ui_search<B: Backend>(f: &mut Frame<B>, rect: Rect, state: &mut State) {
    let input = Paragraph::new(state.search.as_ref())
        .style(match state.search_mode {
            _ => Style::default(),
            // true => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(input, rect);
    f.set_cursor(
        // Put cursor past the end of the input text
        rect.x + state.search.width() as u16 + 1,
        // Move one line down, from the border to the input line
        rect.y + 1,
    )
}

fn ui_list<B: Backend>(f: &mut Frame<B>, rect: Rect, state: &mut State) {
    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = state
        .get_files()
        .iter()
        .map(|path| {
            let lines = vec![Spans::from(
                path.file_name().unwrap().to_str().unwrap().to_owned(),
            )];
            // for _ in 0..i.1 {
            //     lines.push(Spans::from(Span::styled(
            //         "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            //         Style::default().add_modifier(Modifier::ITALIC),
            //     )));
            // }
            ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(items, rect, &mut state.list);
}

fn ui_help<B: Backend>(f: &mut Frame<B>, rect: Rect, state: &mut State) {
    let (msg, style) = match state.search_mode {
        false => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        true => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter the selected dir"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, rect);
}
