pub mod battle_context;
pub mod game_mechanics;
pub mod global_constants;
pub mod io;

mod action;
mod event;
mod prng;
mod test;

pub use action::*;
pub use battle_context::*;
pub use event::*;
pub use game_mechanics::*;
pub use global_constants::*;
pub use io::*;
use prng::Prng;

pub use battle_context::BattleContext;
pub use bcontext_macro::bcontext;
#[allow(unused_imports)]
use bcontext_macro::bcontext_internal;

type TurnOutcome = Result<(), SimError>;

#[derive(Debug)]
pub struct Battle {
    pub ctx: BattleContext,
    pub prng: Prng,
}

impl Battle {
    pub fn new(ctx: BattleContext) -> Self {
        Battle {
            ctx,
            prng: Prng::new(prng::seed_from_time_now()),
        }
    }

    pub fn simulate_turn(&mut self, mut chosen_actions: ChosenActions) -> TurnOutcome {
        let mut result = Ok(());

        Battle::priority_sort(&mut self.prng, &mut chosen_actions, &mut |choice| {
            self.ctx.choice_activation_order(choice)
        });

        for chosen_action in chosen_actions.into_iter() {
            self.ctx.current_action = Some(chosen_action);
            result = match chosen_action {
                ActionChoice::Move {
                    move_uid,
                    target_uid,
                } => match self.ctx.move_(move_uid).category() {
                    MoveCategory::Physical | MoveCategory::Special => PrimaryAction::damaging_move(
                        &mut self.ctx,
                        &mut self.prng,
                        move_uid,
                        target_uid,
                    ),
                    MoveCategory::Status => PrimaryAction::status_move(
                        &mut self.ctx,
                        &mut self.prng,
                        move_uid,
                        target_uid,
                    ),
                },
            };
            // Check if any monster fainted due to the last action.
            if let Some(battler) = self.ctx.battlers().find(|it| it.fainted()) {
                self.ctx
                    .push_message(&format!["{} fainted!", battler.monster.nickname]);
                self.ctx.state = BattleState::Finished;
                break;
            };
            self.ctx.push_message(&EMPTY_LINE);
        }

        result
    }

    /// Sorts the given items using their associated ActivationOrders, resolving speed ties using `prng` after stable sorting.
    pub(crate) fn priority_sort<T: Clone + Copy>(
        prng: &mut Prng,
        vector: &mut Vec<T>,
        activation_order: &mut dyn FnMut(T) -> ActivationOrder,
    ) {
        // Sort without resolving speed ties, this sorting is stable, so it doesn't affect the order of condition-wise equal elements.
        vector.sort_by_key(|a| activation_order(*a));
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
                    use std::cmp::Ordering::{Equal, Greater, Less};
                    match previous_item.cmp(&this_item) {
                        Equal => {
                            tied_monster_indices.push(i);
                            if i == (vector_length - 1) {
                                Self::resolve_speed_tie::<T>(
                                    prng,
                                    vector,
                                    &mut tied_monster_indices,
                                );
                            }
                        }
                        Greater => {
                            Self::resolve_speed_tie::<T>(prng, vector, &mut tied_monster_indices);
                            tied_monster_indices = vec![i];
                        }
                        Less => unreachable!(),
                    }
                }
            }
        }
    }

    /// Shuffles the event handler order for consecutive speed-tied items in place using their associated activation orders.
    fn resolve_speed_tie<T: Clone + Copy>(
        prng: &mut Prng,
        vector: &mut [T],
        tied_monster_indices: &mut Vec<usize>,
    ) {
        if tied_monster_indices.len() < 2 {
            return;
        }
        let mut i: usize = 0;
        let vector_copy = vector.to_owned();
        let offset = tied_monster_indices[0];
        'iteration_over_tied_indices: while !tied_monster_indices.is_empty() {
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
pub enum SimError {
    InvalidStateError(&'static str),
    InputError(String),
}
