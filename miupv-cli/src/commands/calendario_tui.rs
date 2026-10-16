//! Interactive Terminal User Interface (TUI) for UPV Academic Calendar.
//! Implements a Stale-While-Revalidate caching architecture for instant responsive startup.

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use miupv_core::models::CalEvent;
use miupv_core::services::calendar_service::{self, CalendarCache};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::DefaultTerminal;
use std::time::Instant;
use tokio::sync::mpsc;

/// Parsed calendar event with local timezone dates and structured metadata.
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    pub id: String,
    pub date: NaiveDate,
    pub start_time: String,
    pub end_time: String,
    pub acronym: String,
    pub subject_title: String,
    pub teacher: Option<String>,
    pub teaching_type: Option<String>,
    pub pod_group: Option<String>,
    pub classroom: String,
    pub category: String,
    pub raw_desc: String,
    pub is_all_day: bool,
    pub color: Color,
}

impl ParsedEvent {
    pub fn from_cal_event(ev: &CalEvent) -> Self {
        let (date, start_time, end_time) =
            parse_event_times(&ev.fecha_ini, &ev.fecha_fin, ev.todo_dia);
        let (subject_title, teacher, teaching_type, pod_group, clean_desc) =
            parse_html_description(ev.descripcion.as_deref().unwrap_or(""));

        let title = if subject_title.is_empty() {
            ev.titulo.clone()
        } else {
            subject_title
        };

        let cat = ev.tipo.clone().unwrap_or_else(|| "Clases".to_string());
        let color = match cat.to_uppercase().as_str() {
            s if s.contains("EXAM") || s.contains("EVAL") => Color::Red,
            s if s.contains("FEST") || s.contains("VACAC") => Color::Magenta,
            s if s.contains("PRAC") || s.contains("LAB") => Color::Green,
            _ => Color::Cyan,
        };

        Self {
            id: ev.id.clone(),
            date,
            start_time,
            end_time,
            acronym: ev.titulo.clone(),
            subject_title: title,
            teacher,
            teaching_type,
            pod_group,
            classroom: ev
                .lugar
                .clone()
                .unwrap_or_else(|| "Sin aula asignada".to_string()),
            category: cat,
            raw_desc: clean_desc,
            is_all_day: ev.todo_dia,
            color,
        }
    }
}

/// Day container grouping events for that calendar day.
#[derive(Debug, Clone)]
pub struct DayBucket {
    pub date: NaiveDate,
    pub events: Vec<ParsedEvent>,
}

/// TUI Application state.
pub struct CalendarApp {
    pub days: Vec<DayBucket>,
    pub selected_day_idx: usize,
    pub selected_event_idx: usize,
    pub is_syncing: bool,
    pub last_synced: Option<DateTime<Utc>>,
    pub sync_error: Option<String>,
    pub token: String,
    pub lang: String,
    pub spinner_idx: usize,
    pub last_tick: Instant,
}

impl CalendarApp {
    pub fn new(
        token: String,
        lang: String,
        initial_events: Vec<CalEvent>,
        last_synced: Option<DateTime<Utc>>,
    ) -> Self {
        let mut app = Self {
            days: Vec::new(),
            selected_day_idx: 0,
            selected_event_idx: 0,
            is_syncing: false,
            last_synced,
            sync_error: None,
            token,
            lang,
            spinner_idx: 0,
            last_tick: Instant::now(),
        };

        app.set_events(initial_events);
        app.select_today_or_next();
        app
    }

    pub fn set_events(&mut self, raw_events: Vec<CalEvent>) {
        let mut parsed: Vec<ParsedEvent> =
            raw_events.iter().map(ParsedEvent::from_cal_event).collect();
        parsed.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.start_time.cmp(&b.start_time))
        });

        let today = Local::now().date_naive();
        let min_date = today - Duration::days(14);
        let max_date = today + Duration::days(45);

        // Group by day ensuring continuous or populated date range
        let mut buckets: Vec<DayBucket> = Vec::new();
        let mut current_d = min_date;

        while current_d <= max_date {
            let day_events: Vec<ParsedEvent> = parsed
                .iter()
                .filter(|e| e.date == current_d)
                .cloned()
                .collect();

            // Include day if it has events or if it's within +/- 7 days of today
            let diff = (current_d - today).num_days().abs();
            if !day_events.is_empty() || diff <= 7 {
                buckets.push(DayBucket {
                    date: current_d,
                    events: day_events,
                });
            }
            current_d += Duration::days(1);
        }

        // Also add any events beyond that range
        for p in &parsed {
            if p.date < min_date || p.date > max_date {
                if let Some(b) = buckets.iter_mut().find(|b| b.date == p.date) {
                    if !b.events.iter().any(|e| e.id == p.id) {
                        b.events.push(p.clone());
                    }
                } else {
                    buckets.push(DayBucket {
                        date: p.date,
                        events: vec![p.clone()],
                    });
                }
            }
        }

        buckets.sort_by_key(|b| b.date);

        // Preserve selected date if possible
        let prev_date = self.current_selected_date();
        self.days = buckets;

        if let Some(target) = prev_date {
            if let Some(pos) = self.days.iter().position(|b| b.date == target) {
                self.selected_day_idx = pos;
            } else {
                self.select_today_or_next();
            }
        } else {
            self.select_today_or_next();
        }

        self.clamp_event_selection();
    }

    pub fn current_selected_date(&self) -> Option<NaiveDate> {
        self.days.get(self.selected_day_idx).map(|b| b.date)
    }

    pub fn select_today_or_next(&mut self) {
        let today = Local::now().date_naive();
        if let Some(idx) = self.days.iter().position(|b| b.date == today) {
            self.selected_day_idx = idx;
        } else if let Some(idx) = self.days.iter().position(|b| b.date >= today) {
            self.selected_day_idx = idx;
        } else if !self.days.is_empty() {
            self.selected_day_idx = self.days.len() - 1;
        }
        self.selected_event_idx = 0;
    }

    pub fn prev_day(&mut self) {
        if self.selected_day_idx > 0 {
            self.selected_day_idx -= 1;
            self.selected_event_idx = 0;
        }
    }

    pub fn next_day(&mut self) {
        if !self.days.is_empty() && self.selected_day_idx < self.days.len() - 1 {
            self.selected_day_idx += 1;
            self.selected_event_idx = 0;
        }
    }

    pub fn prev_event(&mut self) {
        if self.selected_event_idx > 0 {
            self.selected_event_idx -= 1;
        }
    }

    pub fn next_event(&mut self) {
        if let Some(bucket) = self.days.get(self.selected_day_idx) {
            if !bucket.events.is_empty() && self.selected_event_idx < bucket.events.len() - 1 {
                self.selected_event_idx += 1;
            }
        }
    }

    fn clamp_event_selection(&mut self) {
        if let Some(bucket) = self.days.get(self.selected_day_idx) {
            if bucket.events.is_empty() {
                self.selected_event_idx = 0;
            } else if self.selected_event_idx >= bucket.events.len() {
                self.selected_event_idx = bucket.events.len() - 1;
            }
        }
    }

    pub fn current_day_bucket(&self) -> Option<&DayBucket> {
        self.days.get(self.selected_day_idx)
    }

    pub fn current_selected_event(&self) -> Option<&ParsedEvent> {
        self.current_day_bucket()
            .and_then(|b| b.events.get(self.selected_event_idx))
    }
}

/// Spawns a background task to fetch live calendar events from UPV servers.
fn trigger_background_sync(
    token: String,
    lang: String,
    tx: mpsc::Sender<Result<Vec<CalEvent>, String>>,
) {
    tokio::spawn(async move {
        let today = Local::now().date_naive();
        let ini = (today - Duration::days(30)).format("%Y-%m-%d").to_string();
        let fin = (today + Duration::days(120)).format("%Y-%m-%d").to_string();

        match calendar_service::get_ical_events(&token, &ini, &fin, &lang).await {
            Ok(events) => {
                let _ = calendar_service::save_cached_calendar(&events);
                let _ = tx.send(Ok(events)).await;
            }
            Err(e) => {
                let _ = tx.send(Err(e.to_string())).await;
            }
        }
    });
}

/// Launches the interactive Ratatui Calendar TUI.
pub async fn run_calendar_tui(token: &str, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Stale-While-Revalidate: Load existing local cache immediately (< 5ms)
    let cached = calendar_service::load_cached_calendar();
    let (initial_events, last_synced) = match cached {
        Some(CalendarCache { updated_at, events }) => (events, Some(updated_at)),
        None => (Vec::new(), None),
    };

    let mut app = CalendarApp::new(
        token.to_string(),
        lang.to_string(),
        initial_events,
        last_synced,
    );

    // 2. Setup background channel and trigger automatic background refresh
    let (tx, mut rx) = mpsc::channel::<Result<Vec<CalEvent>, String>>(4);
    app.is_syncing = true;
    trigger_background_sync(token.to_string(), lang.to_string(), tx.clone());

    // 3. Initialize Ratatui terminal with raw mode and alternate screen
    let mut terminal = ratatui::init();

    // 4. Main Event Loop
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let result = run_app_loop(&mut terminal, &mut app, &mut rx, tx.clone(), &spinner_chars).await;

    // 5. Always restore terminal safely
    ratatui::restore();

    result
}

async fn run_app_loop(
    terminal: &mut DefaultTerminal,
    app: &mut CalendarApp,
    rx: &mut mpsc::Receiver<Result<Vec<CalEvent>, String>>,
    tx: mpsc::Sender<Result<Vec<CalEvent>, String>>,
    spinner_chars: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Draw frame
        terminal.draw(|frame| draw_ui(frame, app, spinner_chars))?;

        // Handle network messages from background worker
        if let Ok(msg) = rx.try_recv() {
            app.is_syncing = false;
            match msg {
                Ok(new_events) => {
                    app.set_events(new_events);
                    app.last_synced = Some(Utc::now());
                    app.sync_error = None;
                }
                Err(err) => {
                    app.sync_error = Some(err);
                }
            }
        }

        // Handle spinner tick
        if app.last_tick.elapsed().as_millis() > 80 {
            app.spinner_idx = (app.spinner_idx + 1) % spinner_chars.len();
            app.last_tick = Instant::now();
        }

        // Handle user input events with a short timeout
        if event::poll(std::time::Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('[') => {
                            app.prev_day();
                        }
                        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(']') => {
                            app.next_day();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.prev_event();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.next_event();
                        }
                        KeyCode::Char('t') => {
                            app.select_today_or_next();
                        }
                        KeyCode::Char('r') if !app.is_syncing => {
                            app.is_syncing = true;
                            app.sync_error = None;
                            trigger_background_sync(
                                app.token.clone(),
                                app.lang.clone(),
                                tx.clone(),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn draw_ui(frame: &mut ratatui::Frame, app: &CalendarApp, spinner_chars: &[&str]) {
    let size = frame.area();

    // Main layout: Header, Body, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer / Shortcuts
        ])
        .split(size);

    draw_header(frame, chunks[0], app, spinner_chars);
    draw_content(frame, chunks[1], app);
    draw_footer(frame, chunks[2], app);
}

fn draw_header(frame: &mut ratatui::Frame, area: Rect, app: &CalendarApp, spinner_chars: &[&str]) {
    let today = Local::now().date_naive();
    let today_str = format_spanish_date(today);

    let sync_status = if app.is_syncing {
        let spin = spinner_chars[app.spinner_idx];
        Span::styled(
            format!(" {spin} Sincronizando con UPV... "),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else if let Some(err) = &app.sync_error {
        Span::styled(
            format!(" ✘ Error al sincronizar: {err} "),
            Style::default().fg(Color::Red),
        )
    } else if let Some(last) = app.last_synced {
        let mins = (Utc::now() - last).num_minutes();
        let time_label = if mins == 0 {
            "hace un momento".to_string()
        } else if mins == 1 {
            "hace 1 min".to_string()
        } else {
            format!("hace {mins} min")
        };
        Span::styled(
            format!(" ● Sincronizado ({time_label}) "),
            Style::default().fg(Color::Green),
        )
    } else {
        Span::styled(" ● Datos locales ", Style::default().fg(Color::Cyan))
    };

    let title_line = Line::from(vec![
        Span::styled(
            " miUPV ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "HORARIO Y CALENDARIO ACADÉMICO",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  •  "),
        Span::styled(today_str, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        sync_status,
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(title_line).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_content(frame: &mut ratatui::Frame, area: Rect, app: &CalendarApp) {
    let main_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26), // Left list: Days
            Constraint::Min(40),    // Right pane: Events & Detail
        ])
        .split(area);

    draw_days_list(frame, main_cols[0], app);
    draw_events_and_details(frame, main_cols[1], app);
}

fn draw_days_list(frame: &mut ratatui::Frame, area: Rect, app: &CalendarApp) {
    let today = Local::now().date_naive();

    let items: Vec<ListItem> = app
        .days
        .iter()
        .enumerate()
        .map(|(idx, bucket)| {
            let is_selected = idx == app.selected_day_idx;
            let is_today = bucket.date == today;
            let count = bucket.events.len();

            let weekday = match bucket.date.weekday() {
                chrono::Weekday::Mon => "Lun",
                chrono::Weekday::Tue => "Mar",
                chrono::Weekday::Wed => "Mié",
                chrono::Weekday::Thu => "Jue",
                chrono::Weekday::Fri => "Vie",
                chrono::Weekday::Sat => "Sáb",
                chrono::Weekday::Sun => "Dom",
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let date_str = format!(
                "{:02}/{:02} {}",
                bucket.date.day(),
                bucket.date.month(),
                weekday
            );

            let (label, badge_style) = if is_today {
                (
                    format!("{prefix}{date_str} (Hoy)"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else if count > 0 {
                (
                    format!("{prefix}{date_str} [{count}]"),
                    Style::default().fg(Color::White),
                )
            } else {
                (
                    format!("{prefix}{date_str}"),
                    Style::default().fg(Color::DarkGray),
                )
            };

            let item_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                badge_style
            };

            ListItem::new(Span::styled(label, item_style))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Agenda de Días ")
        .border_style(Style::default().fg(Color::DarkGray));

    let list = List::new(items).block(block);
    let mut state = ListState::default();
    state.select(Some(app.selected_day_idx));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_events_and_details(frame: &mut ratatui::Frame, area: Rect, app: &CalendarApp) {
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // Events of selected day
            Constraint::Percentage(45), // Selected event detail
        ])
        .split(area);

    draw_day_events(frame, right_rows[0], app);
    draw_event_detail(frame, right_rows[1], app);
}

fn draw_day_events(frame: &mut ratatui::Frame, area: Rect, app: &CalendarApp) {
    let current_bucket = app.current_day_bucket();
    let today = Local::now().date_naive();

    let header_title = if let Some(b) = current_bucket {
        let tag = if b.date == today { " (HOY)" } else { "" };
        format!(
            " Clases y Actividades: {}{} ",
            format_spanish_date(b.date),
            tag
        )
    } else {
        " Clases y Actividades ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(header_title)
        .border_style(Style::default().fg(Color::Cyan));

    if let Some(b) = current_bucket {
        if b.events.is_empty() {
            let empty_text = Paragraph::new(vec![
                Line::raw(""),
                Line::from(Span::styled(
                    "   No hay clases ni eventos académicos programados para este día.",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "   Usa [←] o [→] para desplazarte por otros días de la semana.",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(block);
            frame.render_widget(empty_text, area);
            return;
        }

        let items: Vec<ListItem> = b
            .events
            .iter()
            .enumerate()
            .map(|(idx, ev)| {
                let is_selected = idx == app.selected_event_idx;
                let marker = if is_selected { "● " } else { "  " };

                let time_span = Span::styled(
                    format!("{}{} - {} ", marker, ev.start_time, ev.end_time),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

                let acronym_span = Span::styled(
                    format!(" [{}] ", ev.acronym),
                    Style::default().fg(ev.color).add_modifier(Modifier::BOLD),
                );

                let title_span = Span::styled(
                    &ev.subject_title,
                    if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                );

                let mut second_line_spans = vec![
                    Span::raw("     📍 "),
                    Span::styled(&ev.classroom, Style::default().fg(Color::Gray)),
                ];

                if let Some(doc) = &ev.teaching_type {
                    second_line_spans.push(Span::raw("  "));
                    second_line_spans.push(Span::styled(
                        format!("[{doc}]"),
                        Style::default().fg(Color::Green),
                    ));
                }

                let lines = vec![
                    Line::from(vec![time_span, acronym_span, title_span]),
                    Line::from(second_line_spans),
                ];

                let item_style = if is_selected {
                    Style::default().bg(Color::Rgb(20, 35, 55))
                } else {
                    Style::default()
                };

                ListItem::new(lines).style(item_style)
            })
            .collect();

        let list = List::new(items).block(block);
        let mut state = ListState::default();
        state.select(Some(app.selected_event_idx));
        frame.render_stateful_widget(list, area, &mut state);
    } else {
        frame.render_widget(Paragraph::new("Sin datos").block(block), area);
    }
}

fn draw_event_detail(frame: &mut ratatui::Frame, area: Rect, app: &CalendarApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Detalle de la Asignatura / Evento ")
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(ev) = app.current_selected_event() {
        let mut text_lines = Vec::new();

        text_lines.push(Line::from(vec![
            Span::styled(
                "Asignatura: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &ev.subject_title,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        text_lines.push(Line::from(vec![
            Span::styled(
                "Categoría:  ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}]", ev.category),
                Style::default().fg(ev.color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if ev.is_all_day {
                    " (Día completo)"
                } else {
                    ""
                },
                Style::default().fg(Color::Yellow),
            ),
        ]));

        text_lines.push(Line::from(vec![
            Span::styled(
                "Horario:    ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{} - {} ({})",
                    ev.start_time,
                    ev.end_time,
                    format_spanish_date(ev.date)
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]));

        text_lines.push(Line::from(vec![
            Span::styled(
                "Ubicación:  ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&ev.classroom, Style::default().fg(Color::White)),
        ]));

        if let Some(t) = &ev.teacher {
            text_lines.push(Line::from(vec![
                Span::styled(
                    "Profesor:   ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(t, Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(doc) = &ev.teaching_type {
            text_lines.push(Line::from(vec![
                Span::styled(
                    "Docencia:   ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(doc, Style::default().fg(Color::Magenta)),
            ]));
        }

        if let Some(pod) = &ev.pod_group {
            text_lines.push(Line::from(vec![
                Span::styled(
                    "Grupo POD:  ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(pod, Style::default().fg(Color::Gray)),
            ]));
        }

        if !ev.raw_desc.is_empty() && ev.teacher.is_none() && ev.teaching_type.is_none() {
            text_lines.push(Line::raw(""));
            text_lines.push(Line::from(Span::styled(
                &ev.raw_desc,
                Style::default().fg(Color::Gray),
            )));
        }

        let detail_p = Paragraph::new(text_lines)
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(detail_p, area);
    } else {
        let no_sel =
            Paragraph::new("Selecciona un evento de la lista para ver su información detallada.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
        frame.render_widget(no_sel, area);
    }
}

fn draw_footer(frame: &mut ratatui::Frame, area: Rect, _app: &CalendarApp) {
    let shortcuts = Line::from(vec![
        Span::styled(
            " [←/→ o h/l] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Día  "),
        Span::styled(
            " [↑/↓ o k/j] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Evento  "),
        Span::styled(
            " [t] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Hoy  "),
        Span::styled(
            " [r] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Recargar UPV  "),
        Span::styled(
            " [q / Esc] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Salir"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let footer = Paragraph::new(shortcuts).block(block);
    frame.render_widget(footer, area);
}

/// Formats a NaiveDate into full Spanish text (e.g. "Viernes, 16 de Octubre de 2026").
fn format_spanish_date(date: NaiveDate) -> String {
    let weekday = match date.weekday() {
        chrono::Weekday::Mon => "Lunes",
        chrono::Weekday::Tue => "Martes",
        chrono::Weekday::Wed => "Miércoles",
        chrono::Weekday::Thu => "Jueves",
        chrono::Weekday::Fri => "Viernes",
        chrono::Weekday::Sat => "Sábado",
        chrono::Weekday::Sun => "Domingo",
    };

    let month = match date.month() {
        1 => "Enero",
        2 => "Febrero",
        3 => "Marzo",
        4 => "Abril",
        5 => "Mayo",
        6 => "Junio",
        7 => "Julio",
        8 => "Agosto",
        9 => "Septiembre",
        10 => "Octubre",
        11 => "Noviembre",
        12 => "Diciembre",
        _ => "",
    };

    format!(
        "{}, {} de {} de {}",
        weekday,
        date.day(),
        month,
        date.year()
    )
}

/// Parses start and end time strings converting UTC 'Z' timestamps to Local Spanish time.
fn parse_event_times(raw_ini: &str, raw_fin: &str, all_day: bool) -> (NaiveDate, String, String) {
    if all_day {
        let date = if let Ok(d) = NaiveDate::parse_from_str(raw_ini.trim(), "%Y-%m-%d") {
            d
        } else if let Ok(dt) = DateTime::parse_from_rfc3339(raw_ini.trim()) {
            dt.with_timezone(&Local).date_naive()
        } else {
            Local::now().date_naive()
        };
        return (date, "Todo el día".to_string(), "".to_string());
    }

    // Try RFC3339 format (standard UPV Oracle format: "2026-10-16T06:00:00.000Z")
    if let (Ok(dt_ini), Ok(dt_fin)) = (
        DateTime::parse_from_rfc3339(raw_ini.trim()),
        DateTime::parse_from_rfc3339(raw_fin.trim()),
    ) {
        let local_ini = dt_ini.with_timezone(&Local);
        let local_fin = dt_fin.with_timezone(&Local);
        return (
            local_ini.date_naive(),
            local_ini.format("%H:%M").to_string(),
            local_fin.format("%H:%M").to_string(),
        );
    }

    // Try Naive formats (e.g. "2026-10-16 08:30:00")
    for fmt in &[
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
    ] {
        if let (Ok(ni), Ok(nf)) = (
            NaiveDateTime::parse_from_str(raw_ini.trim(), fmt),
            NaiveDateTime::parse_from_str(raw_fin.trim(), fmt),
        ) {
            return (
                ni.date(),
                ni.format("%H:%M").to_string(),
                nf.format("%H:%M").to_string(),
            );
        }
    }

    (
        Local::now().date_naive(),
        raw_ini.to_string(),
        raw_fin.to_string(),
    )
}

/// Cleans UPV HTML description and extracts structured fields.
fn parse_html_description(
    html: &str,
) -> (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
) {
    if html.is_empty() {
        return (String::new(), None, None, None, String::new());
    }

    let mut subject_name = String::new();
    let mut teacher = None;
    let mut teaching_type = None;
    let mut pod_group = None;

    // Subject title inside <b>...</b>
    if let Some(start) = html.find("<b>") {
        if let Some(end) = html[start + 3..].find("</b>") {
            subject_name = html[start + 3..start + 3 + end].trim().to_string();
        }
    }

    let plain = html
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        .replace("<br />", "\n")
        .replace("<b>", "")
        .replace("</b>", "");

    for line in plain.lines() {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.to_lowercase().find("docente") {
            if let Some(colon) = trimmed[idx..].find(':') {
                let val = trimmed[idx + colon + 1..].trim();
                if !val.is_empty() {
                    teacher = Some(val.to_string());
                }
            }
        } else if let Some(idx) = trimmed.to_lowercase().find("tipo de docencia") {
            if let Some(colon) = trimmed[idx..].find(':') {
                let val = trimmed[idx + colon + 1..].trim();
                if !val.is_empty() {
                    teaching_type = Some(val.to_string());
                }
            }
        } else if let Some(idx) = trimmed.to_lowercase().find("grupo pod") {
            if let Some(colon) = trimmed[idx..].find(':') {
                let val = trimmed[idx + colon + 1..].trim();
                if !val.is_empty() {
                    pod_group = Some(val.to_string());
                }
            }
        }
    }

    (subject_name, teacher, teaching_type, pod_group, plain)
}
