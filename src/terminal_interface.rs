use std::io::{self, StdoutLock, Write};

use monsim_utils::{Nothing, Team, NOTHING};

use crate::{app::AppResult, sim::{AvailableActions, AvailableActionsForTeam, Battle, BattleSimulator, ChosenActionsForTurn, FullySpecifiedAction, PartiallySpecifiedAction, PerTeam}};

enum TurnStage {
    ChooseActions(AvailableActions),
    SimulateTurn(ChosenActionsForTurn),
    BattleEnded,
}

pub fn run(mut battle: Battle) -> AppResult<Nothing> {
    let mut turn_stage = TurnStage::ChooseActions(battle.available_actions());

    // We lock stdout so that we don't have to acquire the lock every time with `println!`
    let mut locked_stdout = io::stdout().lock();
    'main: loop {
        match turn_stage {
            TurnStage::ChooseActions(available_actions) => {
                
                let (ally_team_available_actions, opponent_team_available_actions) = available_actions.unwrap();
                
                write_empty_line(&mut locked_stdout)?;
                
                // Ally Team choices
                writeln!(locked_stdout, "Choose an Action for the Ally Team")?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&ally_team_available_actions, &mut locked_stdout)?;
                
                let ally_team_fully_specified_action = match translate_input_to_choices(&battle, Team::ally(ally_team_available_actions), &mut locked_stdout)? {
                    Choice::Quit => {
                        writeln!(locked_stdout, "Exiting...")?;
                        break 'main
                    },
                    Choice::Action(fully_specified_action) => fully_specified_action.expect_ally(),
                };

                // Opponent choices
                writeln!(locked_stdout, "Choose an Action for the Opponent Team")?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&opponent_team_available_actions, &mut locked_stdout)?;
                
                let opponent_team_fully_specified_action = match translate_input_to_choices(&battle, Team::opponent(opponent_team_available_actions), &mut locked_stdout)? {
                    Choice::Quit => {
                        writeln!(locked_stdout, "Exiting...")?;
                        break 'main
                    },
                    Choice::Action(fully_specified_action) => fully_specified_action.expect_opponent(),
                };

                // Package both team's choices up
                let chosen_actions = PerTeam::new(ally_team_fully_specified_action, opponent_team_fully_specified_action);

                turn_stage = TurnStage::SimulateTurn(chosen_actions);
            },
            TurnStage::SimulateTurn(chosen_actions_for_turn) => {
                
                BattleSimulator::simulate_turn(&mut battle, chosen_actions_for_turn)?;

                // Show the message log
                battle.message_log.show_last_turn_messages();
                
                if battle.is_finished {
                    turn_stage = TurnStage::BattleEnded;
                } else {
                    turn_stage = TurnStage::ChooseActions(battle.available_actions());
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

fn write_empty_line(locked_stdout: &mut StdoutLock<'_>) -> AppResult<Nothing> {
    writeln!(locked_stdout, "")?;
    Ok(NOTHING)
}

fn input_as_usize(locked_stdout: &mut StdoutLock, options_count: usize) -> AppResult<usize> {
    
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

fn display_choices(available_actions_for_team: &AvailableActionsForTeam, locked_stdout: &mut StdoutLock) -> AppResult<Nothing> {
    for (index, action) in available_actions_for_team.as_vec().into_iter().enumerate() {
        match action {
            PartiallySpecifiedAction::Move { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] Use {}", index + 1,  display_text)?; // This + 1 converts to 1-based counting
            },
            PartiallySpecifiedAction::SwitchOut { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] {}", index + 1, display_text)?; // This + 1 converts to 1-based counting
            },
        }
    }
    let next_index = available_actions_for_team.count()+1;
    writeln!(locked_stdout, "[{}] Exit program", next_index)?;
    write_empty_line(locked_stdout)?;

    Ok(NOTHING)
}

enum Choice<T> {
    Quit,
    Action(T)
}

fn translate_input_to_choices(battle: &Battle, available_actions_for_team: Team<AvailableActionsForTeam>, locked_stdout: &mut StdoutLock) -> AppResult<Choice<Team<FullySpecifiedAction>>> 
{

    let available_actions_count = available_actions_for_team.apply(|actions| actions.count() );
    let chosen_action_index = input_as_usize(locked_stdout, available_actions_count + 1)?;

    let is_quit_selected = chosen_action_index == available_actions_count;
    if is_quit_selected {
        return Ok(Choice::Quit);
    }

    let partially_specified_action_for_team = available_actions_for_team.map(|actions| actions[chosen_action_index].unwrap());
    let fully_specified_action_for_team = partially_specified_action_for_team.map(|action| {
        match action {
            PartiallySpecifiedAction::Move { move_uid, target_uid, .. } => FullySpecifiedAction::Move { move_uid, target_uid },
            
            PartiallySpecifiedAction::SwitchOut { switcher_uid, possible_switchee_uids, .. } => {
                let switchee_names = possible_switchee_uids.into_iter().flatten().map(|uid| battle.monster(uid).full_name()).enumerate();
                let _ = writeln!(locked_stdout, "Choose a monster to switch with");
                for (index, switchee_name) in switchee_names {
                    let _ = writeln!(locked_stdout, "[{}] {}", index + 1, switchee_name);
                }
                let chosen_switchee_index = input_as_usize(locked_stdout, possible_switchee_uids.iter().flatten().count()).unwrap();
                let chosen_switchee_uid = possible_switchee_uids[chosen_switchee_index].unwrap();
                FullySpecifiedAction::SwitchOut { switcher_uid, switchee_uid: chosen_switchee_uid }
            },
        }
    });
    Ok(Choice::Action(fully_specified_action_for_team))
}