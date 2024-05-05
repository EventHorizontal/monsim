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
pub use targetting::{TargetFlags, BoardPosition, FieldPosition};

type SimResult = Result<(), SimError>;

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

    pub fn simulate_turn(&mut self, mut action_choices: Vec<FullySpecifiedActionChoice>) -> SimResult {
        
        assert!(not!(self.battle.is_finished()), "The simulator cannot be called on a finished battle.");

        self.battle.turn_number += 1;
        
        self.battle.message_log.extend(&[
            "---", 
            EMPTY_LINE,
            &format!["Turn {turn_number}", turn_number = self.battle.turn_number], 
            EMPTY_LINE
            ]
        );

        ordering::sort_by_activation_order(
            &mut self.battle.prng, 
            &mut action_choices, 
            |choice| { choice.activation_order() }
        );

        'turn: for action_choice in action_choices.into_iter() {

            // If the actor fainted we move on to the next action.
            let actor_id = action_choice.actor_id();
            if self.battle.monster(actor_id).is_fainted() {
                self.push_message(
                    format!["{} fainted so it was unable to act.", self.battle.monster(actor_id).name()]
                );
                continue 'turn;
            }

            // Otherwise resolve the action
            match action_choice {
                FullySpecifiedActionChoice::Move { move_id, target_positions, .. } => {
                    // The target position may be empty if the target fainted with no replacement, for example.
                    let target_ids = target_positions.into_iter()
                        .map(|position| self.battle.monster_at_position(position) )
                        .flatten()
                        .map(|monster| monster.id )
                        .collect::<Vec<_>>();
                    UseMove(self, move_id.owner_id, MoveUseContext::new(move_id, MaxSizedVec::from_vec(target_ids)));
                },
                FullySpecifiedActionChoice::SwitchOut { active_monster_id, benched_monster_id, .. } => {
                    PerformSwitchOut(self, active_monster_id, SwitchContext::new(active_monster_id, benched_monster_id));
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
                break 'turn;
            }
        }

        Ok(NOTHING)
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