use std::io::{self, StdoutLock, Write};

use monsim_macros::mon;
use monsim_utils::{Nothing, NOTHING};

use crate::{sim::{AvailableChoices, BattleSimulator, BattleState, FullySpecifiedActionChoice, PartiallySpecifiedActionChoice}, MonsimResult};

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
    let mut actions_chosen_last_turn: Option<Vec<FullySpecifiedActionChoice>> = None;
    'main: loop {
        match turn_stage {
            TurnStage::ChooseActions => {
                
                // We are using one buffer since they will be mixed up when priority sorting anyway.
                let mut chosen_actions_for_turn = Vec::new();
                
                // Check if any of the active monsters have fainted and needs to switched out
                let active_monster_ids = sim.battle.active_monsters().map(|monster| { monster.id });
                for active_monster_id in active_monster_ids {
                    if mon![active_monster_id].is_fainted() {
                        let available_choices_for_active_monster = sim.battle.available_choices_for(mon![active_monster_id]);
                        if let Some(&PartiallySpecifiedActionChoice::SwitchOut { active_monster_id, switchable_benched_monster_ids, activation_order, .. }) = available_choices_for_active_monster.switch_out_choice() {
                            //TODO: what if there are no valid switchees left?
                            
                            let switchable_benched_monster_names = switchable_benched_monster_ids.into_iter().map(|benched_monster_id| mon![benched_monster_id].full_name()).enumerate();
                            writeln!(locked_stdout, "{} fainted! Choose a monster to switch with", mon![active_monster_id].name())?;
                            for (index, monster_name) in switchable_benched_monster_names {
                                writeln!(locked_stdout, "[{}] {}", index + 1, monster_name)?;
                            }
                            let switchable_benched_monster_choice_index = receive_user_input_and_convert_to_choice_index(&mut locked_stdout, switchable_benched_monster_ids.count()).unwrap();
                            let chosen_switchable_benched_monster_id = switchable_benched_monster_ids[switchable_benched_monster_choice_index];

                            // We push the SwitchOut actions to the queue of chosen actions. This is forced onto the user when any of the Monster faints, as a replacement is compulsory.
                            chosen_actions_for_turn.push(FullySpecifiedActionChoice::SwitchOut { active_monster_id, benched_monster_id: chosen_switchable_benched_monster_id, activation_order });
                            actions_chosen_last_turn = None;
                        } else {
                            turn_stage = TurnStage::BattleEnded;
                            continue 'main;
                        }
                    }
                }
                
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
                        let available_action_choices_for_monster = sim.battle.available_choices_for(active_monster);
                        display_choices(&available_action_choices_for_monster, &mut locked_stdout, actions_chosen_last_turn.is_some())?;
                        
                        match receive_user_input_and_convert_to_choice(&sim.battle, available_action_choices_for_monster, &mut locked_stdout, &actions_chosen_last_turn)? {
                            UIChoice::QuitAction => {
                                writeln!(locked_stdout, "Exiting...")?;
                                break 'main;
                            },
                            UIChoice::BattleAction(fully_specified_action_choice) => {
                                chosen_actions_for_turn.push(fully_specified_action_choice);
                            },
                            UIChoice::RepeatLastAction(last_turn_chosen_actions) => {
                                turn_stage = TurnStage::SimulateTurn(last_turn_chosen_actions);
                                continue 'main;
                            },
                        };    
                    }
                }
               
                turn_stage = TurnStage::SimulateTurn(chosen_actions_for_turn.clone());
                actions_chosen_last_turn = Some(chosen_actions_for_turn);
            },
            TurnStage::SimulateTurn(chosen_actions_for_turn) => {

                sim.battle.message_log.set_last_turn_cursor_to_log_length();
                
                sim.simulate_turn(chosen_actions_for_turn)?;

                // Show the message log
                sim.battle.message_log.show_last_turn_messages();
                
                if sim.battle.is_finished() {
                    turn_stage = TurnStage::BattleEnded;
                } else {
                    turn_stage = TurnStage::ChooseActions;
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

fn display_choices(available_actions_for_team: &AvailableChoices, locked_stdout: &mut StdoutLock, last_turn_action: bool) -> MonsimResult<Nothing> {
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
    let mut next_index = available_actions_for_team.count() + 1;
    if last_turn_action {
        writeln!(locked_stdout, "[{}] Repeat last turn actions", next_index)?;
        next_index += 1;
    }
    writeln!(locked_stdout, "[{}] Exit program", next_index)?;
    write_empty_line(locked_stdout)?;

    Ok(NOTHING)
}

enum UIChoice {
    QuitAction,
    BattleAction(FullySpecifiedActionChoice),
    RepeatLastAction(Vec<FullySpecifiedActionChoice>),
}

fn receive_user_input_and_convert_to_choice(
    battle: &BattleState, 
    available_choices_for_monster: AvailableChoices, 
    locked_stdout: &mut StdoutLock, 
    actions_chosen_last_turn: &Option<Vec<FullySpecifiedActionChoice>>
) -> MonsimResult<UIChoice> {

    let available_actions_count = available_choices_for_monster.count();
    let choice_index = receive_user_input_and_convert_to_choice_index(locked_stdout, available_actions_count + 2)?;
    
    let is_repeat_selected = choice_index == available_actions_count;
    let mut quit_offset = 0;
    if is_repeat_selected && actions_chosen_last_turn.is_some() {
        if let Some(actions_chosen_last_turn) = actions_chosen_last_turn {
            return Ok(UIChoice::RepeatLastAction(actions_chosen_last_turn.clone()));
        }
        quit_offset = 1;
    }
    
    let is_quit_selected = choice_index == available_actions_count + quit_offset;
    if is_quit_selected {
        return Ok(UIChoice::QuitAction);
    }

    let partially_specified_action_for_monster = available_choices_for_monster[choice_index];
    let fully_specified_action_for_monster = match partially_specified_action_for_monster {
        PartiallySpecifiedActionChoice::Move { move_id, possible_target_positions, activation_order, .. } => {
            // Target position prompt
            let target_position_names = possible_target_positions.iter()
                .map(|position| {
                    format!["{} ({:?})", battle.monster_at_position(*position).expect("This is precomputed.").full_name(), position]
                })
                .enumerate();
            for (index, position_name) in target_position_names {
                let _ = writeln!(locked_stdout, "[{}] {}", index + 1, position_name);
            }
            let chosen_target_position_index = receive_user_input_and_convert_to_choice_index(locked_stdout, possible_target_positions.count()).unwrap();
            let target_position = possible_target_positions[chosen_target_position_index];
            FullySpecifiedActionChoice::Move { move_id, target_position, activation_order }
        },
        PartiallySpecifiedActionChoice::SwitchOut { active_monster_id, switchable_benched_monster_ids, activation_order, .. } => {
            // Switchee prompt
            let switchable_benched_monster_names = switchable_benched_monster_ids.into_iter().map(|id| battle.monster(id).full_name()).enumerate();
            let _ = writeln!(locked_stdout, "Choose a benched monster to switch in");
            for (index, switchee_name) in switchable_benched_monster_names {
                let _ = writeln!(locked_stdout, "[{}] {}", index + 1, switchee_name);
            }
            let chosen_switchable_benched_monster_choice_index = receive_user_input_and_convert_to_choice_index(locked_stdout, switchable_benched_monster_ids.count()).unwrap();
            let benched_monster_id = switchable_benched_monster_ids[chosen_switchable_benched_monster_choice_index];
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