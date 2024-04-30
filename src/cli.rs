use std::io::{self, StdoutLock, Write};

use monsim_utils::{Nothing, TeamAffl, NOTHING};

use crate::{sim::{AvailableChoicesForTeam, BattleSimulator, BattleState, FullySpecifiedChoice, PartiallySpecifiedChoice, PerTeam}, MonsimResult, PerformSwitchOut};

enum TurnStage {
    ChooseActions(PerTeam<AvailableChoicesForTeam>),
    SimulateTurn(PerTeam<FullySpecifiedChoice>),
    BattleEnded,
}

pub fn run(battle: BattleState) -> MonsimResult<Nothing> {

    let mut sim = BattleSimulator::init(battle);

    let mut turn_stage = TurnStage::ChooseActions(sim.battle.available_choices());

    // We lock stdout so that we don't have to acquire the lock every time with `println!`
    let mut locked_stdout = io::stdout().lock();
    let mut last_turn_chosen_actions: Option<PerTeam<FullySpecifiedChoice>> = None;
    'main: loop {
        match turn_stage {
            TurnStage::ChooseActions(available_choices) => {

                // Check if any of the active monsters has fainted and needs to switched out
                for active_monster_id in sim.battle.active_monsters().map_consume(|monster| { monster.id }) {
                    let available_choices_for_team = &available_choices[active_monster_id.team_id];
                    if sim.battle.monster(active_monster_id).is_fainted() {
                        if let Some(&PartiallySpecifiedChoice::SwitchOut { active_monster_id, switchable_benched_monster_ids, .. }) = available_choices_for_team.switch_out_choice() {
                            let switchable_benched_monster_names = switchable_benched_monster_ids.into_iter().map(|id| sim.battle.monster(id).full_name()).enumerate();
                            writeln!(locked_stdout, "{} fainted! Choose a monster to switch with", sim.battle .monster(active_monster_id).name())?;
                            for (index, monster_name) in switchable_benched_monster_names {
                                writeln!(locked_stdout, "[{}] {}", index + 1, monster_name)?;
                            }
                            let switchable_benched_monster_choice_index = input_to_choice_index(&mut locked_stdout, switchable_benched_monster_ids.count()).unwrap();
                            let chosen_switchable_benched_monster_id = switchable_benched_monster_ids[switchable_benched_monster_choice_index];
                            PerformSwitchOut(&mut sim, crate::SwitchContext { active_monster_id: active_monster_id, benched_monster_id: chosen_switchable_benched_monster_id });
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

                writeln!(locked_stdout, "Ally Active Monster: {}", sim.battle.ally_team().active_monster().status_string())?;
                writeln!(locked_stdout, "Opponent Active Monster {}", sim.battle.opponent_team().active_monster().status_string())?;
                writeln!(locked_stdout, "Ally Team:")?;
                writeln!(locked_stdout, "{}", sim.battle.ally_team().team_status_string())?;
                writeln!(locked_stdout, "Opponent Team:")?;
                writeln!(locked_stdout, "{}", sim.battle.opponent_team().team_status_string())?;
                
                // Ally Team choices
                writeln!(locked_stdout, "Choose an Action for {}", sim.battle.ally_team().active_monster().full_name())?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&ally_team_available_choices, &mut locked_stdout, last_turn_chosen_actions.is_some())?;
                
                let ally_team_fully_specified_action = match translate_input_to_choices(&sim.battle, TeamAffl::ally(ally_team_available_choices), &mut locked_stdout, last_turn_chosen_actions)? {
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
                writeln!(locked_stdout, "Choose an Action for {}", sim.battle.opponent_team().active_monster().full_name())?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&opponent_team_available_choices, &mut locked_stdout, last_turn_chosen_actions.is_some())?;
                
                let opponent_team_fully_specified_action = match translate_input_to_choices(&sim.battle, TeamAffl::opponent(opponent_team_available_choices), &mut locked_stdout, last_turn_chosen_actions)? {
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

                sim.battle.message_log.set_last_turn_cursor_to_log_length();
                
                sim.simulate_turn(chosen_actions_for_turn)?;

                // Show the message log
                sim.battle.message_log.show_last_turn_messages();
                
                if sim.battle.is_finished() {
                    turn_stage = TurnStage::BattleEnded;
                } else {
                    turn_stage = TurnStage::ChooseActions(sim.battle.available_choices());
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

fn write_empty_line(locked_stdout: &mut StdoutLock<'_>) -> MonsimResult<Nothing> {
    writeln!(locked_stdout, "")?;
    Ok(NOTHING)
}

fn display_choices(available_actions_for_team: &AvailableChoicesForTeam, locked_stdout: &mut StdoutLock, last_turn_action: bool) -> MonsimResult<Nothing> {
    for (index, action) in available_actions_for_team.choices().into_iter().enumerate() {
        match action {
            PartiallySpecifiedChoice::Move { display_text, .. } => { 
                writeln!(locked_stdout, "[{}] Use {}", index + 1,  display_text)?; // This + 1 converts to 1-based counting for display  
            },
            PartiallySpecifiedChoice::SwitchOut { display_text, .. } => { 
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

enum UIChoice<T> {
    Quit,
    Action(T),
    Repeat(PerTeam<FullySpecifiedChoice>),
}

fn translate_input_to_choices(battle: &BattleState, available_choices_for_team: TeamAffl<AvailableChoicesForTeam>, locked_stdout: &mut StdoutLock, last_turn_action: Option<PerTeam<FullySpecifiedChoice>>) -> MonsimResult<UIChoice<TeamAffl<FullySpecifiedChoice>>> 
{

    let available_actions_count = available_choices_for_team.apply(|actions| actions.count() );
    let choice_index = input_to_choice_index(locked_stdout, available_actions_count + 2)?;
    
    let is_repeat_selected = choice_index == available_actions_count;
    let mut quit_offset = 0;
    if let Some(last_turn_actions) = last_turn_action {
        if is_repeat_selected {
            return Ok(UIChoice::Repeat(last_turn_actions));
        }
        quit_offset = 1;
    }
    
    let is_quit_selected = choice_index == available_actions_count + quit_offset;
    if is_quit_selected {
        return Ok(UIChoice::Quit);
    }

    let partially_specified_action_for_team = available_choices_for_team.map(|actions| actions[choice_index]);
    let fully_specified_action_for_team = partially_specified_action_for_team.map(|action| {
        match action {
            PartiallySpecifiedChoice::Move { move_id, target_position, activation_order, .. } => FullySpecifiedChoice::Move { move_id, target_position, activation_order },
            
            PartiallySpecifiedChoice::SwitchOut { active_monster_id, switchable_benched_monster_ids, activation_order, .. } => {
                let switchable_benched_monster_names = switchable_benched_monster_ids.into_iter().map(|id| battle.monster(id).full_name()).enumerate();
                let _ = writeln!(locked_stdout, "Choose a benched monster to switch in");
                for (index, switchee_name) in switchable_benched_monster_names {
                    let _ = writeln!(locked_stdout, "[{}] {}", index + 1, switchee_name);
                }
                let chosen_switchable_benched_monster_choice_index = input_to_choice_index(locked_stdout, switchable_benched_monster_ids.count()).unwrap();
                let benched_monster_id = switchable_benched_monster_ids[chosen_switchable_benched_monster_choice_index];
                FullySpecifiedChoice::SwitchOut { active_monster_id, benched_monster_id, activation_order }
            },
        }
    });
    Ok(UIChoice::Action(fully_specified_action_for_team))
}

fn input_to_choice_index(locked_stdout: &mut StdoutLock, total_choices: usize) -> MonsimResult<usize> {
    loop { // We keep asking until the input is valid.
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input)?;
        let input = &input[..input.len()-2]; // I think there's a \cr\n at the end or something. TODO: Investigate later.
        let chosen_action_index = input.chars().next();
        if input.len() == 1 {
            let chosen_action_index = chosen_action_index.map(|char| { char.to_digit(10) }).flatten();
            if let Some(chosen_action_index) = chosen_action_index {
                if 0 < chosen_action_index && chosen_action_index <= total_choices as u32 {
                    return Ok(chosen_action_index as usize - 1); // The -1 converts back to zero based counting 
                }
            }
        };
        writeln!(locked_stdout, "Invalid index. Please try again.")?;
    }
}