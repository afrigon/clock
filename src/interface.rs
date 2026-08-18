use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};
use ratatui::Frame;

use crate::application::{Application, Tab, TimerScreen};
use crate::clock;
use crate::configuration::Configuration;
use crate::digits;

const HELP_HINT: &str = "? - help";

fn tab_layout() -> [(Tab, u16, &'static str); 3] {
    let labels = [
        (Tab::Timer, "F1 timer"),
        (Tab::Clock, "F2 clock"),
        (Tab::Stopwatch, "F3 stopwatch"),
    ];
    let mut column = 1;
    labels.map(|(tab, label)| {
        let start = column;
        column += label.len() as u16 + 3;
        (tab, start, label)
    })
}

pub fn tab_at(column: u16) -> Option<Tab> {
    tab_layout()
        .into_iter()
        .find(|(_, start, label)| column >= *start && column < start + label.len() as u16)
        .map(|(tab, ..)| tab)
}

pub fn draw(frame: &mut Frame, application: &Application) {
    let area = frame.area();
    let content = if application.fullscreen {
        area
    } else {
        Rect {
            y: area.y + 1,
            height: area.height.saturating_sub(1),
            ..area
        }
    };
    let foreground = Style::new().fg(application.configuration.foreground);
    match application.tab {
        Tab::Timer => draw_timer(frame, content, application),
        Tab::Clock => draw_clock(frame, content, application, foreground),
        Tab::Stopwatch => draw_stopwatch(frame, content, application, foreground),
    }
    if !application.fullscreen {
        draw_top_bar(frame, area, application.tab);
    }
    if application.help_visible {
        draw_help(frame, &application.configuration);
    }
}

fn draw_timer(frame: &mut Frame, content: Rect, application: &Application) {
    match &application.timer_screen {
        TimerScreen::Input { input, error } => {
            draw_timer_input(frame, content, &application.configuration, input, error.as_deref());
        }
        TimerScreen::Running { timer, .. } => {
            let foreground = if timer.remaining_milliseconds() <= 0 {
                application.configuration.timer.overtime_foreground
            } else {
                application.configuration.foreground
            };
            let mut style = Style::new().fg(foreground);
            let mut text = timer.display();
            if timer.is_paused() {
                style = style.add_modifier(Modifier::DIM);
                if blink_phase_hidden() {
                    text = text.replace(':', " ");
                }
            }
            digits::render(frame, content, &text, style);
        }
    }
}

fn blink_phase_hidden() -> bool {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since_epoch| since_epoch.as_millis() % 1000 < 500)
        .unwrap_or(false)
}

fn draw_clock(frame: &mut Frame, content: Rect, application: &Application, foreground: Style) {
    let configuration = &application.configuration.clock;
    let digits_area = digits::render(frame, content, &clock::display(configuration), foreground);
    let lines: Vec<Line> = [
        clock::date(configuration)
            .map(|date| Line::styled(date, foreground.add_modifier(Modifier::DIM))),
        configuration
            .message
            .clone()
            .map(|message| Line::styled(message, Style::new().add_modifier(Modifier::DIM))),
    ]
    .into_iter()
    .flatten()
    .collect();
    if lines.is_empty() {
        return;
    }
    let y = digits_area.y + digits_area.height + 1;
    let height = (lines.len() as u16).min((content.y + content.height).saturating_sub(y));
    if height == 0 {
        return;
    }
    let below = Rect {
        y,
        height,
        ..content
    };
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), below);
}

fn draw_stopwatch(frame: &mut Frame, content: Rect, application: &Application, foreground: Style) {
    let milliseconds = format!(".{:03}", application.stopwatch.milliseconds_part());
    digits::render_with_suffix(
        frame,
        content,
        &application.stopwatch.display(),
        &milliseconds,
        foreground,
        foreground.add_modifier(Modifier::DIM),
    );
}

fn draw_top_bar(frame: &mut Frame, area: Rect, active_tab: Tab) {
    let mut spans = vec![Span::raw(" ")];
    for (tab, _, label) in tab_layout() {
        let style = if tab == active_tab {
            Style::new().add_modifier(Modifier::BOLD)
        } else {
            Style::new().add_modifier(Modifier::DIM)
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw("   "));
    }
    let bar = Rect { height: 1, ..area };
    frame.render_widget(Paragraph::new(Line::from(spans)), bar);

    let hint_width = HELP_HINT.len() as u16;
    if area.width > hint_width {
        let hint = Rect {
            x: area.x + area.width - hint_width - 1,
            width: hint_width,
            ..bar
        };
        frame.render_widget(
            Paragraph::new(HELP_HINT).style(Style::new().add_modifier(Modifier::DIM)),
            hint,
        );
    }
}

fn draw_timer_input(
    frame: &mut Frame,
    area: Rect,
    configuration: &Configuration,
    input: &str,
    error: Option<&str>,
) {
    let style = Style::new().fg(configuration.foreground);
    let dim = Style::new().add_modifier(Modifier::DIM);
    let lines = vec![
        Line::from("set a timer for"),
        Line::default(),
        Line::from(format!("{input}▌")),
        Line::default(),
        Line::styled("e.g. 5 (minutes) · 90s · 1h30m · 5:00", dim),
        Line::styled("enter to start", dim),
        Line::default(),
        Line::from(error.unwrap_or_default()),
    ];
    let height = lines.len() as u16;
    let centered = Rect {
        y: area.y + area.height.saturating_sub(height) / 2,
        height: height.min(area.height),
        ..area
    };
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).style(style),
        centered,
    );
}

fn draw_help(frame: &mut Frame, configuration: &Configuration) {
    let entries = [
        ("?", "toggle this help"),
        ("F1 F2 F3", "jump to a tab (clicking works too)"),
        ("← → · h l", "cycle tabs"),
        ("f", "toggle fullscreen"),
        ("m", "music on · off"),
        ("q · esc", "quit"),
        ("", ""),
        ("", "timer"),
        ("0-9 h m s :", "type a duration"),
        ("enter", "start the timer"),
        ("space", "pause · resume"),
        ("r", "restart the timer"),
        ("q · esc", "back to duration entry"),
        ("", ""),
        ("", "stopwatch"),
        ("space", "start · pause"),
        ("r", "reset"),
    ];
    let lines: Vec<Line> = entries
        .iter()
        .map(|(key, description)| {
            if key.is_empty() {
                Line::styled(
                    format!("  {description}"),
                    Style::new().add_modifier(Modifier::DIM),
                )
            } else {
                Line::from(format!("  {key:>11}   {description}"))
            }
        })
        .collect();
    let width = lines.iter().map(Line::width).max().unwrap_or(0) as u16 + 6;
    let height = lines.len() as u16 + 2;
    let area = frame.area();
    let popup = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width: width.min(area.width),
        height: height.min(area.height),
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" keys ")
        .title_alignment(Alignment::Center);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::new().fg(configuration.foreground))
            .block(block),
        popup,
    );
}
