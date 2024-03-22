use std::io::{self, StdoutLock, Write};

use monsim_utils::{Nothing, Team, NOTHING};

use crate::{tui::TuiResult, sim::{AvailableChoices, AvailableChoicesForTeam, BattleState, BattleSimulator, FullySpecifiedChoice, PartiallySpecifiedChoice, PerTeam}};

enum TurnStage {
    ChooseActions(AvailableChoices),
    SimulateTurn(PerTeam<FullySpecifiedChoice>),
    BattleEnded,
}

pub fn run(mut battle: BattleState) -> TuiResult<Nothing> {
    let mut turn_stage = TurnStage::ChooseActions(battle.available_choices());

    // We lock stdout so that we don't have to acquire the lock every time with `println!`
    let mut locked_stdout = io::stdout().lock();
    let mut last_turn_chosen_actions: Option<PerTeam<FullySpecifiedChoice>> = None;
    'main: loop {
        match turn_stage {
            TurnStage::ChooseActions(available_choices) => {

                // Check if any of the active monsters has fainted and needs to switched out
                for active_monster_uid in battle.active_monster_uids() {
                    if battle.monster(active_monster_uid).is_fainted {
                        if let Some(PartiallySpecifiedChoice::SwitchOut { switcher_uid, candidate_switchee_uids, .. }) = available_choices[active_monster_uid.team_uid].switch_out_choice() {
                            let switchee_names = candidate_switchee_uids.into_iter().flatten().map(|uid| battle.monster(uid).full_name()).enumerate();
                            writeln!(locked_stdout, "{} fainted! Choose a monster to switch with", battle.monster(active_monster_uid).name())?;
                            for (index, switchee_name) in switchee_names {
                                writeln!(locked_stdout, "[{}] {}", index + 1, switchee_name)?;
                            }
                            let chosen_switchee_index = input_as_usize(&mut locked_stdout, candidate_switchee_uids.iter().flatten().count()).unwrap();
                            let chosen_switchee_uid = candidate_switchee_uids[chosen_switchee_index].unwrap();
                            BattleSimulator::switch_out_between_turns(&mut battle, switcher_uid, chosen_switchee_uid)?;
                            last_turn_chosen_actions = None;
                        } else {
                            turn_stage = TurnStage::BattleEnded;
                            continue 'main;
                        }
                    }
                }

                let (ally_team_available_choices, opponent_team_available_choices) = available_choices.unwrap();
                
                write_empty_line(&mut locked_stdout)?;
                writeln!(locked_stdout, "Current Battle Status:")?;
                write_empty_line(&mut locked_stdout)?;

                writeln!(locked_stdout, "Ally Active Monster: {}", battle.monster(battle.ally_team().active_monster_uid).status_string())?;
                writeln!(locked_stdout, "Opponent Active Monster {}", battle.monster(battle.opponent_team().active_monster_uid).status_string())?;
                writeln!(locked_stdout, "Ally Team:")?;
                writeln!(locked_stdout, "{}", battle.ally_team().team_status_string())?;
                writeln!(locked_stdout, "Opponent Team:")?;
                writeln!(locked_stdout, "{}", battle.opponent_team().team_status_string())?;
                
                // Ally Team choices
                writeln!(locked_stdout, "Choose an Action for {}", battle.monster(battle.ally_team().active_monster_uid).full_name())?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&ally_team_available_choices, &mut locked_stdout, last_turn_chosen_actions.is_some())?;
                
                let ally_team_fully_specified_action = match translate_input_to_choices(&battle, Team::ally(ally_team_available_choices), &mut locked_stdout, last_turn_chosen_actions)? {
                    UIChoice::Quit => {
                        writeln!(locked_stdout, "Exiting...")?;
                        break 'main
                    },
                    UIChoice::Action(fully_specified_action) => fully_specified_action.expect_ally(),
                    UIChoice::Repeat(last_turn_chosen_actions) => {
                        turn_stage = TurnStage::SimulateTurn(last_turn_chosen_actions);
                        continue;
                    },
                };

                // Opponent choices
                writeln!(locked_stdout, "Choose an Action for {}", battle.monster(battle.opponent_team().active_monster_uid).full_name())?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&opponent_team_available_choices, &mut locked_stdout, last_turn_chosen_actions.is_some())?;
                
                let opponent_team_fully_specified_action = match translate_input_to_choices(&battle, Team::opponent(opponent_team_available_choices), &mut locked_stdout, last_turn_chosen_actions)? {
                    UIChoice::Quit => {
                        writeln!(locked_stdout, "Exiting...")?;
                        break 'main
                    },
                    UIChoice::Action(fully_specified_action) => fully_specified_action.expect_opponent(),
                    UIChoice::Repeat(last_turn_chosen_actions) => {
                        turn_stage = TurnStage::SimulateTurn(last_turn_chosen_actions);
                        continue;
                    },
                };

                // Package both team's choices up
                let chosen_actions = PerTeam::new(ally_team_fully_specified_action, opponent_team_fully_specified_action);
                last_turn_chosen_actions = Some(chosen_actions);

                turn_stage = TurnStage::SimulateTurn(chosen_actions);
            },
            TurnStage::SimulateTurn(chosen_actions_for_turn) => {

                battle.message_log.set_last_turn_cursor_to_log_length();
                
                BattleSimulator::simulate_turn(&mut battle, chosen_actions_for_turn)?;

                // Show the message log
                battle.message_log.show_last_turn_messages();
                
                if battle.is_finished {
                    turn_stage = TurnStage::BattleEnded;
                } else {
                    turn_stage = TurnStage::ChooseActions(battle.available_choices());
                }
            },
            TurnStage::BattleEnded => {
                println!("The battle ended successfully");
                break 'main;
            },
        }
    }
    Ok(NOTHING)
}

fn write_empty_line(locked_stdout: &mut StdoutLock<'_>) -> TuiResult<Nothing> {
    writeln!(locked_stdout, "")?;
    Ok(NOTHING)
}

fn input_as_usize(locked_stdout: &mut StdoutLock, options_count: usize) -> TuiResult<usize> {
    
    loop {
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input)?;
        input.pop();
        let chosen_action_index = input.chars().next();
        let chosen_action_index = if input.len() == 1 {
            chosen_action_index
                .map(|char| { 
                    char.to_digit(10)
                        .filter(|number| (*number as usize) <= options_count)
                })
                .flatten()
        } else {
            None
        };
        // We keep trying to take input until valid input is obtained
        match chosen_action_index {
            Some(chosen_action_index) => { return Ok(chosen_action_index as usize - 1); }, // This - 1 converts back to zero-based counting
            None => { 
                writeln!(locked_stdout, "Invalid index. Please try again.")?;
                continue; 
            },
        }
    }
}

fn display_choices(available_actions_for_team: &AvailableChoicesForTeam, locked_stdout: &mut StdoutLock, last_turn_action: bool) -> TuiResult<Nothing> {
    for (index, action) in available_actions_for_team.as_vec().into_iter().enumerate() {
        match action {
            PartiallySpecifiedChoice::Move { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] Use {}", index + 1,  display_text)?; // This + 1 converts to 1-based counting
            },
            PartiallySpecifiedChoice::SwitchOut { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] {}", index + 1, display_text)?; // This + 1 converts to 1-based counting
            },
        }
    }
    let mut next_index = available_actions_for_team.count()+1;
    if last_turn_action {
        writeln!(locked_stdout, "[{}] Repeat last turn actions", next_index)?;
        next_index += 1;
    }
    writeln!(locked_stdout, "[{}] Exit program", next_index)?;
    write_empty_line(locked_stdout)?;

    Ok(NOTHING)
}

enum UIChoice<T> {
    Quit,
    Action(T),
    Repeat(PerTeam<FullySpecifiedChoice>),
}

fn translate_input_to_choices(battle: &BattleState, available_choices_for_team: Team<AvailableChoicesForTeam>, locked_stdout: &mut StdoutLock, last_turn_action: Option<PerTeam<FullySpecifiedChoice>>) -> TuiResult<UIChoice<Team<FullySpecifiedChoice>>> 
{

    let available_actions_count = available_choices_for_team.apply(|actions| actions.count() );
    let chosen_action_index = input_as_usize(locked_stdout, available_actions_count + 2)?;
    
    let is_repeat_selected = chosen_action_index == available_actions_count;
    let mut quit_offset = 0;
    if let Some(last_turn_actions) = last_turn_action {
        if is_repeat_selected {
            return Ok(UIChoice::Repeat(last_turn_actions));
        }
        quit_offset = 1;
    }
    
    let is_quit_selected = chosen_action_index == available_actions_count + quit_offset;
    if is_quit_selected {
        return Ok(UIChoice::Quit);
    }

    let partially_specified_action_for_team = available_choices_for_team.map(|actions| actions[chosen_action_index].unwrap());
    let fully_specified_action_for_team = partially_specified_action_for_team.map(|action| {
        match action {
            PartiallySpecifiedChoice::Move { move_uid, target_uid, .. } => FullySpecifiedChoice::Move { move_uid, target_uid },
            
            PartiallySpecifiedChoice::SwitchOut { switcher_uid, candidate_switchee_uids: possible_switchee_uids, .. } => {
                let switchee_names = possible_switchee_uids.into_iter().flatten().map(|uid| battle.monster(uid).full_name()).enumerate();
                let _ = writeln!(locked_stdout, "Choose a monster to switch with");
                for (index, switchee_name) in switchee_names {
                    let _ = writeln!(locked_stdout, "[{}] {}", index + 1, switchee_name);
                }
                let chosen_switchee_index = input_as_usize(locked_stdout, possible_switchee_uids.iter().flatten().count()).unwrap();
                let chosen_switchee_uid = possible_switchee_uids[chosen_switchee_index].unwrap();
                FullySpecifiedChoice::SwitchOut { switcher_uid, candidate_switchee_uids: chosen_switchee_uid }
            },
        }
    });
    Ok(UIChoice::Action(fully_specified_action_for_team))
}