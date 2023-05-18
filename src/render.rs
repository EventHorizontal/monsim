use std::io::Stdout;

use tui::{Terminal, backend::CrosstermBackend, terminal::CompletedFrame, layout::{Layout, Direction, Constraint, Alignment}, widgets::{ListState, List, Block, Borders, Paragraph, Wrap}, style::{Style, Modifier}, text::{Spans, Span}};

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
		
		// Divide Chunks on Ally Panels
		let ally_panel_chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					Constraint::Percentage(33),
					Constraint::Length(6),
					Constraint::Length(6),
				]
				.as_ref()
			)
			.split(chunks[0]);
        // Ally Monster Stats Widget
		let ally_stats_widget = Block::default()
			.title(" Ally Active Monster ")
			.borders(Borders::ALL);
		frame.render_widget(ally_stats_widget, ally_panel_chunks[0]);	
		
		// Ally Choice List Menu
        let mut ally_list_state = ListState::default();
        ally_list_state.select(Some(2));
        let ally_choices_widget = List::new(app_state.ally_list_items.clone())
            .block(
                Block::default()
                    .title(" Ally Monster Choices ")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(ally_choices_widget, ally_panel_chunks[1], &mut app_state.ally_list_state);

		// Opponent Team Roster Widget
		let ally_roster_widget = Block::default()
			.title(" Ally Team ")
			.borders(Borders::ALL);
		frame.render_widget(ally_roster_widget, ally_panel_chunks[2]);

        // Message Log Widget
        let 
		text = app_state
            .message_buffer
            .iter()
			.enumerate()
            .filter_map(|(idx, element)| {
				if idx >= app_state.message_log_scroll_idx {
					Some(Spans::from(Span::raw(element)))
				} else {
					None
				}
			})
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

		// Divide Chunks on Opponent Panel
        let opponent_panel_chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					Constraint::Percentage(33),
					Constraint::Length(6),
					Constraint::Length(6),
				]
				.as_ref()
			)
			.split(chunks[2]);
        
		// Opponent Monster Stats Widget
		let opponent_stats_widget = Block::default()
			.title(" Opponent Active Monster ")
			.borders(Borders::ALL);
		frame.render_widget(opponent_stats_widget, opponent_panel_chunks[0]);	
		
		// Opponent Choice List Menu
        let opponent_widget = List::new(app_state.opponent_list_items.clone())
            .block(
                Block::default()
                    .title(" Opponent Monster Choices ")
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(
            opponent_widget,
            opponent_panel_chunks[1],
            &mut app_state.opponent_list_state,
        );

		// Opponent Team Roster Widget
		let opponent_roster_widget = Block::default()
			.title(" Opponent Team ")
			.borders(Borders::ALL);
		frame.render_widget(opponent_roster_widget, opponent_panel_chunks[2]);
    })
}