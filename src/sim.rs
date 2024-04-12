mod actions;
pub mod battle;
pub mod battle_constants;
pub(crate) mod choice;
pub mod game_mechanics;
pub(crate) mod prng;

mod event;
mod ordering;

use std::{error::Error, fmt::Display};

pub use actions::Reaction; use actions::Action;
pub use battle::*;
pub use builders::{MonsterBuilderExt, MoveBuilderExt, AbilityBuilderExt};
pub use monsim_macros::*;
pub use battle_constants::*;
pub use choice::*;
pub use event::{
    contexts::*, event_dex, ActivationOrder, EventFilteringOptions, EventDispatcher, EventHandler, Event, TargetFlags,
};
pub use game_mechanics::*;
pub use monsim_utils::{Outcome, Percent, ClampedPercent};
pub(crate) use monsim_utils::{not, NOTHING, Nothing}; // For internal use

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
pub struct BattleSimulator;

impl BattleSimulator {

    pub fn simulate_turn(battle: &mut Battle, choices: PerTeam<FullySpecifiedChoice>) -> SimResult {
        
        assert!(not!(battle.is_finished), "The simulator cannot be called on a finished battle.");

        Self::increment_turn_number(battle)
            .map_err(|message| { SimError::InvalidStateReached(String::from(message))})?;
        
        battle.message_log.extend(&[
            "---", 
            EMPTY_LINE,
            &format!["Turn {turn_number}", turn_number = battle.turn_number], 
            EMPTY_LINE
            ]
        );

        // TEMP: Will need to be updated when multiple Monsters per battle is implemented.
        let mut choices = choices.as_array();
        ordering::sort_by_activation_order(
            &mut battle.prng, 
            &mut choices, 
            &mut |choice| { choice.activation_order() }
        );

        'turn: for choice in choices.into_iter() {
            
            match choice {
                FullySpecifiedChoice::Move { attacker: move_user, move_used, target, .. } => match move_!(move_used).category() {
                    MoveCategory::Physical | MoveCategory::Special => Action::use_damaging_move(battle, MoveUsed { move_user, move_used, target }),
                    MoveCategory::Status => Action::use_status_move(battle, MoveUsed {move_user, move_used, target}),
                },
                FullySpecifiedChoice::SwitchOut { active_monster_uid, benched_monster_uid, .. } => {
                    Action::perform_switch_out(battle, active_monster_uid, benched_monster_uid)
                }
            }?;

            // Check if a Monster fainted this turn
            let maybe_fainted_active_monster = battle.entities.monsters()
                .find(|monster| monster.is_fainted && battle.entities.is_active_monster(monster.uid));
            
            if let Some(fainted_active_monster) = maybe_fainted_active_monster {
                
                battle.message_log.extend(&[
                    &format!["{fainted_monster} fainted!", fainted_monster = fainted_active_monster.name()], 
                    EMPTY_LINE
                ]);
                
                // Check if any of the teams is out of usable Monsters
                let are_all_ally_team_monsters_fainted = battle.entities.ally_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted });
                let are_all_opponent_team_monsters_fainted = battle.entities.opponent_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted });
                
                if are_all_ally_team_monsters_fainted {
                    battle.is_finished = true;
                    battle.message_log.push_str("Opponent Team won!");
                    break 'turn;
                } 
                if are_all_opponent_team_monsters_fainted {
                    battle.is_finished = true;
                    battle.message_log.push_str("Ally Team won!");
                    break 'turn;
                }
            };

            battle.message_log.push_str(EMPTY_LINE);
        }

        if battle.is_finished {
            battle.message_log.extend(&[EMPTY_LINE, "The battle ended."]);
        }
        battle.message_log.extend(&["---", EMPTY_LINE]);

        Ok(NOTHING)
    }
    
    /// Fails if the turn limit (`u16::MAX`, i.e. `65535`) is exceeded. It's not expected for this to ever happen.
    pub(crate) fn increment_turn_number(battle: &mut Battle) -> Result<Nothing, &str> {
        match battle.turn_number.checked_add(1) {
            Some(turn_number) => { battle.turn_number = turn_number; Ok(NOTHING)},
            None => Err("Turn limit (65535) exceeded."),
        }
    }

    pub(crate) fn switch_out_between_turns(battle: &mut Battle, active_monster_uid: MonsterUID, benched_monster_uid: MonsterUID) -> SimResult {
        Action::perform_switch_out(battle, active_monster_uid, benched_monster_uid)
    }
}