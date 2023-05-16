use std::io::Write;

use super::game_mechanics::{BattlerUID, MoveUID};
use crate::{print_empty_line, BattleContext, BattlerNumber, Move, MoveNumber, TeamID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    Move {
        move_uid: MoveUID,
        target_uid: BattlerUID,
    },
}

impl ActionChoice {
    pub(crate) fn chooser(&self) -> BattlerUID {
        match self {
            ActionChoice::Move {
                move_uid,
                target_uid: _,
            } => move_uid.battler_uid,
        }
    }

    pub(crate) fn target(&self) -> BattlerUID {
        match self {
            ActionChoice::Move {
                move_uid: _,
                target_uid,
            } => *target_uid,
        }
    }
}

// TODO: If/when we support double battles, this needs to take 1-2 choices per team.
pub struct UserInput(AvailableActionChoices);

impl UserInput {
    pub fn receive_input(_context: &BattleContext) -> Self {
        // let mut choice_ids = Vec::new();
        // println!("Please choose a move");
        // print_empty_line();
        // for battler in context.battlers_on_field() {
        //     let mut move_count = 0;
        //     println!("{}'s choices.", battler.monster.nickname);
        //     for (i, move_) in battler.moveset.moves().flatten().enumerate() {
        //         println!("[{}] {}", i, move_.species.name);
        //         move_count += 1;
        //     }
        //     print_empty_line();
        //     let mut waiting_for_input = true;
        //     while waiting_for_input {
        //         print!("Choice: ");
        //         std::io::stdout().flush().expect("flush failed");
        //         let mut user_input = String::new();
        //         std::io::stdin()
        //             .read_line(&mut user_input)
        //             .expect("Error: Stadard Input failed to read the input.");
        //         print_empty_line();
        //         let numeric_input = user_input[0..1]
        //             .parse::<usize>()
        //             .expect("The choice was not parseable.");
        //         if numeric_input < move_count {
        //             waiting_for_input = false;
        //             choice_ids.push(MoveNumber::from(numeric_input));
        //         } else {
        //             println!("Malformed input! Please try again.");
        //             print_empty_line();
        //         }
        //     }
        // }

        // TODO: Make a TUI to support user input and stop hard coding this.
        // TEMP: Hard coded until UI is sophisticated enought to allow for user input.
        // UserInput {
        //     ally_choices: ActionChoice::Move {
        //         move_uid: MoveUID {
        //             battler_uid: BattlerUID {
        //                 team_id: TeamID::Ally,
        //                 battler_number: BattlerNumber::First,
        //             },
        //             move_number: MoveNumber::Fourth,
        //         },
        //         target_uid: BattlerUID {
        //             team_id: TeamID::Opponent,
        //             battler_number: BattlerNumber::First,
        //         },
        //     },
        //     opponent_choices: ActionChoice::Move {
        //         move_uid: MoveUID {
        //             battler_uid: BattlerUID {
        //                 team_id: TeamID::Opponent,
        //                 battler_number: BattlerNumber::First,
        //             },
        //             move_number: MoveNumber::First,
        //         },
        //         target_uid: BattlerUID {
        //             team_id: TeamID::Ally,
        //             battler_number: BattlerNumber::First,
        //         },
        //     },
        // }
        todo!()
    }
}

#[derive(Debug)]
pub struct AvailableActionChoices {
    pub ally_team_choices: TeamActionChoiceList,
    pub opponent_team_choices: TeamActionChoiceList,
}

pub type TeamActionChoiceList = Vec<ActionChoice>;
