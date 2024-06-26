use crate::{
    sim::{AvailableChoices, Battle, PartiallySpecifiedActionChoice},
    FieldPosition, MonsterID, MoveID, SimulatorUi, TeamID,
};
use monsim_macros::{mon, mov};
use monsim_utils::MaxSizedVec;
use std::io::{stdout, Write};

// TODO: We might eventually want to handle io::errors somehow?
pub struct Cli;

impl SimulatorUi for Cli {
    fn update_battle_status(&self, battle: &mut Battle) {
        let mut locked_stdout = stdout().lock();
        _ = writeln![locked_stdout];
        _ = writeln![locked_stdout, "Current Battle Status:"];
        _ = writeln![locked_stdout];
        _ = writeln![locked_stdout, "Active Monsters:"];
        for active_monster_id in battle.active_monster_ids() {
            _ = writeln![locked_stdout, "\t{}: {}", active_monster_id, mon![active_monster_id].status_string()];
        }
        _ = writeln![locked_stdout];
        _ = writeln![locked_stdout, "Ally Team Status:"];
        _ = writeln![locked_stdout, "\t{}", battle.ally_team().team_status_string().replace('\n', "\n\t")];
        _ = writeln![locked_stdout, "Opponent Team Status:"];
        _ = writeln![locked_stdout, "\t{}", battle.opponent_team().team_status_string().replace('\n', "\n\t")];
        _ = writeln![locked_stdout, "Environment:"];
        _ = writeln![
            locked_stdout,
            "\tCurrent Weather: {}",
            battle.environment().weather().map_or("None".to_string(), |weather| format![
                "{} ({} turns remaining)",
                weather.name(),
                weather.remaining_turns
            ])
        ];
        _ = writeln![
            locked_stdout,
            "\tCurrent Terrain: {}",
            battle.environment().terrain().map_or("None".to_string(), |terrain| format![
                "{} ({} turns remaining)",
                terrain.name(),
                terrain.remaining_turns
            ])
        ];
        _ = writeln![locked_stdout, "\tTraps:"];
        let traps = battle
            .environment()
            .traps()
            .map_clone(|trap| if let Some(trap) = trap { trap.name() } else { "None" });
        _ = writeln![locked_stdout, "\t\tAllySide    : {}", traps[TeamID::Allies]];
        _ = writeln![locked_stdout, "\t\tOpponentSide: {}", traps[TeamID::Opponents]];
        _ = writeln![locked_stdout];
    }

    fn prompt_user_to_select_action_for_monster(
        &self,
        battle: &mut Battle,
        monster_id: MonsterID,
        available_choices_for_monster: AvailableChoices,
    ) -> PartiallySpecifiedActionChoice {
        let mut locked_stdout = stdout().lock();
        _ = writeln![locked_stdout, "Choose an action for {}", mon![monster_id].name()];
        for (index, available_choice) in available_choices_for_monster
            .choices()
            .into_iter()
            .chain([PartiallySpecifiedActionChoice::CancelSimulation].into_iter())
            .enumerate()
        {
            let display_text = match available_choice {
                PartiallySpecifiedActionChoice::Move { move_id, .. } => format!["Use {}", mov![move_id].name()],
                PartiallySpecifiedActionChoice::SwitchOut { .. } => String::from("Switch Out"),
                PartiallySpecifiedActionChoice::CancelSimulation => String::from("Exit Monsim"),
            };
            _ = writeln![locked_stdout, "[{}] {}", index + 1, display_text];
        }
        let available_choice_count = available_choices_for_monster.count();
        let exit_index = available_choice_count + 1;
        let total_choice_count = exit_index;

        _ = writeln![locked_stdout];
        let user_choice_index = self.prompt_user_for_choice_index(total_choice_count);

        if user_choice_index < available_choice_count {
            available_choices_for_monster[user_choice_index]
        } else if user_choice_index == exit_index - 1 {
            _ = writeln![locked_stdout, "Exiting..."];
            PartiallySpecifiedActionChoice::CancelSimulation
        } else {
            unreachable!("User choice index is already validated.");
        }
    }

    fn prompt_user_to_select_target_position(
        &self,
        battle: &mut Battle,
        move_id: MoveID,
        possible_target_positions: MaxSizedVec<FieldPosition, 6>,
    ) -> FieldPosition {
        let mut locked_stdout = stdout().lock();
        _ = writeln![locked_stdout, "Please choose a target for {}", mov![move_id].name()];
        for (index, possible_target_position) in possible_target_positions.into_iter().enumerate() {
            let target_name = battle
                .monster_at_position(possible_target_position)
                .expect("Only positions with Monsters are passed to this function.")
                .full_name();
            _ = writeln![locked_stdout, "[{}] {}", index + 1, target_name];
        }
        _ = writeln![locked_stdout];
        let user_choice_index = self.prompt_user_for_choice_index(possible_target_positions.count());
        let selected_target_position = possible_target_positions[user_choice_index];
        selected_target_position
    }

    fn prompt_user_to_select_benched_monster_to_switch_in(
        &self,
        battle: &mut Battle,
        switch_position: FieldPosition,
        switchable_benched_monster_ids: MaxSizedVec<MonsterID, 5>,
    ) -> MonsterID {
        let mut locked_stdout = stdout().lock();
        match battle.monster_at_position(switch_position) {
            Some(active_monster) => {
                _ = writeln![locked_stdout, "Please choose a Monster to switch in for {}", active_monster.full_name()];
            }
            None => {
                _ = writeln![locked_stdout, "Please choose a Monster to switch in to {}", switch_position];
            }
        }
        for (index, switchable_benched_monster_id) in switchable_benched_monster_ids.into_iter().enumerate() {
            let benched_monster_name = mon![switchable_benched_monster_id].name();
            _ = writeln![locked_stdout, "[{}] {}", index + 1, benched_monster_name];
        }
        _ = writeln![locked_stdout];
        let user_choice_index = self.prompt_user_for_choice_index(switchable_benched_monster_ids.count());
        let selected_benched_monster_id = switchable_benched_monster_ids[user_choice_index];
        selected_benched_monster_id
    }
}

impl Cli {
    pub fn new() -> Cli {
        Cli
    }

    fn prompt_user_for_choice_index(&self, total_action_choice_count: usize) -> usize {
        let mut locked_stdout = stdout().lock();
        loop {
            // We keep asking until the input is valid.
            _ = writeln![locked_stdout, "Please enter the number corresponding to the choice you would like to make."];
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            // INFO: Windows uses \r\n. Linux uses only \n, so we use `trim()` which accounts for both.
            let input = input.trim();
            let chosen_action_index = input.chars().next();
            if input.len() == 1 {
                let maybe_action_choice_index = chosen_action_index.and_then(|char| char.to_digit(10));
                if let Some(action_choice_index) = maybe_action_choice_index {
                    if 0 < action_choice_index && action_choice_index <= total_action_choice_count as u32 {
                        return action_choice_index as usize - 1; // The -1 converts back to zero based counting
                    }
                }
            };
            println!("Invalid index. Please try again.");
        }
    }
}

impl Default for Cli {
    fn default() -> Self {
        Cli::new()
    }
}
