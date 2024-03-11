mod actions;
pub mod battle;
pub mod battle_constants;
pub mod choice;
pub mod game_mechanics;
pub mod prng;

mod event;
mod ordering;

use std::{error::Error, fmt::Display};

pub use actions::Effect; use actions::Action;
pub use battle::*;
pub use battle_builder_macro::build_battle;
pub use battle_constants::*;
pub use choice::*;
pub use event::{
    broadcast_contexts::*, event_dex, ActivationOrder, EventHandlerDeck, EventFilteringOptions, EventDispatcher, EventHandler, InBattleEvent, TargetFlags,
};
pub use game_mechanics::*;
pub use monsim_utils::{self as utils, Outcome, Percent, ClampedPercent, Ally, Opponent};
pub(crate) use utils::{not, NOTHING, Nothing}; // For internal use

use prng::Prng;

type TurnResult = Result<(), SimError>;

#[derive(Debug, PartialEq, Eq)]
pub enum SimError {
    InvalidStateReached(String),
}

impl Error for SimError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

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

    pub fn simulate_turn(battle: &mut Battle, choices: ChoicesForTurn) -> TurnResult {
        
        assert!(not!(battle.is_finished), "The simulator cannot be called on a finished battle.");

        battle.message_log.set_last_turn_cursor_to_log_length();
        battle.increment_turn_number()
            .map_err(|message| { SimError::InvalidStateReached(String::from(message))})?;
        
        battle.message_log.extend(&[
            "---", 
            EMPTY_LINE,
            &format!["Turn {turn_number}", turn_number = battle.turn_number], 
            EMPTY_LINE
            ]
        );

        let mut choices = choices.as_array();
        ordering::sort_choices_by_activation_order(battle, &mut choices);

        'turn: for choice in choices.into_iter() {
            
            match choice {
                FullySpecifiedChoice::Move { move_uid, target_uid } => match battle.move_(move_uid).category() {
                    MoveCategory::Physical | MoveCategory::Special => Action::use_damaging_move(battle, move_uid, target_uid),
                    MoveCategory::Status => Action::use_status_move(battle, move_uid, target_uid),
                },
                FullySpecifiedChoice::SwitchOut { switcher_uid, switchee_uid } => {
                    Action::perform_switch_out(battle, switcher_uid, switchee_uid)
                }
            }?;

            // Check if a Monster fainted this turn
            let maybe_fainted_active_monster = battle.monsters()
                .find(|monster| battle.monster(monster.uid).is_fainted && battle.is_active_monster(monster.uid));
            
            if let Some(fainted_active_monster) = maybe_fainted_active_monster {
                
                battle.message_log.extend(&[
                    &format!["{fainted_monster} fainted!", fainted_monster = fainted_active_monster.name()], 
                    EMPTY_LINE
                ]);
                
                // Check if any of the teams is out of usable Monsters
                let are_all_ally_team_monsters_fainted = battle.ally_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted });
                let are_all_opponent_team_monsters_fainted = battle.opponent_team()
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
    
    pub(crate) fn switch_out_between_turns(battle: &mut Battle, active_monster_uid: MonsterUID, benched_monster_uid: MonsterUID) -> TurnResult {
        Action::perform_switch_out(battle, active_monster_uid, benched_monster_uid)
    }
}