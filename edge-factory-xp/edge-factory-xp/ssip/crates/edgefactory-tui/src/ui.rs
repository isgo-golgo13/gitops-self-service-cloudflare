//! Screens: choose (name, zone, hostname, tier, exposure, database, objects, container) ->
//! origins (hybrid only) -> review -> result (file written) or watch (xp/demo).

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use serde_json::{json, Value};
use edgefactory_core::api::{AppRequest, AppStatus, Catalog};

use crate::client::{Client, Outcome};
use crate::Args;

const RED: Color = Color::Rgb(215, 38, 44);
const GOLD: Color = Color::Rgb(201, 162, 39);
const GREY: Color = Color::Rgb(138, 143, 152);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen { Choose, Origins, Review, Done, Watch }

struct Chooser { label: &'static str, options: Vec<String>, state: ListState }
impl Chooser {
    fn new(label: &'static str, options: Vec<String>) -> Self {
        let mut state = ListState::default();
        if !options.is_empty() { state.select(Some(0)); }
        Self { label, options, state }
    }
    fn value(&self) -> String { self.state.selected().and_then(|i| self.options.get(i)).cloned().unwrap_or_default() }
    fn next(&mut self) { let n = self.options.len(); if n > 0 { self.state.select(Some((self.state.selected().unwrap_or(0) + 1) % n)); } }
    fn prev(&mut self) { let n = self.options.len(); if n > 0 { self.state.select(Some((self.state.selected().unwrap_or(0) + n - 1) % n)); } }
}

struct App {
    screen: Screen,
    focus: usize, // 0 name, 1 hostname, 2.. choosers
    name: String,
    hostname: String,
    choosers: Vec<Chooser>, // zone, tier, exposure, database, objects, container
    origins: String,
    error: Option<String>,
    outcome: Option<Outcome>,
    status: Option<AppStatus>,
    last_poll: Instant,
}

impl App {
    fn new(c: &Catalog) -> Self {
        let enum_of = |k: &str| c.spec_schema["properties"][k]["enum"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect::<Vec<_>>()).unwrap_or_default();
        Self {
            screen: Screen::Choose, focus: 0, name: String::new(), hostname: String::new(),
            choosers: vec![
                Chooser::new("zone", c.zones.clone()),
                Chooser::new("tier", if c.tiers.is_empty() { enum_of("tier") } else { c.tiers.clone() }),
                Chooser::new("exposure", enum_of("exposure")),
                Chooser::new("database", enum_of("database")),
                Chooser::new("objects (R2)", vec!["no".into(), "yes".into()]),
                Chooser::new("container", vec!["no".into(), "yes".into()]),
            ],
            origins: String::new(), error: None, outcome: None, status: None, last_poll: Instant::now() - Duration::from_secs(10),
        }
    }
    fn exposure(&self) -> String { self.choosers[2].value() }
    fn spec(&self) -> Value {
        let mut s = json!({
            "zone": self.choosers[0].value(), "hostname": self.hostname, "tier": self.choosers[1].value(),
            "exposure": self.exposure(), "database": self.choosers[3].value(),
            "objects": self.choosers[4].value() == "yes", "container": self.choosers[5].value() == "yes",
        });
        if self.exposure() == "hybrid" {
            s["origins"] = json!(self.origins.split(',').map(|o| o.trim().to_string()).filter(|o| !o.is_empty()).collect::<Vec<_>>());
        }
        s
    }
    fn request(&self, ns: &str) -> AppRequest { AppRequest { namespace: ns.into(), name: self.name.clone(), spec: self.spec() } }
}

pub async fn run(args: &Args, catalog: Catalog, api: Client) -> anyhow::Result<Option<Outcome>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let result = event_loop(&mut terminal, args, catalog, api).await;
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    result
}

async fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, args: &Args, catalog: Catalog, api: Client) -> anyhow::Result<Option<Outcome>> {
    let mut app = App::new(&catalog);
    loop {
        if app.screen == Screen::Watch && app.last_poll.elapsed() > Duration::from_secs(3) {
            if let Some(Outcome::Receipt(r)) = &app.outcome {
                app.status = api.status(&r.namespace, &r.name, &app.spec()).await.ok();
            }
            app.last_poll = Instant::now();
        }
        terminal.draw(|f| draw(f, &mut app, args))?;
        if !event::poll(Duration::from_millis(200))? { continue; }
        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press { continue; }
        match (app.screen, key.code) {
            (_, KeyCode::Esc) => return Ok(app.outcome),
            (Screen::Choose, KeyCode::Tab) => app.focus = (app.focus + 1) % (app.choosers.len() + 2),
            (Screen::Choose, KeyCode::Char(c)) if app.focus < 2 && (c.is_ascii_alphanumeric() || c == '-') => { if app.focus == 0 { app.name.push(c.to_ascii_lowercase()) } else { app.hostname.push(c.to_ascii_lowercase()) } }
            (Screen::Choose, KeyCode::Backspace) if app.focus < 2 => { if app.focus == 0 { app.name.pop(); } else { app.hostname.pop(); } }
            (Screen::Choose, KeyCode::Down | KeyCode::Char('j')) if app.focus >= 2 => app.choosers[app.focus - 2].next(),
            (Screen::Choose, KeyCode::Up | KeyCode::Char('k')) if app.focus >= 2 => app.choosers[app.focus - 2].prev(),
            (Screen::Choose, KeyCode::Enter) => {
                if app.name.is_empty() || app.hostname.is_empty() { app.error = Some("name and hostname are required".into()); }
                else { app.error = None; app.screen = if app.exposure() == "hybrid" { Screen::Origins } else { Screen::Review }; }
            }
            (Screen::Origins, KeyCode::Char(c)) if !c.is_control() => app.origins.push(c),
            (Screen::Origins, KeyCode::Backspace) => { app.origins.pop(); }
            (Screen::Origins, KeyCode::Left) => app.screen = Screen::Choose,
            (Screen::Origins, KeyCode::Enter) => { if app.origins.trim().is_empty() { app.error = Some("hybrid needs at least one origin".into()); } else { app.error = None; app.screen = Screen::Review; } }
            (Screen::Review, KeyCode::Left) => app.screen = if app.exposure() == "hybrid" { Screen::Origins } else { Screen::Choose },
            (Screen::Review, KeyCode::Enter) => match api.submit(&app.request(&args.namespace), &catalog.spec_schema).await {
                Ok(o) => { app.screen = if matches!(o, Outcome::File { .. }) { Screen::Done } else { Screen::Watch }; app.outcome = Some(o); app.error = None; }
                Err(e) => app.error = Some(e.to_string()),
            },
            _ => {}
        }
    }
}

fn frame(f: &mut Frame, title: &str) -> Rect {
    let area = f.area();
    let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(GOLD))
        .title(Span::styled(format!(" EDGE FACTORY  ·  {title} "), Style::default().fg(GOLD).add_modifier(Modifier::BOLD)));
    let inner = block.inner(area);
    f.render_widget(block, area);
    inner
}
fn footer(f: &mut Frame, area: Rect, hint: &str, error: &Option<String>) {
    let text = match error { Some(e) => Line::from(Span::styled(format!("  {e}"), Style::default().fg(RED))), None => Line::from(Span::styled(format!("  {hint}"), Style::default().fg(GREY))) };
    f.render_widget(Paragraph::new(text), area);
}
fn field(f: &mut Frame, area: Rect, title: &str, value: &str, focused: bool) {
    let style = Style::default().fg(if focused { RED } else { GREY });
    f.render_widget(Paragraph::new(value).block(Block::default().borders(Borders::ALL).border_style(style).title(format!(" {title} "))), area);
}

fn draw(f: &mut Frame, app: &mut App, args: &Args) {
    match app.screen {
        Screen::Choose => {
            let inner = frame(f, &format!("request an AppStack ({:?} mode)", args.mode));
            let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(1)]).split(inner);
            let top = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);
            field(f, top[0], "name (app)", &app.name, app.focus == 0);
            field(f, top[1], "hostname (subdomain)", &app.hostname, app.focus == 1);
            let cols = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(22), Constraint::Percentage(14), Constraint::Percentage(16), Constraint::Percentage(16), Constraint::Percentage(16), Constraint::Percentage(16)]).split(rows[1]);
            for (i, ch) in app.choosers.iter_mut().enumerate() {
                let focused = app.focus == i + 2;
                let items: Vec<ListItem> = ch.options.iter().map(|o| ListItem::new(o.as_str())).collect();
                let list = List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(if focused { RED } else { GREY })).title(format!(" {} ", ch.label)))
                    .highlight_style(Style::default().fg(Color::Black).bg(GOLD).add_modifier(Modifier::BOLD)).highlight_symbol("▸ ");
                f.render_stateful_widget(list, cols[i], &mut ch.state);
            }
            footer(f, rows[2], "Tab: next field   ↑/↓: choose   Enter: continue   Esc: quit", &app.error);
        }
        Screen::Origins => {
            let inner = frame(f, "external origins (hybrid): comma-separated hosts");
            let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(1)]).split(inner);
            field(f, rows[0], "origins", &app.origins, true);
            footer(f, rows[2], "Enter: review   ←: back   Esc: quit", &app.error);
        }
        Screen::Review => {
            let inner = frame(f, "review - this is exactly what will be submitted");
            let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(1), Constraint::Length(1)]).split(inner);
            let req = app.request(&args.namespace);
            let body = match args.mode { crate::Mode::Tofu if !args.demo => serde_json::to_string_pretty(&edgefactory_core::xr::request_file(&req)), _ => serde_json::to_string_pretty(&json!({"apiVersion": format!("{}/{}", edgefactory_core::GROUP, edgefactory_core::VERSION), "kind": edgefactory_core::KIND, "metadata": {"name": req.name, "namespace": req.namespace}, "spec": req.spec})) }.unwrap_or_default();
            f.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }), rows[0]);
            footer(f, rows[1], "Enter: submit   ←: back   Esc: quit", &app.error);
        }
        Screen::Done => {
            let inner = frame(f, "request written");
            let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(1), Constraint::Length(1)]).split(inner);
            if let Some(Outcome::File { written, next }) = &app.outcome {
                f.render_widget(Paragraph::new(vec![
                    Line::from(vec![Span::styled("file   ", Style::default().fg(GREY)), Span::styled(written.display().to_string(), Style::default().fg(GOLD))]),
                    Line::from(""), Line::from(Span::raw(next.clone())),
                    Line::from(""), Line::from(Span::styled("The platform's hard opinions (WAF, rate limits, replication, observability) are applied by the profile layer; you never see a Cloudflare attribute.", Style::default().fg(GREY))),
                ]).wrap(Wrap { trim: false }), rows[0]);
            }
            footer(f, rows[1], "Esc: quit", &app.error);
        }
        Screen::Watch => {
            let inner = frame(f, "converging - Crossplane is reconciling your stack");
            let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(5), Constraint::Min(1), Constraint::Length(1)]).split(inner);
            if let Some(Outcome::Receipt(r)) = &app.outcome {
                f.render_widget(Paragraph::new(vec![
                    Line::from(vec![Span::styled("freight   ", Style::default().fg(GREY)), Span::styled(r.freight_reference.clone(), Style::default().fg(GOLD))]),
                    Line::from(vec![Span::styled("digest    ", Style::default().fg(GREY)), Span::raw(r.freight_digest.clone())]),
                    Line::from(vec![Span::styled("requested ", Style::default().fg(GREY)), Span::raw(format!("{} at {}", r.requested_by, r.applied_at))]),
                ]), rows[0]);
            }
            let body: Vec<Line> = match &app.status {
                None => vec![Line::from(Span::styled("waiting for status…", Style::default().fg(GREY)))],
                Some(s) => {
                    let mut v = vec![Line::from(vec![Span::styled(if s.ready { "READY" } else { "NOT READY" }, Style::default().fg(if s.ready { GOLD } else { RED }).add_modifier(Modifier::BOLD)), Span::styled(format!("   synced={}   {}", s.synced, s.message), Style::default().fg(GREY))])];
                    let row = |k: &str, v: &Option<String>| Line::from(format!("  {:<18} {}", k, v.clone().unwrap_or_else(|| "…".into())));
                    v.push(row("hostname", &s.hostname)); v.push(row("worker", &s.worker)); v.push(row("d1", &s.d1_database_id)); v.push(row("r2", &s.r2_bucket)); v.push(row("hyperdrive", &s.hyperdrive_id)); v.push(row("access", &s.access_application_id)); v.push(row("load balancer", &s.load_balancer_id));
                    if let Some(b) = &s.wrangler_bindings { v.push(Line::from("")); v.push(Line::from(Span::styled("wrangler bindings (paste into wrangler.jsonc):", Style::default().fg(GOLD)))); for l in serde_json::to_string_pretty(b).unwrap_or_default().lines() { v.push(Line::from(format!("  {l}"))); } }
                    v
                }
            };
            f.render_widget(Paragraph::new(body), rows[1]);
            footer(f, rows[2], "polling every 3 s   Esc: quit (the stack keeps converging)", &app.error);
        }
    }
}
