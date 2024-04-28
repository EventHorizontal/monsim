pub mod effects;
pub mod battle;
pub mod battle_constants;
pub(crate) mod choice;
pub mod game_mechanics;
pub(crate) mod prng;

mod event_dispatch;
mod ordering;

use std::{error::Error, fmt::Display, ops::RangeInclusive};

pub use effects::*;
pub use battle::*;
pub use builders::{MonsterBuilderExt, MoveBuilderExt, AbilityBuilderExt};
#[cfg(feature="macros")]
pub use monsim_macros::*;
pub use battle_constants::*;
pub use choice::*;
pub use event_dispatch::{
    contexts::*, events::*, EventHandlerDeck, EventFilteringOptions, EventDispatcher, EventHandler, Event, TargetFlags,
};
pub use game_mechanics::*;
pub use monsim_utils::{Outcome, Percent, ClampedPercent};
pub(crate) use monsim_utils::{not, NOTHING, Nothing};
pub use ordering::ActivationOrder;

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

/// The main engine behind `monsim`. This struct is a namespace for all the simulator functionality. It contains no data, just functions that transform a `Battle` from one state to another.
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

    pub fn simulate_turn(&mut self, choices: PerTeam<FullySpecifiedChoice>) -> SimResult {
        
        assert!(not!(self.battle.is_finished()), "The simulator cannot be called on a finished battle.");

        /*
        INFO: Removed error on turn number increment. We are probably not going to exceed the `65535` threshold.
        And if we do, something is wrong anyway.
        */
        _ = self.increment_turn_number();
        
        self.battle.message_log.extend(&[
            "---", 
            EMPTY_LINE,
            &format!["Turn {turn_number}", turn_number = self.battle.turn_number], 
            EMPTY_LINE
            ]
        );

        // TEMP: Will need to be updated when multiple Monsters per self.battle is implemented.
        let mut choices = choices.as_array();
        ordering::sort_by_activation_order(
            &mut self.battle.prng, 
            &mut choices, 
            |choice| { choice.activation_order() }
        );

        'turn: for choice in choices.into_iter() {
            
            match choice {
                FullySpecifiedChoice::Move { move_user_id: _, move_id, target_id, .. } => {
                    UseMove(self, MoveUseContext::new(move_id, target_id));
                },
                FullySpecifiedChoice::SwitchOut { active_monster_id, benched_monster_id, .. } => {
                    PerformSwitchOut(self, SwitchContext::new(active_monster_id, benched_monster_id))
                }
            };

            // Check if a Monster fainted this turn
            let maybe_fainted_active_monster = self.battle.monsters()
                .find(|monster| self.battle.monster(monster.id).is_fainted() && self.battle.is_active_monster(monster.id));
            
            if let Some(fainted_active_monster) = maybe_fainted_active_monster {
                
                self.battle.message_log.extend(&[
                    &format!["{fainted_monster} fainted!", fainted_monster = fainted_active_monster.name()], 
                    EMPTY_LINE
                ]);
                
                // Check if any of the teams is out of usable Monsters
                let ally_team_wiped = self.battle.ally_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted() });
                let opponent_team_wiped = self.battle.opponent_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted() });
                let team_wiped = (ally_team_wiped, opponent_team_wiped);
                match team_wiped {
                    (true, false) => {
                        self.battle.message_log.push("Opponent Team won!");
                        break 'turn;
                    },
                    (false, true) => {
                        self.battle.message_log.push("Ally Team won!");
                        break 'turn;
                    },
                    (true, true) => {
                        self.battle.message_log.push("The teams tied!");
                    },
                    (false, false) => {}
                }
            };

            self.battle.message_log.push(EMPTY_LINE);
        }

        if self.battle.is_finished() {
            self.battle.message_log.extend(&[EMPTY_LINE, "The battle ended."]);
        }
        self.battle.message_log.extend(&["---", EMPTY_LINE]);

        Ok(NOTHING)
    }
    
    /// Fails if the turn limit (`u16::MAX`, i.e. `65535`) is exceeded. It's not expected for this to ever happen.
    pub(crate) fn increment_turn_number(&mut self) -> Result<Nothing, &str> {
        match self.battle.turn_number.checked_add(1) {
            Some(turn_number) => { self.battle.turn_number = turn_number; Ok(NOTHING)},
            None => Err("Turn limit (65535) exceeded."),
        }
    }
    
    fn activate_move_effect(&mut self, context: MoveUseContext) {
        (self.battle.move_(context.move_used_id).on_activate_effect())(self, context)
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