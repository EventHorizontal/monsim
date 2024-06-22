pub mod battle;
pub mod battle_constants;
pub(crate) mod choice;
pub mod effects;
pub mod game_mechanics;
pub(crate) mod prng;

mod event_dispatcher;
mod ordering;
mod targetting;

use std::{error::Error, fmt::Display};

pub use battle::*;
pub use battle_constants::*;
pub use builder::{AbilityBuilderExt, BattleFormat, ItemBuilderExt, MonsterBuilderExt, MoveBuilderExt};
pub use choice::*;
pub use effects::*;
pub use event_dispatcher::{contexts::*, EventFilteringOptions, EventHandler, EventListener, NullEventListener};
use event_dispatcher::{events::OnTurnEndEvent, EventDispatcher};
pub(crate) use event_dispatcher::{Broadcaster, OwnedEventHandlerWithReceiver};
pub use game_mechanics::*;
#[cfg(feature = "macros")]
pub use monsim_macros::*;
use monsim_utils::{not, MaxSizedVec, NOTHING};
pub use monsim_utils::{ClampedPercent, Count, Outcome, Percent};
pub use ordering::ActivationOrder;
pub use status::{PersistentStatusDexEntry, PersistentStatusSpecies, VolatileStatusDexEntry, VolatileStatusSpecies};
use tap::Pipe;
pub use targetting::{BoardPosition, FieldPosition, PositionRelationFlags};

/// `bool` indicates whether the Simulation should be cancelled early.
type SimResult = Result<bool, SimError>;

#[derive(Debug, PartialEq, Eq)]
pub enum SimError {
    InvalidStateReached(String),
}

impl Error for SimError {}

impl Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::InvalidStateReached(message) => write!(f, "{}", message),
        }
    }
}

/**
The main engine behind `monsim`. This struct is a namespace for all the simulator functionality. It contains no data,
just functions that transform a `Battle` from one state to another.
*/
#[derive(Debug)]
pub struct BattleSimulator {
    pub battle: Battle,
}

impl BattleSimulator {
    pub fn init(battle: Battle) -> BattleSimulator {
        BattleSimulator { battle }
    }

    pub fn simulate(mut self, mut ui: impl SimulatorUi) -> SimResult {
        // TODO: We may be able to remove this assert if we consume self.
        assert!(not!(self.battle.is_finished()), "The simulator cannot be called on a finished battle.");

        while not!(self.battle.is_finished()) {
            self.battle.turn_number += 1;
            let simulation_cancelled = self.simulate_turn(&mut ui)?;
            if simulation_cancelled {
                break;
            }
        }

        Ok(true)
    }

    pub fn simulate_turn(&mut self, ui: &mut impl SimulatorUi) -> SimResult {
        // Beginning-of-Turn Upkeep Phase -------------------------------- //
        self.battle.queue_multiple_messages(&[
            "---",
            EMPTY_LINE,
            &format!["Turn {turn_number}", turn_number = self.battle.turn_number],
            EMPTY_LINE,
        ]);

        ui.update_battle_status(&mut self.battle);

        // Choice Phase ------------------------------------------------- //
        let mut action_schedule = Vec::new();
        let mut monsters_selected_for_switch = MaxSizedVec::empty();
        let active_monster_ids = self.battle.active_monster_ids();
        for active_monster_id in active_monster_ids {
            let available_choices_for_monster = self
                .battle
                .available_choices_for_monster(self.battle.monster(active_monster_id), &monsters_selected_for_switch);
            let partially_specified_action_choice =
                ui.prompt_user_to_select_action_for_monster(&mut self.battle, active_monster_id, available_choices_for_monster);
            let fully_specified_action_choice = match partially_specified_action_choice {
                PartiallySpecifiedActionChoice::Move {
                    move_id,
                    possible_target_positions,
                    activation_order,
                    ..
                } => {
                    let move_targets_all = self
                        .battle
                        .move_(move_id)
                        .allowed_target_position_relation_flags()
                        .contains(PositionRelationFlags::ALL);
                    let move_targets_any = self
                        .battle
                        .move_(move_id)
                        .allowed_target_position_relation_flags()
                        .contains(PositionRelationFlags::ANY);

                    // The engine autopicks if the move targets all possible targets...
                    if move_targets_all {
                        let target_positions = possible_target_positions;
                        FullySpecifiedActionChoice::Move {
                            move_id,
                            target_positions,
                            activation_order,
                        }
                    } else if move_targets_any {
                        // ...and if the move targets any possible target and there is only one possible target.
                        let chosen_target_position = (if possible_target_positions.count() == 1 {
                            possible_target_positions[0]
                        } else {
                            ui.prompt_user_to_select_target_position(&mut self.battle, move_id, possible_target_positions)
                        })
                        .pipe(|chosen_target_position| MaxSizedVec::from_slice(&[chosen_target_position]));
                        FullySpecifiedActionChoice::Move {
                            move_id,
                            target_positions: chosen_target_position,
                            activation_order,
                        }
                    } else {
                        unreachable!("Expected move to target either ALL or ANY target(s).")
                    }
                }
                PartiallySpecifiedActionChoice::SwitchOut {
                    active_monster_id,
                    switchable_benched_monster_ids,
                    activation_order,
                    ..
                } => {
                    let active_monster_position = self
                        .battle
                        .monster(active_monster_id)
                        .field_position()
                        .expect("active_monster shoudld be on the field.");
                    let selected_benched_monster_id =
                        ui.prompt_user_to_select_benched_monster_to_switch_in(&mut self.battle, active_monster_position, switchable_benched_monster_ids);
                    monsters_selected_for_switch.push(selected_benched_monster_id);
                    FullySpecifiedActionChoice::SwitchOut {
                        active_monster_id,
                        benched_monster_id: selected_benched_monster_id,
                        activation_order,
                    }
                }
                PartiallySpecifiedActionChoice::CancelSimulation => {
                    return Ok(true);
                }
            };
            action_schedule.push(fully_specified_action_choice);
        }

        // Action Phase --------------------------------------------- //

        #[cfg(feature = "debug")]
        let action_phase_start_time = std::time::SystemTime::now();

        ordering::sort_by_activation_order(&mut self.battle.prng, &mut action_schedule, |choice| choice.activation_order());

        'turn: for action_choice in action_schedule.into_iter() {
            // If the actor fainted we move on to the next action..
            let actor_id = action_choice.actor_id();
            if self.battle.monster(actor_id).is_fainted() {
                self.battle.queue_multiple_messages(
                    [
                        &format!["{} fainted so it was unable to act.", self.battle.monster(actor_id).name()],
                        EMPTY_LINE,
                    ]
                    .as_slice(),
                );
                continue 'turn;
            }

            // ...otherwise resolve the action
            match action_choice {
                FullySpecifiedActionChoice::Move { move_id, target_positions, .. } => {
                    // The target position may be empty if the target fainted with no replacement, for example.
                    let target_ids = target_positions
                        .into_iter()
                        .filter_map(|position| self.battle.monster_at_position(position))
                        .map(|monster| monster.id)
                        .collect::<Vec<_>>();
                    effects::use_move(&mut self.battle, MoveUseContext::new(move_id, MaxSizedVec::from_vec(target_ids)));
                }
                FullySpecifiedActionChoice::SwitchOut {
                    active_monster_id,
                    benched_monster_id,
                    ..
                } => {
                    effects::switch_monsters(&mut self.battle, SwitchContext::new(active_monster_id, benched_monster_id));
                }
            }

            self.battle.queue_message(EMPTY_LINE);

            // After each action, we check if the the battle is finished or not.
            let ally_team_wiped = self.battle.ally_team().monsters().all(|monster| monster.is_fainted());
            let opponent_team_wiped = self.battle.opponent_team().monsters().all(|monster| monster.is_fainted());

            match (ally_team_wiped, opponent_team_wiped) {
                (true, true) => self.battle.queue_message("Neither team has any usable Monsters, it's a tie!"),
                (true, false) => self.battle.queue_message("Opponent team won!"),
                (false, true) => self.battle.queue_message("Ally team won!"),
                (false, false) => {}
            }

            if ally_team_wiped || opponent_team_wiped {
                self.battle.queue_multiple_messages(&[EMPTY_LINE, "The battle ended.", "---", EMPTY_LINE]);
                self.battle.message_log.show_new_messages();
                return Ok(true);
            }
        }

        #[cfg(feature = "debug")]
        let action_phase_phase_elapsed_time = action_phase_start_time.elapsed().expect("Expected to always get duration");

        self.battle.message_log.show_new_messages();

        // Monster Replacement Phase ---------------------------------------------- //
        let empty_field_positions = self
            .battle
            .format()
            .valid_positions()
            .into_iter()
            .filter(|position| self.battle.monster_at_position(*position).is_none())
            .collect::<Vec<_>>();
        for empty_field_position in empty_field_positions {
            let team_id = empty_field_position.side();
            let switchable_benched_monster_ids = self.battle.switchable_benched_monster_ids(team_id, &MaxSizedVec::empty());
            if switchable_benched_monster_ids.is_empty() {
                self.battle
                    .queue_message(format!["({} is empty but {} is out of switchable Monsters)", empty_field_position, team_id]);
            } else {
                let monster_selected_for_switch_id =
                    ui.prompt_user_to_select_benched_monster_to_switch_in(&mut self.battle, empty_field_position, switchable_benched_monster_ids);
                /*
                INFO: Monsters get switched in immediately if they are replacing a fainted Monster
                that fainted last turn, so we don't add them to the 'action_schedule'.
                */
                effects::switch_in_monster(&mut self.battle, monster_selected_for_switch_id, empty_field_position)
            }
        }

        // If we are in an Battle Format with multiple Monsters per side, we need to check whether
        // either team (or both) has only one monster remaining and then move it to the middle.

        if matches!(self.battle.format, BattleFormat::Double | BattleFormat::Triple) {
            let remaining_monsters_count_per_team = self
                .battle
                .teams()
                .map_clone(|team| team.monsters().filter(|monster| not![monster.is_fainted()]).count());

            for (team_id, remaining_monsters_count) in remaining_monsters_count_per_team.iter_with_team_id() {
                if remaining_monsters_count == 1 {
                    let active_monster_for_team_id = self
                        .battle
                        .team(team_id)
                        .active_monsters()
                        .pop()
                        .expect("Expected to get a Monster since remaining monster count is 1")
                        .id;

                    let centre_position = match team_id {
                        TeamID::Allies => FieldPosition::AllySideCentre,
                        TeamID::Opponents => FieldPosition::OpponentSideCentre,
                    };

                    // Only shift the monster if it isn't already in the centre.
                    if self.battle.monster(active_monster_for_team_id).board_position != BoardPosition::Field(centre_position) {
                        effects::shift_monster(&mut self.battle, active_monster_for_team_id, centre_position);
                    }
                }
            }
        }

        // End-of-Turn Upkeep Phase --------------------------------------------- //

        #[cfg(feature = "debug")]
        let end_of_turn_phase_start_time = std::time::SystemTime::now();

        // Update volatile status state
        for monster in self.battle.monsters_mut() {
            let mut deletion_queue = Vec::new();
            for (index, volatile_status) in monster.volatile_statuses.iter_mut().enumerate() {
                if volatile_status.remaining_turns == 0 {
                    deletion_queue.push(index);
                } else {
                    volatile_status.remaining_turns -= 1;
                }
            }
            for index in deletion_queue {
                monster.volatile_statuses.remove(index);
            }
        }

        let mut should_remove_weather = false;
        if let Some(weather) = self.battle.environment_mut().weather_mut() {
            weather.remaining_turns -= 1;
            if weather.remaining_turns == 0 {
                should_remove_weather = true;
            }
        }
        if should_remove_weather {
            clear_weather(&mut self.battle);
        }

        EventDispatcher::dispatch_notify_event(&mut self.battle, OnTurnEndEvent, NOTHING, NOTHING);

        self.battle.message_log.show_new_messages();

        #[cfg(feature = "debug")]
        {
            let elapsed_time = end_of_turn_phase_start_time.elapsed().expect("Expected to always get duration.") + action_phase_phase_elapsed_time;
            println!("The turn took {:?} to simulate.", elapsed_time);
        }

        Ok(false)
    }
}
