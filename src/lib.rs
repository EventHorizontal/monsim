mod action;
pub mod battle_context;
mod event;
pub mod game_mechanics;
pub mod global_constants;
pub mod io;
mod prng;

pub use action::*;
pub use battle_context::*;
pub use event::*;
pub use game_mechanics::*;
pub use global_constants::*;
pub use io::*;
use prng::LCRNG;

pub use battle_context::BattleContext;
pub use bcontext_macro::bcontext;
#[allow(unused_imports)]
use bcontext_macro::bcontext_internal;

#[test]
fn test_bcontext_macro() {
    let test_bcontext = bcontext_internal!(
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
            crate::game_mechanics::BattlerTeam::new(
                vec![
                        (crate::game_mechanics::Battler::new(
                            crate::game_mechanics::BattlerUID {
                                team_id: crate::game_mechanics::TeamID::Ally,
                                battler_number: crate::game_mechanics::monster::BattlerNumber::First,
                            },
                            true,
                            crate::game_mechanics::monster::Monster::new(
                                crate::game_mechanics::monster_dex::Torchic,
                                "Ruby",
                            ),
                            crate::game_mechanics::move_::MoveSet::new(
                                vec![
                                    (crate::game_mechanics::move_::Move::new(
                                        crate::game_mechanics::move_dex::Scratch,
                                    )),
                                    (crate::game_mechanics::move_::Move::new(
                                        crate::game_mechanics::move_dex::Ember,
                                    )),
                                ],
                            ),
                            crate::game_mechanics::ability::Ability::new(
                                crate::game_mechanics::ability_dex::FlashFire,
                            ),
                        )),
                        (crate::game_mechanics::Battler::new(
                            crate::game_mechanics::BattlerUID {
                                team_id: crate::game_mechanics::TeamID::Ally,
                                battler_number: crate::game_mechanics::monster::BattlerNumber::Second,
                            },
                            false,
                            crate::game_mechanics::monster::Monster::new(
                                crate::game_mechanics::monster_dex::Torchic,
                                "Sapphire",
                            ),
                            crate::game_mechanics::move_::MoveSet::new(
                                vec![
                                    (crate::game_mechanics::move_::Move::new(
                                        crate::game_mechanics::move_dex::Scratch,
                                    )),
                                    (crate::game_mechanics::move_::Move::new(
                                        crate::game_mechanics::move_dex::Ember,
                                    )),
                                ],
                            ),
                            crate::game_mechanics::ability::Ability::new(
                                crate::game_mechanics::ability_dex::FlashFire,
                            ),
                        )),
                        (crate::game_mechanics::Battler::new(
                            crate::game_mechanics::BattlerUID {
                                team_id: crate::game_mechanics::TeamID::Ally,
                                battler_number: crate::game_mechanics::monster::BattlerNumber::Third,
                            },
                            false,
                            crate::game_mechanics::monster::Monster::new(
                                crate::game_mechanics::monster_dex::Torchic,
                                "Emerald",
                            ),
                            crate::game_mechanics::move_::MoveSet::new(
                                vec![
                                    (crate::game_mechanics::move_::Move::new(
                                        crate::game_mechanics::move_dex::Scratch,
                                    )),
                                    (crate::game_mechanics::move_::Move::new(
                                        crate::game_mechanics::move_dex::Ember,
                                    )),
                                ],
                            ),
                            crate::game_mechanics::ability::Ability::new(
                                crate::game_mechanics::ability_dex::FlashFire,
                            ),
                        )),
                    ],
                
            ),
            crate::game_mechanics::BattlerTeam::new(
                vec![
                    (crate::game_mechanics::Battler::new(
                        crate::game_mechanics::BattlerUID {
                            team_id: crate::game_mechanics::TeamID::Opponent,
                            battler_number: crate::game_mechanics::monster::BattlerNumber::First,
                        },
                        true,
                        crate::game_mechanics::monster::Monster::new(
                            crate::game_mechanics::monster_dex::Torchic,
                            "Cheerio",
                        ),
                        crate::game_mechanics::move_::MoveSet::new(
                            vec![
                                (crate::game_mechanics::move_::Move::new(
                                    crate::game_mechanics::move_dex::Scratch,
                                )),
                                (crate::game_mechanics::move_::Move::new(
                                    crate::game_mechanics::move_dex::Ember,
                                )),
                            ],
                        ),
                        crate::game_mechanics::ability::Ability::new(
                            crate::game_mechanics::ability_dex::FlashFire,
                        ),
                    ))
                ],
            ),
        ),
    );
}

type BattleResult = Result<(), BattleError>;

#[derive(Debug)]
pub struct Battle {
    pub context: BattleContext,
}

impl Battle {
    pub fn new(context: BattleContext) -> Self {
        Battle { context }
    }

    pub fn simulate_turn(&mut self, user_input: UserInput) -> BattleResult {
        let mut result = Ok(());
        let mut action_choices = user_input.choices();
        {
            // TODO: We need to revamp the BattleContext so that we can send it smaller chunks of info as/when it
            // needs to read/write and so we can split borrows here.
            let battle_context: BattleContext = self.context.clone();
            Battle::priority_sort(
                &mut self.context.prng,
                &mut action_choices,
                &mut |it| battle_context.choice_activation_order(it),
            );
        }
        for action_choice in action_choices.into_iter() {
            self.context.current_action = action_choice;
            result = match action_choice {
                ActionChoice::Move {
                    move_uid,
                    target_uid,
                } => Action::damaging_move(&mut self.context, move_uid, target_uid),
                ActionChoice::None => {
                    Err(BattleError::WrongState("No action was taken by a Monster."))
                }
            };
            // Check if any monster fainted due to the last action.
            if let Some(battler) = self.context.battlers().find(|it| it.fainted()) {
                self.context.message_buffer.push(format!["{} fainted!", battler.monster.nickname]);
                self.context.state = BattleState::Finished;
                break;
            };
        }
        self.context.message_buffer.push("\n-------------------------------------\n".to_string());

        result
    }

    /// Sorts the given items using their associated ActivationOrders, resolving speed ties using `prng` after stable sorting.
    pub(crate) fn priority_sort<T: Clone + Copy>(
        prng: &mut LCRNG,
        vector: &mut Vec<T>,
        activation_order: &mut dyn FnMut(T) -> ActivationOrder,
    ) {
        // Sort without resolving speed ties, this sorting is stable, so it doesn't affect the order of condition-wise equal elements.
        vector.sort_by(|a, b| activation_order(*a).cmp(&activation_order(*b)));
        // Sorting is ascending, but we want descending sorting, so reverse the vector.
        vector.reverse();

        let vector_length = vector.len();
        match vector_length.cmp(&2) {
            std::cmp::Ordering::Less => (),
            std::cmp::Ordering::Equal => {
                let previous_item = activation_order(vector[0]);
                let this_item = activation_order(vector[1]);
                if this_item == previous_item {
                    Self::resolve_speed_tie::<T>(prng, vector, &mut vec![0, 1]);
                }
            }
            std::cmp::Ordering::Greater => {
                let mut tied_monster_indices: Vec<usize> = vec![0];
                // If there are more than two items, iterated through the 2nd through last index of the vector, comparing each item to the previous one.
                for i in 1..vector_length {
                    let previous_item = activation_order(vector[i - 1]);
                    let this_item = activation_order(vector[i]);
                    // If the item we are looking at has the same speed as the previous, add its index to the tied queue.
                    if previous_item == this_item {
                        tied_monster_indices.push(i);
                        if i == (vector_length - 1) {
                            Self::resolve_speed_tie::<T>(prng, vector, &mut tied_monster_indices);
                        }
                    // If the priority or speed of the last item is higher, sort the current tied items using the PRNG and then reset the tied queue.
                    } else if previous_item > this_item {
                        Self::resolve_speed_tie::<T>(prng, vector, &mut tied_monster_indices);
                        tied_monster_indices = vec![i];
                    }
                }
            }
        }
    }

    /// Shuffles the event handler order for consecutive speed-tied items in place using their associated activation orders.
    fn resolve_speed_tie<T: Clone + Copy>(
        prng: &mut LCRNG,
        vector: &mut Vec<T>,
        tied_monster_indices: &mut Vec<usize>,
    ) {
        if tied_monster_indices.len() < 2 {
            return;
        }
        let mut i: usize = 0;
        let vector_copy = vector.clone();
        let offset = tied_monster_indices[0];
        'iteration_over_tied_indices: while tied_monster_indices.len() > 0 {
            let number_tied = tied_monster_indices.len() as u16;
            // Roll an n-sided die and put the monster corresponding to the roll at the front of the tied order.
            let prng_roll = prng.generate_number_in_range(0..=number_tied - 1) as usize;
            vector[i + offset] = vector_copy[tied_monster_indices.remove(prng_roll)];
            // Once there is only one remaining tied monster, put it at the end of the queue.
            if tied_monster_indices.len() == 1 {
                vector[i + offset + 1] = vector_copy[tied_monster_indices[0]];
                break 'iteration_over_tied_indices;
            }
            i += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleError {
    WrongState(&'static str),
    InputError(String),
}
