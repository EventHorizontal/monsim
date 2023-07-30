use super::*;

#[must_use]
pub(super) fn update_switch_out_state<'a>(
    ui: &mut Ui,
    battle: &mut Battle,
    receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
    team_id: TeamID,
    input_key: KeyCode,
) -> AppResult<Option<UiState<'a>>> {

    let team_ui_state = match team_id {
        TeamID::Allies => &mut ui.ally_panel_ui_state,
        TeamID::Opponents => &mut ui.opponent_panel_ui_state,
    };

    let message_log_ui_state = &mut ui.message_log_ui_state;

    let switch_out_state = match &mut ui.state {
        UiState::PromptSwitchOut(switch_out_state) => switch_out_state,
        _ => unreachable!("App is checked to be in the PromptSwitchOut state before calling this function.")
    };

    match input_key {
        KeyCode::Up => {
            match switch_out_state.list_state.selected() {
                Some(index) => {
                    let number_of_choices = switch_out_state.list_of_choices.len();
                    let new_index = (index + number_of_choices - 1) % number_of_choices;
                    switch_out_state.list_state.select(Some(new_index));
                },
                None => {
                    switch_out_state.list_state.select(Some(0));
                },
            }
        },
        KeyCode::Down => {
            match switch_out_state.list_state.selected() {
                Some(index) => {
                    let number_of_choices = switch_out_state.list_of_choices.len();
                    let new_index = (index + 1) % number_of_choices;
                    switch_out_state.list_state.select(Some(new_index));
                },
                None => {
                    switch_out_state.list_state.select(Some(0));
                },
            }
        },
        KeyCode::Enter => {
            match switch_out_state.list_state.selected() {
                Some(index) => {
                    let benched_battler_uid = switch_out_state.list_of_choices[index].0;
                    team_ui_state.selected_action = Some((team_ui_state.list_items.len() - 1, ChosenAction::SwitchOut { 
                        switcher_uid: switch_out_state.switching_battler, 
                        switchee_uid: benched_battler_uid,
                    }));
                },
                None => {
                    battle.push_messages(
                        &[
                            &"Simulator: Switchee was not chosen. Please select a battler to switch to before activating the simulation.",
                            &"---",
                            &EMPTY_LINE
                        ]
                    );
                    Ui::snap_message_log_scroll_index_to_turn_end(message_log_ui_state, battle);
                },
            }
            return Ok(Some(UiState::Processing(ProcessingState::FreeInput(battle.available_actions()))));
        }
        KeyCode::Esc =>  { return Ok(Some(UiState::Terminating)) },
        _ => NOTHING
    }
    Ok(None)
}

#[must_use]
pub(super) fn update_ui_from_free_input<'a>(
    ui: &mut Ui, 
    battle: &mut Battle, 
    available_actions: &AvailableActions, 
    input_key: KeyCode
) -> Option<UiState<'a>> {
    
    if input_key == KeyCode::Esc {
        return Some(UiState::Terminating);
    }

    match input_key {
        KeyCode::Up => {
            match ui.currently_selected_widget {
                SelectableWidget::MessageLog => { 
                    ui.scroll_message_log_up();
                },
                SelectableWidget::AllyChoices => { ui.ally_panel_ui_state.scroll_selection_up() },
                SelectableWidget::OpponentChoices => { ui.opponent_panel_ui_state.scroll_selection_up(); },
                SelectableWidget::AllyRoster => unreachable!(),
                SelectableWidget::OpponentRoster => unreachable!(),
            }
        },
        KeyCode::Down => {
            match ui.currently_selected_widget {
                SelectableWidget::MessageLog => { 
                    let message_log_length = battle.message_buffer.len(); 
                    ui.scroll_message_log_down(message_log_length);
                },
                SelectableWidget::AllyChoices => { ui.ally_panel_ui_state.scroll_selection_down() },
                SelectableWidget::OpponentChoices => { ui.opponent_panel_ui_state.scroll_selection_down(); },
                SelectableWidget::AllyRoster => unreachable!(),
                SelectableWidget::OpponentRoster => unreachable!(),
            }
        },
        KeyCode::Left => {
            ui.currently_selected_widget.shift_left()
        }
        KeyCode::Right => {
            ui.currently_selected_widget.shift_right()
        },
        KeyCode::Enter => {
            match ui.currently_selected_widget {
                SelectableWidget::AllyChoices => { 
                    update_ui_selection_state(
                        &mut ui.ally_panel_ui_state, 
                        battle,
                        available_actions.ally_team_available_actions,
                    );
                },
                SelectableWidget::MessageLog => NOTHING,
                SelectableWidget::OpponentChoices => {
                    update_ui_selection_state(
                        &mut ui.opponent_panel_ui_state, 
                        battle,
                        available_actions.opponent_team_available_actions,
                    );
                },
                SelectableWidget::AllyRoster => todo!(),
                SelectableWidget::OpponentRoster => todo!(),
            }
        }
        KeyCode::Tab => { 
            if let (Some(ally_selected_action), Some(opponent_selected_action)) =
                (ui.ally_panel_ui_state.selected_action, ui.opponent_panel_ui_state.selected_action)
            {
                let chosen_actions = [
                    ally_selected_action,
                    opponent_selected_action,
                ];
                ui.state = UiState::Processing(ProcessingState::Simulation(chosen_actions));
            } else {
                battle.push_messages(
                    &[
                        &"Simulator: Actions were not chosen... please select something before activating the simulation.",
                        &"---",
                        &EMPTY_LINE
                    ]
                );
                Ui::snap_message_log_scroll_index_to_turn_end(&mut ui.message_log_ui_state, battle);                  
            }
        },
        _ => NOTHING,
    }

    None
}

pub(super) fn update_ui_from_simulation(app: &mut Ui, simulator: &mut BattleSimulator, chosen_actions: ChosenActionsForTurn) {
    match simulator.simulate_turn(chosen_actions) {
        Ok(_) => simulator.battle.push_message(&"Simulator: The turn was calculated successfully."),
        Err(error) => simulator.battle.push_message(&format!["Simulator: {:?}", error]),
    }
                        
    if simulator.sim_state == SimState::BattleFinished {
        simulator.battle.push_messages(&[&EMPTY_LINE, &"The battle ended."]);
    }
    simulator.battle.push_messages(&[&"---", &EMPTY_LINE]);
}

fn update_ui_selection_state<'a>(
    team_ui_state: &mut TeamUiState, 
    battle: &mut Battle, 
    team_available_actions: TeamAvailableActions, 
) -> Option<UiState<'a>> {
    let switch_action_index = team_available_actions.switch_out_action_index();
    let switch_selected = team_ui_state.list_state.selected() == switch_action_index;
    let team_id = team_ui_state.team_id;
                                
    let list_of_choices = battle.switch_partners_on_team(team_id)
        .iter()
        .map(|battler| {
            (battler.uid, battler.monster.nickname, battler.monster.species.name)
        })
        .collect::<Vec<_>>();

    let switch_out_state = SwitchOutState {
        switching_battler: battle.active_battlers_on_team(team_id).0.uid,
        team_id, 
        list_of_choices, 
        list_state: new_list_state(),
    };

    if switch_selected {
        Some(UiState::PromptSwitchOut(switch_out_state))
    } else {
        if let Some(selected_index) = team_ui_state.list_state.selected() {
            team_ui_state.selected_action = team_available_actions[selected_index].map( |(idx, choosable_action)| {
                match choosable_action {
                    ChoosableAction::Move(move_uid) => (idx, ChosenAction::Move { move_uid, target_uid: todo!() }),
                    ChoosableAction::SwitchOut { switcher_uid } => (idx, ChosenAction::SwitchOut { switcher_uid, switchee_uid: todo!() }),
                }
            }); 
        };
        None
    }
}

#[must_use]
pub(super) fn update_ui_from_post_battle_input<'a>(
    ui: &mut Ui, 
    battle: &mut Battle,
    input_key: KeyCode
) -> Option<UiState<'a>> {
    
    match input_key {
        KeyCode::Up => { ui.scroll_message_log_up(); },
        KeyCode::Down => { 
            let message_log_length = battle.message_buffer.len(); 
            ui.scroll_message_log_down(message_log_length); 
        },
        _ => NOTHING,
    }
    
    None	
}