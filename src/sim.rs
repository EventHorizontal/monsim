pub mod actions;
pub mod battle;
pub mod battle_constants;
pub(crate) mod choice;
pub mod game_mechanics;
pub(crate) mod prng;

mod event_dispatch;
mod ordering;

use std::{error::Error, fmt::Display, ops::{Index, IndexMut, RangeInclusive}};

pub use actions::*;
pub use battle::*;
pub use builders::{MonsterBuilderExt, MoveBuilderExt, AbilityBuilderExt};
#[cfg(feature="macros")]
pub use monsim_macros::*;
pub use battle_constants::*;
pub use choice::*;
pub use event_dispatch::{
    contexts::*, events::*, ActivationOrder, EventHandlerDeck, EventFilteringOptions, EventDispatcher, EventHandler, Event, TargetFlags,
};
pub use game_mechanics::*;
pub use monsim_utils::{Outcome, Percent, ClampedPercent};
pub(crate) use monsim_utils::{not, NOTHING, Nothing};

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
    pub(crate) battle: BattleState,
}

impl BattleSimulator { // simulation

    pub fn init(battle: BattleState) -> BattleSimulator {
        BattleSimulator {
            battle,
        }
    }

    pub fn simulate_turn(&mut self, choices: PerTeam<FullySpecifiedChoice>) -> SimResult {
        
        assert!(not!(self.battle.is_finished), "The simulator cannot be called on a finished battle.");

        // TODO: Why am I so paranoid about exceeded the turn number limit?
        self.increment_turn_number()
            .map_err(|message| { SimError::InvalidStateReached(String::from(message))})?;
        
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
            &mut |choice| { choice.activation_order() }
        );

        'turn: for choice in choices.into_iter() {
            
            match choice {
                // TODO: Put a `MoveUseContext` in `FullySpecifiedChoice`?
                FullySpecifiedChoice::Move { move_user: _, move_used, target, .. } => {
                    UseMove(self, MoveUseContext::new(move_used, target));
                },
                FullySpecifiedChoice::SwitchOut { active_monster_uid: active_monster, benched_monster_uid: benched_monster, .. } => {
                    PerformSwitchOut(self, SwitchContext::new(active_monster, benched_monster))
                }
            };

            // Check if a Monster fainted this turn
            let maybe_fainted_active_monster = self.battle.monsters()
                .find(|monster| self.battle.monster(monster.uid).is_fainted && self.battle.is_active_monster(monster.uid));
            
            if let Some(fainted_active_monster) = maybe_fainted_active_monster {
                
                self.battle.message_log.extend(&[
                    &format!["{fainted_monster} fainted!", fainted_monster = fainted_active_monster.name()], 
                    EMPTY_LINE
                ]);
                
                // Check if any of the teams is out of usable Monsters
                let are_all_ally_team_monsters_fainted = self.battle.ally_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted });
                let are_all_opponent_team_monsters_fainted = self.battle.opponent_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted });
                
                if are_all_ally_team_monsters_fainted {
                    self.battle.is_finished = true;
                    self.battle.message_log.push("Opponent Team won!");
                    break 'turn;
                } 
                if are_all_opponent_team_monsters_fainted {
                    self.battle.is_finished = true;
                    self.battle.message_log.push("Ally Team won!");
                    break 'turn;
                }
            };

            self.battle.message_log.push(EMPTY_LINE);
        }

        if self.battle.is_finished {
            self.battle.message_log.extend(&[EMPTY_LINE, "The self.battle ended."]);
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
        if let Some(on_activate_handler) = self[context.move_used].on_activate_effect() {
            on_activate_handler(self, context)
        }
    }

    pub(crate) fn switch_out_between_turns(&mut self, active_monster: MonsterUID, benched_monster: MonsterUID) {
        PerformSwitchOut(self, SwitchContext::new(active_monster, benched_monster))
    }

    fn trigger_try_event<C: Copy, E: Event<EventReturnType = Outcome, ContextType = C>>(
        &mut self, 
        event: E, 
        broadcaster: MonsterUID,
        event_context: C,
    ) -> Outcome {
        EventDispatcher::dispatch_event(self, event, broadcaster, event_context, Outcome::Success, Some(Outcome::Failure))
    }
    
    fn trigger_event<R: Copy + PartialEq, C: Copy, E: Event<EventReturnType = R, ContextType = C>>(
        &mut self, 
        event: E, 
        broadcaster: MonsterUID,
        event_context: C,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        EventDispatcher::dispatch_event(self, event, broadcaster, event_context, default, short_circuit)
    }
}

impl BattleSimulator { // public interface
    
    pub fn push_message(&mut self, message: impl ToString) {
        self.battle.message_log.push(message);
    }

    fn generate_random_number_in_range_inclusive(&mut self, range: RangeInclusive<u16>) -> u16 {
        self.battle.prng.generate_random_u16_in_range(range)
    }
}

impl Index<MonsterUID> for BattleSimulator {
    type Output = Monster;

    fn index(&self, index: MonsterUID) -> &Self::Output {
        self.battle.monster(index)
    }
}

impl IndexMut<MonsterUID> for BattleSimulator {
    fn index_mut(&mut self, index: MonsterUID) -> &mut Self::Output {
        self.battle.monster_mut(index)
    }
}

impl Index<MoveUID> for BattleSimulator {
    type Output = Move;

    fn index(&self, index: MoveUID) -> &Self::Output {
        self.battle.move_(index)
    }
}

impl IndexMut<MoveUID> for BattleSimulator {
    fn index_mut(&mut self, index: MoveUID) -> &mut Self::Output {
        self.battle.move_mut(index)
    }
}

impl Index<AbilityUID> for BattleSimulator {
    type Output = Ability;

    fn index(&self, index: AbilityUID) -> &Self::Output {
        self.battle.ability(index.owner)
    }
}

impl IndexMut<AbilityUID> for BattleSimulator {
    fn index_mut(&mut self, index: AbilityUID) -> &mut Self::Output {
        self.battle.ability_mut(index.owner)
    }
}