use crate::app::{App, short_container_id};
use crate::backend::ContainerState;
use crate::i18n;
use crate::ui::mouse::ClickAction;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

pub fn render_list(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(app.theme.border));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let area = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(1),
        inner.height,
    );

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        i18n::containers_title(app.containers.containers.len()),
        Style::default()
            .fg(app.theme.fg_primary)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if app.containers_load.loading && app.containers.containers.is_empty() {
        lines.push(Line::from(Span::styled(
            "loading...",
            Style::default().fg(app.theme.fg_secondary),
        )));
    } else if let Some(error) = app.containers_load.error.as_ref() {
        lines.push(Line::from(Span::styled(
            i18n::containers_error(error),
            Style::default().fg(Color::Red),
        )));
    } else if app.containers.containers.is_empty() {
        lines.push(Line::from(Span::styled(
            i18n::containers_empty(),
            Style::default().fg(app.theme.fg_secondary),
        )));
    }

    for (idx, container) in app.containers.containers.iter().enumerate() {
        let row_y = area.y + lines.len() as u16;
        let selected = idx == app.containers.selected_idx;
        let bg = selected.then_some(app.theme.selection_bg);
        let style = apply_bg(Style::default().fg(app.theme.fg_primary), bg);
        let state_style = apply_bg(
            Style::default()
                .fg(state_color(container.state))
                .add_modifier(Modifier::BOLD),
            bg,
        );
        let name_budget = area.width.saturating_sub(18) as usize;
        let name = truncate_cols(
            if container.names.is_empty() {
                short_container_id(&container.id)
            } else {
                &container.names
            },
            name_budget,
        );
        lines.push(Line::from(vec![
            Span::styled(if selected { "› " } else { "  " }, style),
            Span::styled(pad_right(&name, name_budget), style),
            Span::styled(" ", style),
            Span::styled(pad_right(container.state.label(), 10), state_style),
        ]));

        if row_y < area.y + area.height {
            app.hit_registry
                .register_row(area.x, row_y, area.width, ClickAction::ContainerSelect(idx));
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}

pub fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let inner = Rect::new(
        area.x + 1,
        area.y,
        area.width.saturating_sub(1),
        area.height,
    );
    let mut lines = Vec::new();
    let Some(container) = app.selected_container() else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            i18n::containers_empty(),
            Style::default().fg(app.theme.fg_secondary),
        )));
        f.render_widget(Paragraph::new(lines), inner);
        return;
    };

    let title = if container.names.is_empty() {
        short_container_id(&container.id)
    } else {
        &container.names
    };
    lines.push(Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(app.theme.fg_primary)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    push_field(&mut lines, app, "ID", &container.id);
    push_field(&mut lines, app, "Image", &container.image);
    push_field(&mut lines, app, "State", container.state.label());
    push_field(&mut lines, app, "Status", &container.status);
    push_field(&mut lines, app, "Ports", &container.ports);
    push_field(&mut lines, app, "Created", &container.created);
    push_field(&mut lines, app, "Command", &container.command);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "r refresh  s start  x stop  R restart",
        Style::default().fg(app.theme.fg_secondary),
    )));
    if app.container_action_in_flight {
        lines.push(Line::from(Span::styled(
            "container action running...",
            Style::default().fg(Color::Cyan),
        )));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn push_field(lines: &mut Vec<Line<'static>>, app: &App, label: &'static str, value: &str) {
    lines.push(Line::from(vec![
        Span::styled(
            format!("{label:<8}"),
            Style::default()
                .fg(app.theme.fg_secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(app.theme.fg_primary)),
    ]));
}

fn state_color(state: ContainerState) -> Color {
    match state {
        ContainerState::Running => Color::Green,
        ContainerState::Exited => Color::Gray,
        ContainerState::Paused => Color::Yellow,
        ContainerState::Restarting => Color::Cyan,
        ContainerState::Created => Color::Blue,
        ContainerState::Dead => Color::Red,
        ContainerState::Other => Color::Magenta,
    }
}

fn apply_bg(style: Style, bg: Option<Color>) -> Style {
    if let Some(bg) = bg {
        style.bg(bg)
    } else {
        style
    }
}

fn truncate_cols(s: &str, max_cols: usize) -> String {
    if UnicodeWidthStr::width(s) <= max_cols {
        return s.to_string();
    }
    let mut out = String::new();
    let mut used = 0usize;
    for ch in s.chars() {
        let w = UnicodeWidthStr::width(ch.to_string().as_str());
        if used + w + 1 > max_cols {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    out
}

fn pad_right(s: &str, cols: usize) -> String {
    let width = UnicodeWidthStr::width(s);
    if width >= cols {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(cols - width))
    }
}
