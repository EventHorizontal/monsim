use std::io::Stdout;

use monsim::BattleContext;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::CompletedFrame,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
    Terminal,
};

use crate::{AppState, ScrollableWidgets};

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
                .as_ref(),
            )
            .split(chunks[0]);

        // Ally Active Monster Status Widget
        let ally_active_monster_text =
            BattleContext::monster_status_string(context.ally_team.active_battler());
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
        let ally_choice_menu_widget = List::new(app_state.ally_list_items.clone())
            .block(
                Block::default()
                    .title(
                        if app_state.selected_list == ScrollableWidgets::AllyTeamChoices {
                            Span::styled(
                                " Ally Monster Choices ",
                                Style::default().fg(Color::Yellow),
                            )
                        } else {
                            Span::raw(" Ally Monster Choices ")
                        },
                    )
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(
            ally_choice_menu_widget,
            ally_panel_chunks[1],
            &mut app_state.ally_list_state,
        );

        // Opponent Team Roster Widget
        let ally_team_status_text = context.ally_team_string();

        let ally_team_status_widget = Paragraph::new(ally_team_status_text)
            .block(
                Block::default()
                    .title({
                        if app_state.selected_list == ScrollableWidgets::AllyTeamStatus {
                            Span::styled(" Ally Team Status ", Style::default().fg(Color::Yellow))
                        } else {
                            Span::raw(" Ally Team Status ")
                        }
                    })
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(ally_team_status_widget, ally_panel_chunks[2]);

        // Message Log Widget
        let text = app_state
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
                    .title(
                        if app_state.selected_list == ScrollableWidgets::MessageLog {
                            Span::styled(" Message Log ", Style::default().fg(Color::Yellow))
                        } else {
                            Span::raw(" Message Log ")
                        },
                    )
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
                .as_ref(),
            )
            .split(chunks[2]);

        // Opponent Active Monster Status Widget
        let opponent_active_monster_text =
            BattleContext::monster_status_string(context.opponent_team.active_battler());
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
        let opponent_choice_menu_widget = List::new(app_state.opponent_list_items.clone())
            .block(
                Block::default()
                    .title(
                        if app_state.selected_list == ScrollableWidgets::OpponentTeamChoices {
                            Span::styled(
                                " Opponent Monster Choices ",
                                Style::default().fg(Color::Yellow),
                            )
                        } else {
                            Span::raw(" Opponent Monster Choices ")
                        },
                    )
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(
            opponent_choice_menu_widget,
            opponent_panel_chunks[1],
            &mut app_state.opponent_list_state,
        );

        // Opponent Team Roster Widget
        let opponent_team_status_text = context.opponent_team_string();

        let opponent_team_status_widget = Paragraph::new(opponent_team_status_text)
            .block(
                Block::default()
                    .title(
                        if app_state.selected_list == ScrollableWidgets::OpponentTeamStatus {
                            Span::styled(
                                " Opponent Team Status ",
                                Style::default().fg(Color::Yellow),
                            )
                        } else {
                            Span::raw(" Opponent Team Status ")
                        },
                    )
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(opponent_team_status_widget, opponent_panel_chunks[2]);
    })
}
