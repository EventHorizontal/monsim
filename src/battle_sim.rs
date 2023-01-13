pub mod battle_context;
pub mod game_mechanics;
mod prng;
mod event;
mod global_constants;
mod action;

use event::*;
use game_mechanics::*;
use battle_context::*;
use prng::LCRNG;
use global_constants::*;
use action::*;


pub use battle_context::BattleContext;
pub use bcontext_macro::bcontext;

#[test]
fn test_bcontext_macro() {

    let test_bcontext = bcontext!(
        {
            AllyTeam {
                mon Torchic "Ruby" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire, 
                },
                mon Torchic "Sapphire" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire, 
                },
                mon Torchic "Emerald" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire, 
                },
            },
            OpponentTeam {
                mon Torchic "Cheerio" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            }
        }  
    );
    assert_eq!(
        test_bcontext, 
        BattleContext::new(
            crate::battle_sim::game_mechanics::MonsterTeam::new([
                Some(crate::battle_sim::game_mechanics::Battler::new(
                    crate::battle_sim::game_mechanics::BattlerUID {
                        team_id: crate::battle_sim::game_mechanics::TeamID::Ally,
                        battler_number:
                            crate::battle_sim::game_mechanics::monster::BattlerNumber::First,
                    },
                    true,
                    crate::battle_sim::game_mechanics::monster::Monster::new(
                        crate::battle_sim::game_mechanics::monster_dex::Torchic,
                        "Ruby",
                    ),
                    crate::battle_sim::game_mechanics::move_::MoveSet::new([
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Scratch,
                        )),
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Ember,
                        )),
                        None,
                        None,
                    ]),
                    crate::battle_sim::game_mechanics::ability::Ability::new(
                        crate::battle_sim::game_mechanics::ability_dex::FlashFire,
                    ),
                )),
                Some(crate::battle_sim::game_mechanics::Battler::new(
                    crate::battle_sim::game_mechanics::BattlerUID {
                        team_id: crate::battle_sim::game_mechanics::TeamID::Ally,
                        battler_number:
                            crate::battle_sim::game_mechanics::monster::BattlerNumber::Second,
                    },
                    false,
                    crate::battle_sim::game_mechanics::monster::Monster::new(
                        crate::battle_sim::game_mechanics::monster_dex::Torchic,
                        "Sapphire",
                    ),
                    crate::battle_sim::game_mechanics::move_::MoveSet::new([
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Scratch,
                        )),
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Ember,
                        )),
                        None,
                        None,
                    ]),
                    crate::battle_sim::game_mechanics::ability::Ability::new(
                        crate::battle_sim::game_mechanics::ability_dex::FlashFire,
                    ),
                )),
                Some(crate::battle_sim::game_mechanics::Battler::new(
                    crate::battle_sim::game_mechanics::BattlerUID {
                        team_id: crate::battle_sim::game_mechanics::TeamID::Ally,
                        battler_number:
                            crate::battle_sim::game_mechanics::monster::BattlerNumber::Third,
                    },
                    false,
                    crate::battle_sim::game_mechanics::monster::Monster::new(
                        crate::battle_sim::game_mechanics::monster_dex::Torchic,
                        "Emerald",
                    ),
                    crate::battle_sim::game_mechanics::move_::MoveSet::new([
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Scratch,
                        )),
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Ember,
                        )),
                        None,
                        None,
                    ]),
                    crate::battle_sim::game_mechanics::ability::Ability::new(
                        crate::battle_sim::game_mechanics::ability_dex::FlashFire,
                    ),
                )),
                None,
                None,
                None,
            ]),
            crate::battle_sim::game_mechanics::MonsterTeam::new([
                Some(crate::battle_sim::game_mechanics::Battler::new(
                    crate::battle_sim::game_mechanics::BattlerUID {
                        team_id: crate::battle_sim::game_mechanics::TeamID::Opponent,
                        battler_number:
                            crate::battle_sim::game_mechanics::monster::BattlerNumber::First,
                    },
                    true,
                    crate::battle_sim::game_mechanics::monster::Monster::new(
                        crate::battle_sim::game_mechanics::monster_dex::Torchic,
                        "Cheerio",
                    ),
                    crate::battle_sim::game_mechanics::move_::MoveSet::new([
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Scratch,
                        )),
                        Some(crate::battle_sim::game_mechanics::move_::Move::new(
                            crate::battle_sim::game_mechanics::move_dex::Ember,
                        )),
                        None,
                        None,
                    ]),
                    crate::battle_sim::game_mechanics::ability::Ability::new(
                        crate::battle_sim::game_mechanics::ability_dex::FlashFire,
                    ),
                )),
                None,
                None,
                None,
                None,
                None,
            ]),
        )
    );
}

type BattleResult = Result<(), BattleError>;

pub struct Battle {
    context: BattleContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleError {
    WrongState(&'static str),
}

impl Battle {
    pub fn new( context: BattleContext ) -> Self {
        Battle { context, }
    }

    /// Main function for the simulator.
    pub fn simulate(&mut self) -> BattleResult {
        // Keep simulating turns until the battle is finished. TODO: User Input
        while self.context.state != BattleState::Finished {
            let result = self.simulate_turn();
            if result != Ok(()) { return result; }
        }
        Ok(())
    }

    fn simulate_turn(&mut self) -> BattleResult {
        let result = match self.context.state {
            BattleState::ChoosingActions => todo!(),
            BattleState::UsingMove { move_uid, target_uid } => {
                Action::damaging_move(&mut self.context, move_uid, target_uid)
            },
            BattleState::Finished => Err(BattleError::WrongState("simulate_turn was called in the BattleState::Finished state.")),
        };
        self.context.state = BattleState::Finished;
        result
    }
}