use std::io::{self, Stdout};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

pub fn run(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    out.execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        let Event::Key(k) = event::read()? else {
            continue;
        };
        if k.kind != KeyEventKind::Press {
            continue;
        }

        match (k.code, k.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
            (KeyCode::Esc, _) => {
                if app.query.is_empty() {
                    break;
                }
                app.query.clear();
                app.selected = 0;
            }
            (KeyCode::Up, _) => app.move_selection(-1),
            (KeyCode::Down, _) => app.move_selection(1),
            (KeyCode::Tab, _) => app.cycle_category(true),
            (KeyCode::BackTab, _) => app.cycle_category(false),
            (KeyCode::Backspace, _) => app.backspace(),
            (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => app.apply_char(c),
            _ => {}
        }
    }
    Ok(())
}

/// Pad or truncate (with an ellipsis) a string to exactly `w` display columns.
fn fit(s: &str, w: usize) -> String {
    let len = s.chars().count();
    if len <= w {
        format!("{s:<w$}")
    } else if w == 0 {
        String::new()
    } else {
        let head: String = s.chars().take(w - 1).collect();
        format!("{head}…")
    }
}

fn category_tabs(app: &App) -> String {
    let active = app.filter.map(|c| c.key()).unwrap_or("all");
    let mut s = String::new();
    for name in ["all", "http", "exit", "curl", "git"] {
        if name == active {
            s.push_str(&format!("[{name}] "));
        } else {
            s.push_str(&format!(" {name}  "));
        }
    }
    s
}

fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header (search + tabs)
            Constraint::Min(5),    // list
            Constraint::Length(7), // detail
            Constraint::Length(1), // footer
        ])
        .split(frame.area());

    // Header: search line + category tabs.
    let header = Paragraph::new(Line::from(vec![
        Span::styled("codez", Style::default().fg(theme::LAVENDER).bold()),
        Span::raw("  search: "),
        Span::styled(app.query.clone(), Style::default().fg(theme::TEXT)),
        Span::styled("_", Style::default().fg(theme::OVERLAY)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::SURFACE))
            .title(Span::styled(
                category_tabs(app),
                Style::default().fg(theme::MAUVE),
            )),
    );
    frame.render_widget(header, chunks[0]);

    // List of filtered entries. Column widths size to the visible content
    // (codes range from 3-digit HTTP to long git slugs), capped and ellipsized.
    let hits = app.filtered();
    let code_w = hits
        .iter()
        .map(|e| e.code.chars().count())
        .max()
        .unwrap_or(3)
        .clamp(3, 18);
    let name_w = hits
        .iter()
        .map(|e| e.name.chars().count())
        .max()
        .unwrap_or(8)
        .clamp(8, 28);
    let items: Vec<ListItem> = hits
        .iter()
        .map(|e| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    fit(&e.code, code_w),
                    Style::default().fg(theme::group_color(&e.group)),
                ),
                Span::raw("  "),
                Span::styled(fit(&e.name, name_w), Style::default().fg(theme::SAPPHIRE)),
                Span::raw("  "),
                Span::styled(e.summary.clone(), Style::default().fg(theme::OVERLAY)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    if !hits.is_empty() {
        state.select(Some(app.selected.min(hits.len() - 1)));
    }
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::SURFACE)),
        )
        .highlight_style(Style::default().fg(theme::TEXT).bold())
        .highlight_symbol("▸ ");
    frame.render_stateful_widget(list, chunks[1], &mut state);

    // Detail pane for the selected entry.
    let detail = match app.selected_entry() {
        Some(e) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} {}", e.code, e.name),
                        Style::default().fg(theme::group_color(&e.group)).bold(),
                    ),
                    Span::styled(format!("  ·  {}", e.group), Style::default().fg(theme::OVERLAY)),
                ]),
                Line::from(Span::styled(e.summary.clone(), Style::default().fg(theme::TEXT))),
            ];
            if let Some(d) = &e.detail {
                lines.push(Line::from(Span::styled(d.clone(), Style::default().fg(theme::OVERLAY))));
            }
            if let Some(f) = &e.fix {
                lines.push(Line::from(vec![
                    Span::styled("fix: ", Style::default().fg(theme::GREEN)),
                    Span::styled(f.clone(), Style::default().fg(theme::TEXT)),
                ]));
            }
            if let Some(r) = &e.reference {
                lines.push(Line::from(Span::styled(
                    format!("ref: {r}"),
                    Style::default().fg(theme::OVERLAY),
                )));
            }
            Paragraph::new(lines).wrap(Wrap { trim: true })
        }
        None => Paragraph::new(Span::styled(
            "no matches",
            Style::default().fg(theme::OVERLAY),
        )),
    }
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::SURFACE)),
    );
    frame.render_widget(detail, chunks[2]);

    // Footer key hints.
    let footer = Paragraph::new(Span::styled(
        "  ↑↓ move   ⇥ category   type to search   esc clear/quit   ^C quit",
        Style::default().fg(theme::OVERLAY),
    ));
    frame.render_widget(footer, chunks[3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::load_all;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn git_slugs_render_without_clipping_the_code() {
        use crate::model::Category;
        let mut app = App::new(load_all());
        app.filter = Some(Category::Git);
        let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("nothing-to-commit"));
        assert!(text.contains("non-fast-forward"));
    }

    #[test]
    fn draw_renders_header_and_a_row() {
        let app = App::new(load_all());
        let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("codez"));
        assert!(text.contains("200"));
    }
}
