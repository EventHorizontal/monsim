use std::io::Stdout;

use tui::{Terminal, backend::CrosstermBackend, terminal::CompletedFrame, layout::{Layout, Direction, Constraint, Alignment}, widgets::{ListState, List, Block, Borders, Paragraph, Wrap}, style::{Style, Color, Modifier}, text::{Spans, Span}};

use crate::AppState;

pub fn render<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    app_state: &mut AppState,
) -> std::io::Result<CompletedFrame<'a>> {
    terminal.draw(|frame| {
        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(frame.size());

        // Ally Monster Stats Widget
        let mut ally_list_state = ListState::default();
        ally_list_state.select(Some(2));
        let ally_widget = List::new(app_state.ally_list_items.clone())
            .block(
                Block::default()
                    .title(" Ally Team Choices ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(ally_widget, chunks[0], &mut app_state.ally_list_state);

        // Message Log Widget
        let text = app_state
            .message_buffer
            .iter()
            .map(|element| Spans::from(Span::raw(element)))
            .collect::<Vec<_>>();
        let paragraph_widget = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Message Log ")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph_widget, chunks[1]);

        // Opponent Monster Stats Widget
        let opponent_widget = List::new(app_state.opponent_list_items.clone())
            .block(
                Block::default()
                    .title(" Opponent Team Choices ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(
            opponent_widget,
            chunks[2],
            &mut app_state.opponent_list_state,
        );
    })
}