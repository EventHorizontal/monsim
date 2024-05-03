use std::io::{self, StdoutLock, Write};

use monsim_macros::mon;
use monsim_utils::{Nothing, NOTHING};

use crate::{sim::{AvailableChoices, BattleSimulator, BattleState, FullySpecifiedActionChoice, PartiallySpecifiedActionChoice}, BattleFormat, FieldPosition, MonsimResult, MonsterID};

enum TurnStage {
    ChooseActions,
    SimulateTurn(Vec<FullySpecifiedActionChoice>),
    BattleEnded,
}

pub fn run(battle: BattleState) -> MonsimResult<Nothing> {

    let mut sim = BattleSimulator::init(battle);

    let mut turn_stage = TurnStage::ChooseActions;

    // We lock stdout so that we don't have to acquire the lock every time with `println!`
    let mut locked_stdout = io::stdout().lock();
    
    'main: loop {
        let mut monsters_already_chosen_for_switching = Vec::new();
        match turn_stage {
            TurnStage::ChooseActions => {
                
                // We are using one buffer since they will be mixed up when priority sorting anyway.
                let mut chosen_actions_for_turn = Vec::new();
                
                write_empty_line(&mut locked_stdout)?;
                writeln!(locked_stdout, "Current Battle Status:")?;
                write_empty_line(&mut locked_stdout)?;
                
                for active_monsters_per_team in sim.battle.active_monsters_by_team() {
                    for active_monster in active_monsters_per_team {
                        writeln!(locked_stdout, "{} Active Monster: {}", active_monster.id.team_id, active_monster.status_string())?;
                    }
                }
                
                writeln!(locked_stdout, "Ally Team:")?;
                writeln!(locked_stdout, "{}", sim.battle.ally_team().team_status_string())?;
                writeln!(locked_stdout, "Opponent Team:")?;
                writeln!(locked_stdout, "{}", sim.battle.opponent_team().team_status_string())?;
                
                for active_monsters_per_team in sim.battle.active_monsters_by_team() {
                    for active_monster in active_monsters_per_team {
                        writeln!(locked_stdout, "Choose an Action for {}", active_monster.full_name())?;
                        write_empty_line(&mut locked_stdout)?;
                        let available_action_choices_for_monster = sim.battle.available_choices_for(active_monster, &monsters_already_chosen_for_switching);
                        display_choices(&available_action_choices_for_monster, &mut locked_stdout)?;
                        
                        match receive_user_input_and_convert_to_choice(&sim.battle, available_action_choices_for_monster, &mut locked_stdout, &mut monsters_already_chosen_for_switching)? {
                            UIChoice::QuitAction => {
                                writeln!(locked_stdout, "Exiting...")?;
                                break 'main;
                            },
                            UIChoice::BattleAction(fully_specified_action_choice) => {
                                chosen_actions_for_turn.push(fully_specified_action_choice);
                            },
                        };    
                    }
                }
               
                turn_stage = TurnStage::SimulateTurn(chosen_actions_for_turn.clone());
            },
            TurnStage::SimulateTurn(chosen_actions_for_turn) => {

                sim.battle.message_log.snap_last_turn_cursor_to_end();
                
                sim.simulate_turn(chosen_actions_for_turn)?;
                
                // Show the message log
                sim.battle.message_log.show_last_turn_messages();
                
                if sim.battle.is_finished() {
                    turn_stage = TurnStage::BattleEnded;
                } else {
                    turn_stage = TurnStage::ChooseActions;
                }

                // Check if any board positions are empty and replace them with Monsters if there are any possible.
                let empty_field_positions = valid_positions_in(sim.battle.format())
                    .into_iter()
                    .filter(|position| { sim.battle.monster_at_position(*position).is_none() })
                    .collect::<Vec<_>>();

                monsters_already_chosen_for_switching.clear();

                // Replace each monster that we can.
                for field_position in empty_field_positions {
                    let team_id = field_position.side();
                    /*
                    TEMP: The queues is empty at the moment and we don't refill it because we directly replace the monster instead of waiting
                    for it to be naturally activated in order, so no use holding the MonsterID as the switch is already carried out and will
                    be taken into account in the next iteration of the loop. This is temporary because the real solution is to integrate the
                    choice selection into the engine itself, so that we can deal with these battle logic specific issues in the engine.
                    */
                    let switchable_benched_monster_ids = sim.battle.switchable_benched_monster_ids(team_id, &monsters_already_chosen_for_switching);
                    if switchable_benched_monster_ids.is_empty() {
                        sim.push_message(format!["{} is empty but {} is out of switchable Monsters!", field_position, team_id]);
                        continue;
                    }
                    let switchable_benched_monster_names = switchable_benched_monster_ids.into_iter().map(|benched_monster_id| mon![benched_monster_id].full_name()).enumerate();
                    writeln!(locked_stdout, "Choose a monster to switch in to {}", field_position)?;
                    for (index, monster_name) in switchable_benched_monster_names {
                        writeln!(locked_stdout, "[{}] {}", index + 1, monster_name)?;
                    }
                    let switchable_benched_monster_choice_index = receive_user_input_and_convert_to_choice_index(&mut locked_stdout, switchable_benched_monster_ids.count()).unwrap();
                    let chosen_switchable_benched_monster_id = switchable_benched_monster_ids[switchable_benched_monster_choice_index];

                    // We directly replace the empty position with the chosen Monster
                    crate::sim::effects::ReplaceFaintedMonster(&mut sim, (chosen_switchable_benched_monster_id, field_position));
                    sim.battle.message_log.show_last_message();
                } 
            },
            TurnStage::BattleEnded => {
                println!("The simulator terminated successfully.");
                println!("Exitting...");
                break 'main;
            },
        }
    }
    Ok(NOTHING)
}

fn valid_positions_in(battle_format: BattleFormat) -> Vec<FieldPosition> {
    match battle_format {
        BattleFormat::Single => {
            vec![FieldPosition::AllySideCentre, FieldPosition::OpponentSideCentre]
        
        },
        BattleFormat::Double => {
            vec![FieldPosition::AllySideCentre, FieldPosition::AllySideRight, FieldPosition::OpponentSideCentre, FieldPosition::OpponentSideRight]
        },
        BattleFormat::Triple => {
            vec![FieldPosition::AllySideLeft, FieldPosition::AllySideCentre, FieldPosition::AllySideRight, FieldPosition::OpponentSideLeft, FieldPosition::OpponentSideCentre, FieldPosition::OpponentSideRight]
        },
    }
}

fn display_choices(available_actions_for_team: &AvailableChoices, locked_stdout: &mut StdoutLock) -> MonsimResult<Nothing> {
    for (index, action) in available_actions_for_team.choices().into_iter().enumerate() {
        match action {
            PartiallySpecifiedActionChoice::Move { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] Use {}", index + 1,  display_text)?; // This + 1 converts to 1-based counting for display  
            },
            PartiallySpecifiedActionChoice::SwitchOut { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] {}", index + 1, display_text)?; // This + 1 converts to 1-based counting for display
            },
        }
    }
    let next_index = available_actions_for_team.count() + 1;
    writeln!(locked_stdout, "[{}] Exit monsim", next_index)?;
    write_empty_line(locked_stdout)?;

    Ok(NOTHING)
}

enum UIChoice {
    QuitAction,
    BattleAction(FullySpecifiedActionChoice),
}

fn receive_user_input_and_convert_to_choice(
    battle: &BattleState, 
    available_choices_for_monster: AvailableChoices, 
    locked_stdout: &mut StdoutLock, 
    monsters_already_chosen_for_switching: &mut Vec<MonsterID>,
) -> MonsimResult<UIChoice> {

    let available_actions_count = available_choices_for_monster.count();
    let choice_index = receive_user_input_and_convert_to_choice_index(locked_stdout, available_actions_count + 2)?;
    
    let is_quit_selected = choice_index == available_actions_count;
    if is_quit_selected {
        return Ok(UIChoice::QuitAction);
    }

    let partially_specified_action_for_monster = available_choices_for_monster[choice_index];
    let fully_specified_action_for_monster = match partially_specified_action_for_monster {
        PartiallySpecifiedActionChoice::Move { move_id, possible_target_positions, activation_order, .. } => {
            // Target position prompt
            write_empty_line(locked_stdout)?;
            let target_position_names = possible_target_positions.iter()
                .map(|position| {
                    format!["{} ({:?})", battle.monster_at_position(*position).expect("This is precomputed.").full_name(), position]
                })
                .enumerate();
            for (index, position_name) in target_position_names {
                writeln!(locked_stdout, "[{}] {}", index + 1, position_name)?;
            }
            write_empty_line(locked_stdout)?;
            let chosen_target_position_index = receive_user_input_and_convert_to_choice_index(locked_stdout, possible_target_positions.count()).unwrap();
            let target_position = possible_target_positions[chosen_target_position_index];
            FullySpecifiedActionChoice::Move { move_id, target_position, activation_order }
        },
        PartiallySpecifiedActionChoice::SwitchOut { active_monster_id, switchable_benched_monster_ids, activation_order, .. } => {
            // Switchee prompt
            let switchable_benched_monster_names = switchable_benched_monster_ids.into_iter().map(|id| battle.monster(id).full_name()).enumerate();
            let _ = writeln!(locked_stdout, "Choose a benched monster to switch in");
            for (index, switchee_name) in switchable_benched_monster_names {
                writeln!(locked_stdout, "[{}] {}", index + 1, switchee_name)?;
            }
            write_empty_line(locked_stdout)?;
            let chosen_switchable_benched_monster_choice_index = receive_user_input_and_convert_to_choice_index(locked_stdout, switchable_benched_monster_ids.count()).unwrap();
            let benched_monster_id = switchable_benched_monster_ids[chosen_switchable_benched_monster_choice_index];
            // TEMP: This is another component of what I wrote above where we have to keep track of who's already been selected for switching out
            // because this whole process needs to occur inside the engine but the infrastructure for that doesn't exist yet.
            monsters_already_chosen_for_switching.push(benched_monster_id);
            FullySpecifiedActionChoice::SwitchOut { active_monster_id, benched_monster_id, activation_order }
        },
    };
    Ok(UIChoice::BattleAction(fully_specified_action_for_monster))
}

fn receive_user_input_and_convert_to_choice_index(locked_stdout: &mut StdoutLock, total_action_choices: usize) -> MonsimResult<usize> {
    loop { // We keep asking until the input is valid.
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input)?;
        // INFO: Windows uses \r\n. Linux uses only \n, so we use `trim()` which accounts for both.
        let input = input.trim();
        let chosen_action_index = input.chars().next();
        if input.len() == 1 {
            let maybe_action_choice_index = chosen_action_index.map(|char| { char.to_digit(10) }).flatten();
            if let Some(action_choice_index) = maybe_action_choice_index {
                if 0 < action_choice_index && action_choice_index <= total_action_choices as u32 {
                    return Ok(action_choice_index as usize - 1); // The -1 converts back to zero based counting 
                }
            }
        };
        writeln!(locked_stdout, "Invalid index. Please try again.")?;
    }
}

fn write_empty_line(locked_stdout: &mut StdoutLock<'_>) -> MonsimResult<Nothing> {
    writeln!(locked_stdout, "")?;
    Ok(NOTHING)
}