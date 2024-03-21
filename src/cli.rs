use std::io::{self, StdoutLock, Write};

use monsim_utils::{Nothing, TeamAffil, NOTHING};

use crate::{sim::{AvailableChoicesForTeam, Battle, BattleSimulator, FullySpecifiedChoice, PartiallySpecifiedChoice, PerTeam}, MonsimResult};

enum TurnStage<'a> {
    ChooseActions(PerTeam<AvailableChoicesForTeam<'a>>),
    SimulateTurn(PerTeam<FullySpecifiedChoice<'a>>),
    BattleEnded,
}

pub fn run(mut battle: Battle) -> MonsimResult<Nothing> {
    let mut turn_stage = TurnStage::ChooseActions(battle.available_choices());

    // We lock stdout so that we don't have to acquire the lock every time with `println!`
    let mut locked_stdout = io::stdout().lock();
    let mut last_turn_chosen_actions: Option<PerTeam<FullySpecifiedChoice>> = None;
    'main: loop {
        match turn_stage {
            TurnStage::ChooseActions(available_choices) => {

                // Check if any of the active monsters has fainted and needs to switched out
                for active_monster in battle.active_monsters() {
                    if active_monster.is_fainted.get() {
                        if let Some(PartiallySpecifiedChoice::SwitchOut { active_monster, switchable_benched_monsters, .. }) = available_choices[active_monster.team()].switch_out_choice() {
                            let switchable_benched_monster_names = switchable_benched_monsters.iter().map(|monster| monster.full_name()).enumerate();
                            writeln!(locked_stdout, "{} fainted! Choose a monster to switch with", active_monster.name())?;
                            for (index, switchee_name) in switchable_benched_monster_names {
                                writeln!(locked_stdout, "[{}] {}", index + 1, switchee_name)?;
                            }
                            let chosen_benched_monster_index = input_as_choice_index(&mut locked_stdout, switchable_benched_monsters.iter().count())?;
                            let chosen_switchee_uid = switchable_benched_monsters[chosen_benched_monster_index];
                            BattleSimulator::switch_out_between_turns(&mut battle, active_monster, chosen_switchee_uid)?;
                            last_turn_chosen_actions = None;
                        } else {
                            turn_stage = TurnStage::BattleEnded;
                            continue 'main;
                        }
                    }
                }

                let (ally_team_available_choices, opponent_team_available_choices) = available_choices.unwrap();
                let (ally_team_active_monster, opponent_team_active_monster) = battle.active_monsters().unwrap();
                
                // Display current battle status
                write_empty_line(&mut locked_stdout)?;
                writeln!(locked_stdout, "Current Battle Status:")?;
                write_empty_line(&mut locked_stdout)?;

                writeln!(locked_stdout, "Ally Active Monster: {}", ally_team_active_monster.status_string())?;
                writeln!(locked_stdout, "Opponent Active Monster {}", opponent_team_active_monster.status_string())?;
                writeln!(locked_stdout, "Ally Team:")?;
                writeln!(locked_stdout, "{}", battle.ally_team().status_string())?;
                writeln!(locked_stdout, "Opponent Team:")?;
                writeln!(locked_stdout, "{}", battle.opponent_team().status_string())?;
                
                // Display ally team choices
                writeln!(locked_stdout, "Choose an Action for {}", ally_team_active_monster.full_name())?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&ally_team_available_choices, &mut locked_stdout, last_turn_chosen_actions.is_some())?;
                
                let ally_team_fully_specified_action = match translate_input_to_choices(TeamAffil::ally(ally_team_available_choices), &mut locked_stdout, last_turn_chosen_actions)? {
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

                // Display opponent team choices
                writeln!(locked_stdout, "Choose an Action for {}", opponent_team_active_monster.full_name())?;
                write_empty_line(&mut locked_stdout)?;
                display_choices(&opponent_team_available_choices, &mut locked_stdout, last_turn_chosen_actions.is_some())?;
                
                let opponent_team_fully_specified_action = match translate_input_to_choices(TeamAffil::opponent(opponent_team_available_choices), &mut locked_stdout, last_turn_chosen_actions)? {
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

fn write_empty_line(locked_stdout: &mut StdoutLock<'_>) -> MonsimResult<Nothing> {
    writeln!(locked_stdout, "")?;
    Ok(NOTHING)
}

fn input_as_choice_index(locked_stdout: &mut StdoutLock, options_count: usize) -> MonsimResult<usize> {
    
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

fn display_choices(available_actions_for_team: &AvailableChoicesForTeam, locked_stdout: &mut StdoutLock, last_turn_action: bool) -> MonsimResult<Nothing> {
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

enum UIChoice<'a, T> {
    Quit,
    Action(T),
    Repeat(PerTeam<FullySpecifiedChoice<'a>>),
}

fn translate_input_to_choices<'a>(
    available_choices_for_team: TeamAffil<AvailableChoicesForTeam<'a>>, 
    locked_stdout: &mut StdoutLock, 
    last_turn_action: Option<PerTeam<FullySpecifiedChoice<'a>>>
) -> MonsimResult<UIChoice<'a, TeamAffil<FullySpecifiedChoice<'a>>>> 
{

    let available_actions_count = available_choices_for_team.apply(|actions| actions.count() );
    let chosen_action_index = input_as_choice_index(locked_stdout, available_actions_count + 2)?;
    
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

    let partially_specified_action_for_team = available_choices_for_team.map(|actions| actions.get_by_index(chosen_action_index));
    let fully_specified_action_for_team = partially_specified_action_for_team.map(|action| {
        match action {
            PartiallySpecifiedChoice::Move { attacker, move_, target, .. } => FullySpecifiedChoice::Move { attacker, move_, target },
            
            PartiallySpecifiedChoice::SwitchOut { active_monster, switchable_benched_monsters, .. } => {
                let switchable_benched_monster_names = switchable_benched_monsters.iter().map(|monster| monster.full_name()).enumerate();
                let _ = writeln!(locked_stdout, "Choose a monster to switch with");
                for (index, monster_name) in switchable_benched_monster_names {
                    let _ = writeln!(locked_stdout, "[{}] {}", index + 1, monster_name);
                }
                let chosen_switchable_monster_index = input_as_choice_index(locked_stdout, switchable_benched_monsters.iter().count()).unwrap();
                let benched_monster = switchable_benched_monsters[chosen_switchable_monster_index];
                FullySpecifiedChoice::SwitchOut { active_monster, benched_monster }
            },
        }
    });
    Ok(UIChoice::Action(fully_specified_action_for_team))
}