use crate::{arb_feed::*, get_and_parse_arb_feed};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use spinners::{Spinner, Spinners};
use std::sync::Arc;
use std::{
    error::Error,
    io::{self, Stdout},
    os::unix::thread,
    // thread::sleep,
    time::Duration,
};
use std::{marker::Send, vec};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_util::task::LocalPoolHandle;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

pub struct App {
    state: TableState,
    items: Vec<Vec<String>>,
}
// unsafe impl Send for App {}
// unsafe impl Sync for App {}
impl App {
    pub fn new(rows: Vec<Vec<String>>) -> App {
        App {
            state: TableState::default(),
            items: rows,
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
    pub fn go_to_explorer(&mut self) {
        let row_index = self.state.selected().unwrap();
        let row = self.items.get(row_index).unwrap();
        let mut explorer = "https://explorer.solana.com/tx/".to_string().to_owned();
        explorer.push_str(row.get(2).unwrap());

        open::that(explorer).unwrap();
    }
}

pub async fn display_table(
    rows: Vec<Vec<String>>,
) -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(rows);
    let res = run_app(&mut terminal, app).await;

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

    Ok(terminal)
}

async fn run_app<B: Backend + std::marker::Send>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<Vec<String>>>(9000);
    let local_set = LocalPoolHandle::new(1);
    // let tx2 = tx.clone();
    // println!("enter");
    local_set.spawn_pinned(async move || loop {
        // panic!("fuck");
        sleep(Duration::from_millis(5000)).await;
        // let mut sp = Spinner::new(Spinners::Dots8Bit, " updating".into());
        let resp = get_and_parse_arb_feed().await.unwrap();
        tx.send(resp).await.unwrap();
        // sp.stop();
    });
    // let mut newitems: Vec<Vec<String>> = Vec::new();
    // newitems = app.items.clone();
    Ok(loop {
        // app.items = newitems.clone();
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Char('r') => {
                    terminal.clear();
                    sleep(Duration::from_millis(500));
                    continue;
                }
                KeyCode::Enter => app.go_to_explorer(),
                _ => {
                    if let Some(mut msg) = rx.recv().await {
                        if msg.len() > 0 {
                            // terminal.clear();
                            // msg.push(vec![
                            //     "hello1".to_string(),
                            //     "2".to_string(),
                            //     "3".to_string(),
                            //     "4".to_string(),
                            // ]);
                            // newitems = vec![vec![String::from("something")]];
                            continue;
                            // app.state.select(index)
                            // println!("reaches here");
                        }
                    }
                    // continue;
                }
            }
        }
    })
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // println!("redraw :{:?}", app.items.len());
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(2)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::LightGreen);
    let header_cells = [
        "blocktime",
        "slot_id",
        "txn_hash",
        "profit_amt",
        "currency",
        "signer",
        "price_usd",
        "profit_usd",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.clone()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Latest Arbs"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Length(10),
            Constraint::Min(10),
            Constraint::Percentage(20),
            Constraint::Length(10),
            Constraint::Min(10),
            Constraint::Percentage(10),
            Constraint::Length(20),
            // Constraint::Min(10),
        ])
        .column_spacing(1);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}
