use crate::key::Key;
use crate::{
    arb_feed::*,
    events::{Events, InputEvent},
    get_and_parse_arb_feed,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use spinners::{Spinner, Spinners};
use std::{
    borrow::{Borrow, BorrowMut},
    sync::Arc,
};
use std::{
    error::Error,
    io::{self, Stdout},
    os::unix::thread,
    // thread::sleep,
    time::{Duration, Instant},
};
use std::{marker::Send, vec};
use tmev_protos::tmev_proto::Bundle;
use tokio::sync::MutexGuard;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tokio::time::sleep;
// use tokio_util::task::LocalPoolHandle;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs, Wrap},
    Frame, Terminal,
};

pub struct App {
    state: TableState,
    title: String,
    tabs: TabsState,
    arbs: Vec<Vec<String>>,
    bundle: FullBundleTable,
    analytics: AnalyticsTable,
}
// unsafe impl Send for App {}
// unsafe impl Sync for App {}
impl App {
    pub fn new(
        title: String,
        rows: Vec<Vec<String>>,
        bundle_vec: Vec<Vec<String>>,
        analytics: Vec<Vec<String>>,
    ) -> App {
        App {
            title,
            state: TableState::default(),
            arbs: rows,
            bundle: FullBundleTable {
                title: "Bundles Processed".to_string(),
                state: TableState::default(),
                sent_bundles: bundle_vec,
            },
            tabs: TabsState::new(vec![
                "Arbs".to_string(),
                "Bundles".to_string(),
                "Your Bundles".to_string(),
            ]),
            analytics: AnalyticsTable {
                title: "Your Bundles".to_string(),
                state: TableState::default(),
                items: analytics,
            },
        }
    }
    pub fn next(&mut self) {
        // println!("tab index {:?}", self.tabs.index);
        match self.tabs.index {
            0 => {
                let i = match self.state.selected() {
                    Some(i) => {
                        if i >= self.arbs.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.state.select(Some(i));
            }
            1 => {
                let i = match self.bundle.state.selected() {
                    Some(i) => {
                        if i >= self.bundle.sent_bundles.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                // println!("");
                self.bundle.state.select(Some(i));
            }
            2 => {
                let i = match self.analytics.state.selected() {
                    Some(i) => {
                        if i >= self.analytics.items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.analytics.state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn previous(&mut self) {
        match self.tabs.index {
            0 => {
                let i = match self.state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.arbs.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.state.select(Some(i));
            }

            1 => {
                let i = match self.bundle.state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.bundle.sent_bundles.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.bundle.state.select(Some(i));
            }
            2 => {
                let i = match self.bundle.state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.bundle.sent_bundles.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.bundle.state.select(Some(i));
            }
            _ => {}
        }
    }
    pub fn go_to_explorer(&mut self) {
        if self.tabs.index == 1 {
            return;
        }
        let row_index = self.state.selected().unwrap();
        let row = self.arbs.get(row_index).unwrap();
        let mut explorer = "https://explorer.solana.com/tx/".to_string().to_owned();
        explorer.push_str(row.get(2).unwrap());

        open::that(explorer).unwrap();
    }
    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }
    // pub fn on_tick(&mut self){

    // }
}
pub struct TabsState {
    pub titles: Vec<String>,
    pub index: usize,
}

impl TabsState {
    pub fn new(titles: Vec<String>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
        // else {
        //     self.index = self.titles.len() - 1;
        // }
    }
}

// Sep table struct for all bundles in the bundle tab
pub struct FullBundleTable {
    title: String,
    state: TableState,
    sent_bundles: Vec<Vec<String>>,
}
impl FullBundleTable {
    pub fn new(sent_bundles: Vec<Vec<String>>) -> FullBundleTable {
        FullBundleTable {
            title: "Bundles Processed".to_string(),
            state: TableState::default(),
            sent_bundles,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.sent_bundles.len() - 1 {
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
                    self.sent_bundles.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn on_tick(&mut self, new_bundle: Vec<String>) {
        self.sent_bundles.push(new_bundle)
    }
}

pub struct AnalyticsTable {
    title: String,
    state: TableState,
    items: Vec<Vec<String>>,
}

impl AnalyticsTable {
    pub fn new(items: Vec<Vec<String>>) -> AnalyticsTable {
        AnalyticsTable {
            title: "Bundles Processed".to_string(),
            state: TableState::default(),
            items,
        }
    }
    // pub fn next(&mut self) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i >= self.items.len() - 1 {
    //                 0
    //             } else {
    //                 i + 1
    //             }
    //         }
    //         None => 0,
    //     };
    //     self.state.select(Some(i));
    // }

    // pub fn previous(&mut self) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i == 0 {
    //                 self.items.len() - 1
    //             } else {
    //                 i - 1
    //             }
    //         }
    //         None => 0,
    //     };
    //     self.state.select(Some(i));
    // }
}
//this is our "start_ui" from the monkey blog
pub async fn display_table(
    rows: Vec<Vec<String>>,
    bundle_vec: Vec<Vec<String>>,
    analytics: Vec<Vec<String>>,
) -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new(
        "JITO SEARCHER TERMINAL 🤑".to_string(),
        rows,
        bundle_vec,
        analytics,
    );
    // let app = Arc::clone(&app);
    let res = run_app(&mut terminal, &mut app).await;
    //let events = Events::new(Duration::from_millis(200));

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

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let text = vec![Spans::from("quit - [q] | reload - [r] | tab-change - [↔]")];
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Legend",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
async fn run_app<B: Backend + std::marker::Send>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    // let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<Vec<String>>>(9000);
    // let local_set = LocalPoolHandle::new(1);
    // let tx2 = tx.clone();
    // println!("enter");
    // local_set.spawn_pinned(async move || loop {
    //     // panic!("fuck");
    //     sleep(Duration::from_millis(5000)).await;
    //     // let mut sp = Spinner::new(Spinners::Dots8Bit, " updating".into());
    //     let resp = get_and_parse_arb_feed().await.unwrap();
    //     tx.send(resp).await.unwrap();
    //     // sp.stop();
    // });
    // let mut newitems: Vec<Vec<String>> = Vec::new();
    // newitems = app.items.clone();
    let events = Events::new(Duration::from_millis(200));
    loop {
        // app.items = newitems.clone();
        terminal.draw(|f| draw(f, app))?;
        if let InputEvent::Input(i) = events.next().unwrap() {
            match i {
                Key::Char('q') => break,
                Key::Down => app.next(),
                Key::Up => app.previous(),
                Key::Right => app.on_right(),
                Key::Left => app.on_left(),
                Key::Char('r') => {
                    terminal.clear().unwrap();
                    sleep(Duration::from_millis(500));
                    continue;
                }
                Key::Enter => app.go_to_explorer(),
                _ => {} // continue;
            }
        }
    }
    Ok(())
}
//let (tx2, mut rx2) = unbounded_channel();
// update file or redis
// updates app on every render
// tokio::spawn(run_bundle_request_loop(tx2));
// tokio::spawn(async move {
//     if let Some(new_bundles) = rx2.recv().await {
//         for new_bundle in new_bundles {
//             app.bundle.on_tick(new_bundle);
//         }
//     }
// });
//part that gives error because we are moving app into a diff thread closure
// and we need to do this because running recv on main thread would block input and ui render

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // let app = app.lock().await;
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.title.clone()),
        )
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Green))
        .select(app.tabs.index);
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Footer",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_first_tab(f, app, chunks[1]),
        1 => draw_second_tab(f, app, chunks[1]),
        2 => draw_third_tab(f, app, chunks[1]),
        _ => {}
    };
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(20),
                Constraint::Length(3),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    ui(f, app, chunks[0]);
    draw_text(f, chunks[1]);
}
fn draw_second_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(20),
                Constraint::Length(3),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    draw_full_bundles_table(f, app, area)
    // draw_text(f, chunks[1]);
}
fn draw_third_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(20),
                Constraint::Length(3),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    draw_analytics_table(f, app, area)
}
//draws our table
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    // println!("redraw :{:?}", app.items.len());
    // let rects = Layout::default()
    //     .constraints([Constraint::Percentage(70)].as_ref())
    //     .margin(2)
    //     .split(area);

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
    let rows = app.arbs.iter().map(|item| {
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
    f.render_stateful_widget(t, area, &mut app.state);
}

fn draw_full_bundles_table<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(area);
    // let block = Block::default().borders(Borders::ALL).title("Graphs");
    // f.render_widget(block, area);

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::LightGreen);
    let header_cells = ["slot", "searcher_key", "uuid", "tip_amt"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.bundle.sent_bundles.iter().map(|item| {
        // let item = vec![
        //     item.uuid.clone(),
        //     item.searcher_key.clone(),
        //     item.transaction_hash.clone(),
        // ];
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Latest Bundles"),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(10),
            // Constraint::Percentage(10),
        ])
        .column_spacing(1);
    f.render_stateful_widget(t, area, &mut app.bundle.state);
}

fn draw_analytics_table<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(area);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::LightGreen);
    let header_cells = ["slot", "searcher_key", "uuid", "tip_amt"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows = app.analytics.items.iter().map(|item| {
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
        .block(Block::default().borders(Borders::ALL).title("Your Bundles"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(5),
            // Constraint::Percentage(10),
        ])
        .column_spacing(1);
    f.render_stateful_widget(t, area, &mut app.bundle.state);
}
