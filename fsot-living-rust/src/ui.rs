//! Interactive terminal UI — type to query, watch life run

use crate::habitat::Habitat;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

pub struct App {
    pub habitat: Habitat,
    pub input: String,
    pub focus_input: bool,
    pub should_quit: bool,
    pub status_line: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            habitat: Habitat::new(42),
            input: String::new(),
            focus_input: true,
            should_quit: false,
            status_line: "Enter=ask · Esc=quit · Space=pause · Tab=focus".into(),
        }
    }

    pub fn run(mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
        let tick_rate = Duration::from_millis(80);
        let mut last = Instant::now();

        loop {
            terminal.draw(|f| self.draw(f))?;

            let timeout = tick_rate
                .checked_sub(last.elapsed())
                .unwrap_or(Duration::from_millis(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.on_key(key.code, key.modifiers);
                    }
                }
            }

            if last.elapsed() >= tick_rate {
                self.habitat.step();
                // External Python/agent queries — same path as Enter in this TUI
                self.habitat.poll_external_queries();
                // Live review file for external tools / agents (every 5 ticks)
                if self.habitat.tick % 5 == 0 {
                    let _ = crate::export::write_live(&self.habitat);
                }
                last = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn on_key(&mut self, code: KeyCode, mods: KeyModifiers) {
        if mods.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        match code {
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab => {
                self.focus_input = !self.focus_input;
            }
            KeyCode::Char(' ') if !self.focus_input => {
                self.habitat.paused = !self.habitat.paused;
                self.status_line = if self.habitat.paused {
                    "PAUSED".into()
                } else {
                    "RUNNING".into()
                };
            }
            KeyCode::Enter if self.focus_input => {
                let q = self.input.trim().to_string();
                if !q.is_empty() {
                    self.habitat.ask(&q);
                    self.input.clear();
                    self.status_line = format!("kind={}", self.habitat.last_query_kind);
                }
            }
            KeyCode::Backspace if self.focus_input => {
                self.input.pop();
            }
            KeyCode::Char(c) if self.focus_input => {
                if !c.is_control() {
                    self.input.push(c);
                }
            }
            KeyCode::Char('q') if !self.focus_input => self.should_quit = true,
            KeyCode::Char('p') if !self.focus_input => {
                self.habitat.paused = !self.habitat.paused;
            }
            _ => {}
        }
    }

    fn draw(&self, f: &mut ratatui::Frame) {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(8),
                Constraint::Length(3),
            ])
            .split(f.area());

        self.draw_header(f, root[0]);
        self.draw_main(f, root[1]);
        self.draw_answer(f, root[2]);
        self.draw_input(f, root[3]);
    }

    fn draw_header(&self, f: &mut ratatui::Frame, area: Rect) {
        let h = self.habitat.hero();
        let pause = if self.habitat.paused { " PAUSED" } else { "" };
        let title = format!(
            " FSOT Living{pause} t={} gen={} | {} · {} | fed={} exams={} ",
            self.habitat.tick,
            self.habitat.generation,
            self.habitat.current_phase,
            self.habitat.current_detail.chars().take(40).collect::<String>(),
            self.habitat.fed,
            self.habitat.exams
        );
        let regime_color = match h.regime.as_str() {
            "emergence" => Color::Green,
            "damping" => Color::Red,
            _ => Color::Cyan,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(regime_color));
        let text = Line::from(vec![
            Span::styled(
                format!(" {} ", h.id),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}  S={:.4}  ", h.regime.to_uppercase(), h.s),
                Style::default().fg(regime_color),
            ),
            Span::raw(format!(
                "acc={:.0}%  codons={}/64  fit={:.2}  P={:.2}  c={:.2}  body={}",
                h.cognitive_accuracy * 100.0,
                h.codon_table.len(),
                h.fitness_score(),
                h.pressure,
                h.coherence,
                if self.habitat.body.online {
                    format!(
                        "{} c={:.2} m={}",
                        if self.habitat.body.transport.is_empty() {
                            "ON"
                        } else {
                            self.habitat.body.transport.as_str()
                        },
                        self.habitat.body.couple_pressure,
                        self.habitat.body.motors_sent
                    )
                } else {
                    "off".into()
                }
            )),
        ]);
        f.render_widget(Paragraph::new(text).block(block), area);
    }

    fn draw_main(&self, f: &mut ratatui::Frame, area: Rect) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);

        let h = self.habitat.hero();
        let vitals = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(3),
            ])
            .split(cols[0]);

        let s_pct = (((h.s + 1.0) / 2.0) * 100.0).clamp(0.0, 100.0) as u16;
        f.render_widget(
            Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Scalar S "))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(s_pct)
                .label(format!("{:.4}", h.s)),
            vitals[0],
        );
        f.render_widget(
            Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Coherence "))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent((h.coherence * 100.0) as u16)
                .label(format!("{:.3}", h.coherence)),
            vitals[1],
        );
        f.render_widget(
            Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Pressure "))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent((h.pressure.min(1.0) * 100.0) as u16)
                .label(format!("{:.3}", h.pressure)),
            vitals[2],
        );
        f.render_widget(
            Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Cognitive "))
                .gauge_style(Style::default().fg(Color::Magenta))
                .percent((h.cognitive_accuracy * 100.0) as u16)
                .label(format!("{:.1}%", h.cognitive_accuracy * 100.0)),
            vitals[3],
        );
        let info = format!(
            "D_eff={:.2} dpsi={:.3}\nAA: {}\nintel={:.3}\ncode={:.0}% skill={:.2}\nvision={:.0}% skill={:.2}\ntools={} ({})",
            h.d_eff,
            h.delta_psi,
            h.guidance.aa_chain,
            h.intelligence,
            h.code_accuracy * 100.0,
            h.code_skill,
            h.vision_accuracy * 100.0,
            h.vision_skill,
            h.tools_owned.len(),
            h.tools_owned.join(",")
        );
        f.render_widget(
            Paragraph::new(info).block(Block::default().borders(Borders::ALL).title(" Detail ")),
            vitals[4],
        );

        f.render_widget(
            List::new(
                self.habitat
                    .pack
                    .iter()
                    .enumerate()
                    .map(|(i, o)| {
                        let mark = if i == self.habitat.hero_idx { "*" } else { " " };
                        let style = if i == self.habitat.hero_idx {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        };
                        ListItem::new(format!(
                            "{mark} {:14} S={:.3} a={:3.0}% c={:2} f={:.2}",
                            o.id,
                            o.s,
                            o.cognitive_accuracy * 100.0,
                            o.codon_table.len(),
                            o.fitness_score()
                        ))
                        .style(style)
                    })
                    .collect::<Vec<_>>(),
            )
            .block(Block::default().borders(Borders::ALL).title(" Population ")),
            cols[1],
        );

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(cols[2]);

        let ev: Vec<ListItem> = self
            .habitat
            .events
            .iter()
            .take(20)
            .map(|e| ListItem::new(format!("[t{}] {}", e.tick, e.msg)))
            .collect();
        f.render_widget(
            List::new(ev).block(Block::default().borders(Borders::ALL).title(" Habitat log ")),
            right[0],
        );

        let lin: Vec<ListItem> = self
            .habitat
            .lineage
            .iter()
            .rev()
            .take(12)
            .map(|l| {
                ListItem::new(format!(
                    "g{} {} S={:.3} a={:.0}% f={:.2} cod={}",
                    l.gen,
                    l.champion,
                    l.s,
                    l.acc * 100.0,
                    l.fit,
                    l.codons
                ))
            })
            .collect();
        f.render_widget(
            List::new(lin).block(Block::default().borders(Borders::ALL).title(" Lineage ")),
            right[1],
        );
    }

    fn draw_answer(&self, f: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Answer [{}] ", self.habitat.last_query_kind))
            .border_style(Style::default().fg(Color::Rgb(94, 234, 212)));
        f.render_widget(
            Paragraph::new(self.habitat.last_answer.as_str())
                .block(block)
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn draw_input(&self, f: &mut ratatui::Frame, area: Rect) {
        let style = if self.focus_input {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Query — type here, press Enter ")
            .border_style(style);
        let cursor = if self.focus_input { "_" } else { "" };
        f.render_widget(
            Paragraph::new(format!("> {}{}", self.input, cursor)).block(block),
            area,
        );
    }
}

pub fn run_tui() -> anyhow::Result<()> {
    // Explicit setup so errors are clear (ratatui::init hides some Windows issues)
    enable_raw_mode().map_err(|e| anyhow::anyhow!("enable_raw_mode failed: {e}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| anyhow::anyhow!("EnterAlternateScreen failed: {e}"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| anyhow::anyhow!("Terminal::new failed: {e}"))?;

    let app_result = App::new().run(&mut terminal);

    // Always restore terminal
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    app_result
}
