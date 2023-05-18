use std::io::Stdout;

use monsim::BattleContext;
use tui::{Terminal, backend::CrosstermBackend, terminal::CompletedFrame, layout::{Layout, Direction, Constraint, Alignment}, widgets::{ListState, List, Block, Borders, Paragraph, Wrap}, style::{Style, Modifier}, text::{Spans, Span}};

use crate::AppState;

pub fn render<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    app_state: &mut AppState,
	context: &BattleContext,
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

        // Ally Active Monster Status Widget
		let ally_active_monster_text = BattleContext::monster_status_string(context.ally_team.active_battler());
		let ally_stats_widget = Paragraph::new(ally_active_monster_text)
			.block(
				Block::default()
					.title(" Ally Active Monster ")
					.borders(Borders::ALL),
			)
			.alignment(Alignment::Left)
			.wrap(Wrap { trim: true });
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
		let ally_team_text = context.ally_team_string();

		let ally_roster_widget = Paragraph::new(ally_team_text)
		.block(
			Block::default()
				.title(" Opponent Team ")
				.borders(Borders::ALL),
		)
		.alignment(Alignment::Left)
		.wrap(Wrap { trim: true });
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
        
		// Opponent Active Monster Status Widget
		let opponent_active_monster_text = BattleContext::monster_status_string(context.opponent_team.active_battler());
		let opponent_stats_widget = Paragraph::new(opponent_active_monster_text)
			.block(
				Block::default()
					.title(" Opponent Active Monster ")
					.borders(Borders::ALL),
			)
			.alignment(Alignment::Left)
			.wrap(Wrap { trim: true });
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
		let opponent_team_text = context.opponent_team_string();

		let opponent_roster_widget = Paragraph::new(opponent_team_text)
		.block(
			Block::default()
				.title(" Opponent Team ")
				.borders(Borders::ALL),
		)
		.alignment(Alignment::Left)
		.wrap(Wrap { trim: true });
	frame.render_widget(opponent_roster_widget, opponent_panel_chunks[2]);
    })
}