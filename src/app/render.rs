use super::*;

pub(super) fn render_interface<'a>(
    ui: &mut Ui,
    app_state: &mut AppState,
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>, 
    message_buffer: &MessageBuffer,
) -> std::io::Result<CompletedFrame<'a>> {

    terminal.draw(|frame| {
        let chunks = divide_screen_into_chunks(frame);
        
        let ally_panel_chunks = divide_panel_into_chunks(chunks[0]);
        let ally_stats_widget = construct_stats_panel_widget(&ui.ally_panel_ui_state); 
        let ally_choice_menu_widget = construct_choice_menu_widget(&ui.ally_panel_ui_state, ui.currently_selected_widget);
        let ally_team_roster_widget = construct_roster_widget(&ui.ally_panel_ui_state, ui.currently_selected_widget);

        let message_log_widget = construct_message_log_widget(message_buffer, &mut ui.message_log_ui_state, ui.currently_selected_widget);
        
        let opponent_panel_chunks = divide_panel_into_chunks(chunks[2]);
        let opponent_stats_widget = construct_stats_panel_widget(&ui.opponent_panel_ui_state); 
        let opponent_choice_menu_widget = construct_choice_menu_widget(&ui.opponent_panel_ui_state, ui.currently_selected_widget);
        let opponent_team_roster_widget = construct_roster_widget(&ui.opponent_panel_ui_state, ui.currently_selected_widget);

        if let AppState::PromptSwitchOut(ref mut switch_out_state) = app_state {
            
            let message_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(80),
                    Constraint::Max(5),
                ]
                .as_ref(),
            )
            .split(chunks[1]);

            let list_of_choices = switch_out_state.list_of_choices
                .clone()
                .iter()
                .map(|(_, benched_teammate_nickname, benched_teammate_species_name)| { ListItem::new(format!["{} the {}", *benched_teammate_nickname, benched_teammate_species_name])})
                .collect::<Vec<_>>();

            let switch_out_widget = List::new(list_of_choices)
                .block(
                    Block::default()
                        .title(Span::styled(" Switch out with? ", Style::default().fg(Color::Yellow)))
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            frame.render_widget(message_log_widget, message_chunks[0]);
            frame.render_stateful_widget(switch_out_widget, message_chunks[1], &mut switch_out_state.list_state);
        } else {
            frame.render_widget(message_log_widget, chunks[1]);
        }

        frame.render_widget(ally_stats_widget, ally_panel_chunks[0]);
        let mut ally_list_state = ui.ally_panel_ui_state.list_state.clone();
        frame.render_stateful_widget(ally_choice_menu_widget, ally_panel_chunks[1], &mut ally_list_state); 
        frame.render_widget(ally_team_roster_widget, ally_panel_chunks[2]);
        
        
        frame.render_widget(opponent_stats_widget, opponent_panel_chunks[0]);
        let mut opponent_list_state = ui.opponent_panel_ui_state.list_state.clone();
        frame.render_stateful_widget(opponent_choice_menu_widget, opponent_panel_chunks[1], &mut opponent_list_state);
        frame.render_widget(opponent_team_roster_widget, opponent_panel_chunks[2]);
    })
}

fn divide_screen_into_chunks(frame: &mut Frame<CrosstermBackend<Stdout>>) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Min(25),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(frame.size())
}

fn divide_panel_into_chunks(chunk: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(33), Constraint::Length(8), Constraint::Length(6)].as_ref())
        .split(chunk)
}

fn construct_stats_panel_widget<'a>(team_ui_state: &'a TeamUiState) -> Paragraph<'a> {
    let team_name = team_ui_state.team_id;
    Paragraph::new(team_ui_state.active_battler_status.as_str())
        .block(Block::default().title(format![" {team_name} Active Monster "]).borders(Borders::ALL))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
}

fn construct_roster_widget<'a>(team_ui_state: &'a TeamUiState, currently_selected_list: SelectableWidget) -> Paragraph<'a> {
    let team_name = team_ui_state.team_id;
    let team_roster_list = match team_name {
        TeamID::Allies => SelectableWidget::AllyRoster,
        TeamID::Opponents => SelectableWidget::OpponentRoster,
    };
    Paragraph::new(team_ui_state.team_roster_status.as_str())
            .block(
                Block::default()
                    .title({
                        if currently_selected_list == team_roster_list {
                            Span::styled(format![" {team_name} Team Status "], Style::default().fg(Color::Yellow))
                        } else {
                            Span::raw(format![" {team_name} Team Status "])
                        }
                    })
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
}

fn construct_choice_menu_widget<'a>(team_ui_state: &'a TeamUiState, currently_selected_list: SelectableWidget) -> List<'a> {
    let team_name = team_ui_state.team_id;
    let team_choices_list = match team_name {
        TeamID::Allies => SelectableWidget::AllyChoices,
        TeamID::Opponents => SelectableWidget::OpponentChoices,
    };
    let currently_selected_choice = team_ui_state.selected_action.map(|it| { it.0 });
    let list_items = team_ui_state.list_items.iter().enumerate().map(|(i, list_item)| {
        if let Some(currently_selected_choice) = currently_selected_choice {
            if i == currently_selected_choice {
                list_item.clone().style(Style::default().fg(Color::Green))
            } else {
                list_item.clone()
            }
        } else {
            list_item.clone()
        }
    }).collect::<Vec<_>>();
    List::new(list_items)
        .block(
            Block::default()
                .title(if currently_selected_list == team_choices_list {
                    Span::styled(format![" {team_name} Monster Choices "], Style::default().fg(Color::Yellow))
                } else {
                    Span::raw(format![" {team_name} Monster Choices "])
                })
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
}

/// `message_log_scroll_index` is the index of  the first line of the message buffer to be rendered.
fn construct_message_log_widget<'a>(message_buffer: &'a MessageBuffer, message_log_ui_state:&'a mut MessageLogUiState, currently_selected_list: SelectableWidget) -> Paragraph<'a> {
    let text = message_buffer
        .iter()
        .enumerate()
        .filter_map(|(idx, element)| {
            if idx >= message_log_ui_state.message_log_scroll_idx { 
                if element.contains("Turn") {
                    Some(Spans::from(Span::styled(element, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))))
                } else {
                    Some(Spans::from(Span::raw(element)))
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(text)
        .block(
            Block::default()
                .title(if currently_selected_list == SelectableWidget::MessageLog {
                    Span::styled(" Message Log ", Style::default().fg(Color::Yellow))
                } else {
                    Span::raw(" Message Log ")
                })
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}
