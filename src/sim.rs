pub mod effects;
pub mod battle;
pub mod battle_constants;
pub(crate) mod choice;
pub mod game_mechanics;
pub(crate) mod prng;

mod event_dispatch;
mod ordering;
mod targetting;

use std::{error::Error, fmt::Display, ops::RangeInclusive};

pub use effects::*;
pub use battle::*;
pub use builders::{MonsterBuilderExt, MoveBuilderExt, AbilityBuilderExt, BattleFormat};
#[cfg(feature="macros")]
pub use monsim_macros::*;
pub use battle_constants::*;
pub use choice::*;
pub use event_dispatch::{
    contexts::*, events::*, EventHandlerDeck, EventFilteringOptions, EventDispatcher, EventHandler, Event,
};
pub use game_mechanics::*;
use monsim_utils::MaxSizedVec;
pub use monsim_utils::{Outcome, Percent, ClampedPercent};
pub(crate) use monsim_utils::{not, NOTHING, Nothing};
pub use ordering::ActivationOrder;
use tap::Pipe;
pub use targetting::{TargetFlags, BoardPosition, FieldPosition};

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
    pub battle: BattleState,
}

impl BattleSimulator { // simulation

    pub fn init(battle: BattleState) -> BattleSimulator {
        BattleSimulator {
            battle,
        }
    }

    pub fn simulate(mut self, mut ui: impl SimulatorUi) -> SimResult {
        // TODO: We may be able to remove this assert if we consume self.
        assert!(not!(self.battle.is_finished()), "The simulator cannot be called on a finished battle.");
        
        while not!(self.battle.is_finished()) {
            self.battle.turn_number += 1;
            let simulation_cancelled = self.simulate_turn(&mut ui)?;
            if simulation_cancelled { break }
        }

        Ok(true)
    }
    
    pub fn simulate_turn(&mut self, ui: &mut impl SimulatorUi) -> SimResult {
        
        // Beginning-of-turn Upkeep Phase
        self.battle.message_log.extend(&[
            "---", 
            EMPTY_LINE,
            &format!["Turn {turn_number}", turn_number = self.battle.turn_number], 
            EMPTY_LINE
            ]
        );

        ui.update_battle_status(&mut self.battle);

        // Choice Phase    
        let mut action_schedule = Vec::new();
        let mut monsters_selected_for_switch = MaxSizedVec::empty();
        let active_monster_ids = self.battle.active_monster_ids();
        for active_monster_id in active_monster_ids {
            let available_choices_for_monster = self.battle.available_choices_for(self.battle.monster(active_monster_id), &monsters_selected_for_switch);
            let partially_specified_action_choice = ui.prompt_user_to_select_action_for_monster(&mut self.battle, active_monster_id, available_choices_for_monster);
            let fully_specified_action_choice = match partially_specified_action_choice {
                PartiallySpecifiedActionChoice::Move { move_id, possible_target_positions, activation_order, .. } => {
                    
                    let move_targets_all = self.battle.move_(move_id).allowed_target_flags().contains(TargetFlags::ALL);
                    let move_targets_any = self.battle.move_(move_id).allowed_target_flags().contains(TargetFlags::ANY);
                    
                    // The engine autopicks if the move targets all possible targets...
                    if move_targets_all {
                        let target_positions = possible_target_positions;
                        FullySpecifiedActionChoice::Move { move_id, target_positions, activation_order }
                    } else if move_targets_any {
                        // ...and if the move targets any possible target and there is only one possible target.
                        let chosen_target_position = (if possible_target_positions.count() == 1 {
                            possible_target_positions[0]
                        } else {
                            ui.prompt_user_to_select_target_position(&mut self.battle, move_id, possible_target_positions)    
                        })
                        .pipe(|chosen_target_position| {
                            MaxSizedVec::from_slice(&[chosen_target_position])
                        });
                        FullySpecifiedActionChoice::Move { move_id, target_positions: chosen_target_position, activation_order }
                    } else {
                        unreachable!("Expected move to target either ALL or ANY target(s).")
                    }
                },
                PartiallySpecifiedActionChoice::SwitchOut { active_monster_id, switchable_benched_monster_ids, activation_order, .. } => {
                    let active_monster_position = self.battle.monster(active_monster_id).field_position().expect("active_monster shoudld be on the field.");
                    let selected_benched_monster_id = ui.prompt_user_to_select_benched_monster_to_switch_in(&mut self.battle, active_monster_position, switchable_benched_monster_ids);
                    monsters_selected_for_switch.push(selected_benched_monster_id);
                    FullySpecifiedActionChoice::SwitchOut { active_monster_id, benched_monster_id: selected_benched_monster_id, activation_order }
                },
                PartiallySpecifiedActionChoice::CancelSimulation => {
                    return Ok(true);
                },
            };
            action_schedule.push(fully_specified_action_choice);
        }
    
        // Action Phase
        ordering::sort_by_activation_order(
            &mut self.battle.prng, 
            &mut action_schedule, 
            |choice| { choice.activation_order() }
        );

        'turn: for action_choice in action_schedule.into_iter() {

            // If the actor fainted we move on to the next action..
            let actor_id = action_choice.actor_id();
            if self.battle.monster(actor_id).is_fainted() {
                self.push_message(
                    format!["{} fainted so it was unable to act.", self.battle.monster(actor_id).name()]
                );
                continue 'turn;
            }

            // ...otherwise resolve the action
            match action_choice {
                FullySpecifiedActionChoice::Move { move_id, target_positions, .. } => {
                    // The target position may be empty if the target fainted with no replacement, for example.
                    let target_ids = target_positions.into_iter()
                        .map(|position| self.battle.monster_at_position(position) )
                        .flatten()
                        .map(|monster| monster.id )
                        .collect::<Vec<_>>();
                    effects::UseMove(self, move_id.owner_id, MoveUseContext::new(move_id, MaxSizedVec::from_vec(target_ids)));
                },
                FullySpecifiedActionChoice::SwitchOut { active_monster_id, benched_monster_id, .. } => {
                    effects::PerformSwitchOut(self, active_monster_id, SwitchContext::new(active_monster_id, benched_monster_id));
                },
            }

            self.push_message(EMPTY_LINE);

            // After each action, we check if the the battle is finished or not.
            let ally_team_wiped = self.battle.ally_team().monsters().all(|monster| monster.is_fainted());
            let opponent_team_wiped = self.battle.opponent_team().monsters().all(|monster| monster.is_fainted());

            match (ally_team_wiped, opponent_team_wiped) {
                (true, true) => {
                    self.push_message("Neither team has any usable Monsters, it's a tie!")
                },
                (true, false) => {
                    self.push_message("Opponent team won!")
                },
                (false, true) => {
                    self.push_message("Ally team won!")
                },
                (false, false) => {},
            }

            if ally_team_wiped || opponent_team_wiped {
                self.battle.message_log.extend(&[
                    EMPTY_LINE, 
                    "The battle ended.",
                    "---",
                    EMPTY_LINE,
                ]);
                self.battle.message_log.show_new_messages();
                return Ok(true)
            }
        }

        self.battle.message_log.show_new_messages();

        // Monster Replacement Phase
        let empty_field_positions = self.battle.valid_positions_in_format()
            .into_iter()
            .filter(|position| { self.battle.monster_at_position(*position).is_none() })
            .collect::<Vec<_>>();
        for empty_field_position in empty_field_positions {
            let team_id = empty_field_position.side();
            let switchable_benched_monster_ids = self.battle.switchable_benched_monster_ids(team_id, &MaxSizedVec::empty());
            if switchable_benched_monster_ids.is_empty() {
                self.push_message(format!["{} is empty but {} is out of switchable Monsters!", empty_field_position, team_id]);
            } else {
                let monster_selected_for_switch_id = ui.prompt_user_to_select_benched_monster_to_switch_in(&mut self.battle, empty_field_position, switchable_benched_monster_ids);
                /*
                INFO: Monsters get switched in immediately if they are replacing a fainted Monster
                that fainted last turn, so we don't add them to the 'action_schedule'.
                */
                effects::ReplaceFaintedMonster(self, monster_selected_for_switch_id, (monster_selected_for_switch_id, empty_field_position))
            }
        }

        self.battle.message_log.show_new_messages();

        Ok(false)
    }

    fn trigger_try_event<C: Copy, E: Event<EventReturnType = Outcome, ContextType = C>>(
        &mut self, 
        event: E, 
        broadcaster_id: MonsterID,
        event_context: C,
    ) -> Outcome {
        EventDispatcher::dispatch_event(self, event, broadcaster_id, event_context, Outcome::Success, Some(Outcome::Failure))
    }
    
    fn trigger_event<R: Copy + PartialEq, C: Copy, E: Event<EventReturnType = R, ContextType = C>>(
        &mut self, 
        event: E, 
        broadcaster_id: MonsterID,
        event_context: C,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        EventDispatcher::dispatch_event(self, event, broadcaster_id, event_context, default, short_circuit)
    }
}

impl BattleSimulator { // public
    
    pub fn push_message(&mut self, message: impl ToString) {
        self.battle.message_log.push(message);
    }

    fn generate_random_number_in_range_inclusive(&mut self, range: RangeInclusive<u16>) -> u16 {
        self.battle.prng.generate_random_u16_in_range(range)
    }
}