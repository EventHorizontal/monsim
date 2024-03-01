use std::io::{self, StdoutLock, Write};

use monsim_utils::{Nothing, NOTHING, TeamAffiliation};

use crate::{app::AppResult, sim::{AvailableActions, AvailableActionsForTeam, Battle, BattleSimulator, ChosenActionsForTurn, FullySpecifiedAction, PartiallySpecifiedAction, PerTeam, TeamID}};

enum TurnStage {
    ChooseAction(AvailableActions),
    ChooseSwitchPartner(TeamID),
    Simulate(ChosenActionsForTurn),
    BattleEnded,
}

pub fn run(mut battle: Battle) -> AppResult<Nothing> {
    let mut turn_stage = TurnStage::ChooseAction(battle.available_actions());

    let mut locked_stdout = io::stdout().lock();
    'main: loop {
        match turn_stage {
            TurnStage::ChooseAction(available_actions) => {
                
                let (ally_team_available_actions, opponent_team_available_actions) = available_actions.unwrap();
                
                // Ally choices
                writeln!(locked_stdout, "Choose an Action for the Ally Team")?;
                writeln!(locked_stdout, "")?;
                display_choices(&ally_team_available_actions, &mut locked_stdout)?;
                let chosen_action_index = input()?;
                let quit_choice_index = ally_team_available_actions.count();
                if chosen_action_index == quit_choice_index {
                    break 'main;
                }
                let ally_team_chosen_action = fully_specify_team_action(ally_team_available_actions, chosen_action_index)?;

                // Opponent choices
                writeln!(locked_stdout, "Choose an Action for the Opponent Team")?;
                writeln!(locked_stdout, "")?;
                display_choices(&opponent_team_available_actions, &mut locked_stdout)?;
                let chosen_action_index = input()?;
                let quit_choice_index = ally_team_available_actions.count();
                if chosen_action_index == quit_choice_index {
                    break 'main;
                }
                let opponent_team_chosen_action = fully_specify_team_action(opponent_team_available_actions, chosen_action_index)?;

                let chosen_actions = PerTeam::new(ally_team_chosen_action, opponent_team_chosen_action);

                turn_stage = TurnStage::Simulate(chosen_actions);
            },
            TurnStage::ChooseSwitchPartner(team_id) => {
                todo!()
            },
            TurnStage::Simulate(chosen_actions_for_turn) => {
                
                BattleSimulator::simulate_turn(&mut battle, chosen_actions_for_turn)?;

                // Show the message log
                battle.message_log.show_last_turn_messages();
                
                if battle.is_finished {
                    turn_stage = TurnStage::BattleEnded;
                } else {
                    turn_stage = TurnStage::ChooseAction(battle.available_actions());
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

fn input() -> AppResult<usize> {
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input)?;
    let chosen_action_index = input.chars()
        .next()
        .map(|char| { char.to_digit(10) })
        .flatten()
        .expect("Currently the program expects the user to supply valid input.") as usize - 1;
    Ok(chosen_action_index)
}

fn fully_specify_team_action<T>(available_actions_for_team: T, chosen_action_index: usize) -> AppResult<T::R<FullySpecifiedAction>> 
    where T: Copy + TeamAffiliation<AvailableActionsForTeam> 
{
    let opponent_team_chosen_action = available_actions_for_team
        .map(|actions| { 
            let action = actions[chosen_action_index].expect("The indices were generated from the same AvailableActions"); 
            fully_specify_action(action)
        }
    );
    println!();
    Ok(opponent_team_chosen_action)
}

fn fully_specify_action(action: PartiallySpecifiedAction) -> FullySpecifiedAction {
    match action {
        PartiallySpecifiedAction::Move { move_uid, target_uid, .. } => FullySpecifiedAction::Move { move_uid, target_uid },
        PartiallySpecifiedAction::SwitchOut { .. } => todo!(),
    }
}

fn display_choices(available_actions_for_team: &AvailableActionsForTeam, locked_stdout: &mut StdoutLock) -> AppResult<Nothing> {
    for (index, action) in available_actions_for_team.as_vec().into_iter().enumerate() {
        match action {
            PartiallySpecifiedAction::Move { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] Use {}", index+1,  display_text)?; 
            },
            PartiallySpecifiedAction::SwitchOut { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] {}", index+1, display_text)?; 
            },
        }
    }
    let next_index = available_actions_for_team.count()+1;
    writeln!(locked_stdout, "[{}] Exit program", next_index)?;

    Ok(NOTHING)
}