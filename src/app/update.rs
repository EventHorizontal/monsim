use super::*;

/// Returns `ControlFlow` indicating whether the `App` should stop looping or not.
pub(super) fn update_app_state(
    app: &mut AppInstance,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    simulator: &mut BattleSimulator,
    receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
) -> AppResult<ControlFlow<Nothing>> {
    match app.state {
        AppState::Processing(ref processing_state) => {
            match processing_state.clone() {
                ProcessingState::AwaitingUserInput(available_actions) => {
                    process_awaiting_input_state(
                        app,
                        terminal,
                        simulator, 
                        &receiver, 
                        available_actions
                    )?;
                },
                ProcessingState::Simulating(chosen_actions) => {
                    process_simulation(simulator, chosen_actions, app);
                },
            }
        }
        AppState::PromptSwitchOut(SwitchOutState { team_id, .. }) => { 
            process_switch_out_state(
                app,
                terminal,
                simulator, 
                &receiver,
                team_id
            )?; 
        }, 
        AppState::Exiting => { return Ok(ControlFlow::Break(NOTHING)); }
    }
    Ok(ControlFlow::Continue(NOTHING))
}

fn process_switch_out_state(
    app: &mut AppInstance,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    simulator: &mut BattleSimulator,
    receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
    team_id: TeamID,
) -> AppResult<Nothing> {

    let app_state = &mut app.state;
    let team_ui_state = match team_id {
        TeamID::Allies => &mut app.ally_panel_ui_state,
        TeamID::Opponents => &mut app.opponent_panel_ui_state,
    };
    let message_log_ui_state = &mut app.message_log_ui_state;
    let battle = &mut simulator.battle;

    let switch_out_state = match app_state {
        AppState::PromptSwitchOut(switch_out_state) => switch_out_state,
        _ => unreachable!("App is checked to be in the PromptSwitchOut state before calling this function.")
    };

    use KeyEventKind::Release;
    match receiver.recv()? {
        TuiEvent::Input(event) => {
            match (event.code, event.kind) {
                (KeyCode::Up, Release) => {
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
                (KeyCode::Down, Release) => {
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
                (KeyCode::Enter, Release) => {
                    match switch_out_state.list_state.selected() {
                        Some(index) => {
                            let benched_battler_uid = Some(switch_out_state.list_of_choices[index].0);
                            team_ui_state.selected_action = Some((team_ui_state.list_items.len() - 1, ActionChoice::SwitchOut { 
                                active_battler_uid: switch_out_state.switching_battler, 
                                benched_battler_uid,
                            }));
                        },
                        None => {
                            battle.push_messages(
                                &[
                                    &"Simulator: Switch partner was not chosen... please select a battler to switch to before activating the simulation.",
                                    &"---",
                                    &EMPTY_LINE
                                ]
                            );
                            AppInstance::snap_message_log_scroll_index_to_turn_end(message_log_ui_state, battle);
                        },
                    }
                    *app_state = AppState::Processing(ProcessingState::AwaitingUserInput(battle.available_actions()));
                }
                (KeyCode::Esc, Release) =>  {
                    terminate(terminal, app_state)?; 
                    return Ok(NOTHING);
                }
                _ => NOTHING
            }
        },
        TuiEvent::Tick => NOTHING,
    }
    Ok(NOTHING)
}

fn process_awaiting_input_state(
    app: &mut AppInstance,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    simulator: &mut BattleSimulator,
    receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
    available_actions: AvailableActions,
) -> AppResult<Nothing> {
    match receiver.recv()? {
        TuiEvent::Input(event) => {
            use KeyEventKind::Release;
            use ProcessingState::Simulating;
            
            let was_escape_key_released = (KeyCode::Esc, Release) == (event.code, event.kind);
            if was_escape_key_released { 
                terminate(terminal, &mut app.state)?; 
                return Ok(NOTHING); 
            }
            
            let is_battle_finished = simulator.sim_state == SimState::BattleFinished;
            if not!(is_battle_finished) {
                match (event.code, event.kind) {
                    (KeyCode::Up, Release) => {
                        match app.currently_selected_widget {
                            SelectableWidget::MessageLog => { 
                                app.scroll_message_log_up();
                            },
                            SelectableWidget::AllyChoices => { app.ally_panel_ui_state.scroll_selection_up() },
                            SelectableWidget::OpponentChoices => { app.opponent_panel_ui_state.scroll_selection_up(); },
                            SelectableWidget::AllyRoster => { /* does nothing for now */ },
                            SelectableWidget::OpponentRoster => { /* does nothing for now */},
                        }
                    },
                    (KeyCode::Down, Release) => {
                        match app.currently_selected_widget {
                            SelectableWidget::MessageLog => { 
                                let message_log_length = simulator.battle.message_buffer.len(); 
                                app.scroll_message_log_down(message_log_length);
                            },
                            SelectableWidget::AllyChoices => { app.ally_panel_ui_state.scroll_selection_down() },
                            SelectableWidget::OpponentChoices => { app.opponent_panel_ui_state.scroll_selection_down(); },
                            SelectableWidget::AllyRoster => { /* does nothing for now */ },
                            SelectableWidget::OpponentRoster => { /* does nothing for now */},
                        }
                    },
                    (KeyCode::Left, Release) => {
                        app.currently_selected_widget.shift_left()
                    }
                    (KeyCode::Right, Release) => {
                        app.currently_selected_widget.shift_right()
                    },
                    (KeyCode::Enter, Release) => {
                        match app.currently_selected_widget {
                            SelectableWidget::AllyChoices => {
                                
                                process_choice_selection(
                                    available_actions.ally_team_available_actions,
                                    &mut app.ally_panel_ui_state, 
                                    simulator,
                                    &mut app.state,
                                );
                            },
                            SelectableWidget::MessageLog => NOTHING,
                            SelectableWidget::OpponentChoices => {
                                
                                process_choice_selection(
                                    available_actions.opponent_team_available_actions,
                                    &mut app.opponent_panel_ui_state, 
                                    simulator,
                                    &mut app.state,
                                );
                            },
                            SelectableWidget::AllyRoster => todo!(),
                            SelectableWidget::OpponentRoster => todo!(),
                        }
                    }
                    (KeyCode::Tab, Release) => { 
                        if let (Some(ally_selected_action), Some(opponent_selected_action)) =
                            (app.ally_panel_ui_state.selected_action, app.opponent_panel_ui_state.selected_action)
                        {
                            let chosen_actions = [
                                ally_selected_action,
                                opponent_selected_action,
                            ];
                            app.state = AppState::Processing(Simulating(chosen_actions));
                        } else {
                            simulator.battle.push_messages(
                                &[
                                    &"Simulator: Actions were not chosen... please select something before activating the simulation.",
                                    &"---",
                                    &EMPTY_LINE
                                ]
                            );
                            AppInstance::snap_message_log_scroll_index_to_turn_end(&mut app.message_log_ui_state, &mut simulator.battle);                  
                        }
                    },
                    _ => NOTHING,
                }
            } else { // Battle is finished
                match (event.code, event.kind) {
                    (KeyCode::Up, Release) => { app.scroll_message_log_up(); },
                    (KeyCode::Down, Release) => { 
                        let message_log_length = simulator.battle.message_buffer.len(); 
                        app.scroll_message_log_down(message_log_length); 
                    },
                    _ => NOTHING,
                }	
            }
        },
        TuiEvent::Tick => NOTHING,
    }
    Ok(NOTHING)
}

fn process_simulation(simulator: &mut BattleSimulator, chosen_actions: [EnumeratedActionChoice; 2], app: &mut AppInstance<'_>) {
    let result = simulator.simulate_turn(chosen_actions);
    match result {
        Ok(_) => {
            simulator.battle.push_message(&"Simulator: The turn was calculated successfully.");
        }
        Err(error) => simulator.battle.push_message(&format!["Simulator: {:?}", error]),
    }
                        
    if simulator.sim_state == SimState::BattleFinished {
        simulator.battle.push_messages(&[&EMPTY_LINE, &"The battle ended."]);
    }
    simulator.battle.push_messages(&[&"---", &EMPTY_LINE]);
                        
    let available_actions = simulator.battle.available_actions();
    app.state = AppState::Processing(ProcessingState::AwaitingUserInput(available_actions));
    app.regenerate_ui_data(&mut simulator.battle, available_actions);
}

fn process_choice_selection(team_available_actions: TeamAvailableActions, team_ui_state: &mut TeamUiState, battle_sim: &mut BattleSimulator, app_state: &mut AppState) {
    let switch_action_index = team_available_actions.switch_out_action_index();
    let switch_selected = team_ui_state.list_state.selected() == switch_action_index;
    let team_id = team_ui_state.team_id;
                                
    let list_of_choices = battle_sim.battle.switch_partners_on_team(team_id)
        .iter()
        .map(|battler| {
            (battler.uid, battler.monster.nickname, battler.monster.species.name)
        })
        .collect::<Vec<_>>();

    let switch_out_state = SwitchOutState {
        switching_battler: battle_sim.battle.active_battlers_on_team(team_id).0.uid,
        team_id, 
        list_of_choices, 
        list_state: new_list_state(),
    };

    if switch_selected {
        *app_state = AppState::PromptSwitchOut(switch_out_state);
    } else {
        AppInstance::confirm_selection(team_ui_state, team_available_actions);
    }
}