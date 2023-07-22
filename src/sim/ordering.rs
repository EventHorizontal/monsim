use crate::sim::{ActivationOrder, Prng};

use super::{ActionChoice, Battle};

/// Sorts the given items using their associated ActivationOrders, resolving speed ties using `prng` after stable sorting.
pub(crate) fn sort_by_activation_order<T: Clone + Copy>(prng: &mut Prng, vector: &mut Vec<T>, activation_order: &mut dyn FnMut(T) -> ActivationOrder) {
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
                resolve_speed_tie::<T>(prng, vector, &mut vec![0, 1]);
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
                            resolve_speed_tie::<T>(prng, vector, &mut tied_monster_indices);
                        }
                    }
                    Greater => {
                        resolve_speed_tie::<T>(prng, vector, &mut tied_monster_indices);
                        tied_monster_indices = vec![i];
                    }
                    Less => unreachable!(),
                }
            }
        }
    }
}

pub(crate) fn context_sensitive_sort_by_activation_order(battle: &mut Battle, vector: &mut Vec<ActionChoice>) {
    // Sort without resolving speed ties, this sorting is stable, so it doesn't affect the order of condition-wise equal elements.
    vector.sort_by_key(|choice| battle.choice_activation_order(*choice));
    // Sorting is ascending, but we want descending sorting, so reverse the vector.
    vector.reverse();

    let vector_length = vector.len();
    match vector_length.cmp(&2) {
        std::cmp::Ordering::Less => (),
        std::cmp::Ordering::Equal => {
            let previous_item = battle.choice_activation_order(vector[0]);
            let this_item = battle.choice_activation_order(vector[1]);
            if this_item == previous_item {
                resolve_speed_tie(&mut battle.prng, vector, &mut vec![0, 1]);
            }
        }
        std::cmp::Ordering::Greater => {
            let mut tied_monster_indices: Vec<usize> = vec![0];
            // If there are more than two items, iterated through the 2nd through last index of the vector, comparing each item to the previous one.
            for i in 1..vector_length {
                let previous_item = battle.choice_activation_order(vector[i - 1]);
                let this_item = battle.choice_activation_order(vector[i]);
                // If the item we are looking at has the same speed as the previous, add its index to the tied queue.
                use std::cmp::Ordering::{Equal, Greater, Less};
                match previous_item.cmp(&this_item) {
                    Equal => {
                        tied_monster_indices.push(i);
                        if i == (vector_length - 1) {
                            resolve_speed_tie(&mut battle.prng, vector, &mut tied_monster_indices);
                        }
                    }
                    Greater => {
                        resolve_speed_tie(&mut battle.prng, vector, &mut tied_monster_indices);
                        tied_monster_indices = vec![i];
                    }
                    Less => unreachable!(),
                }
            }
        }
    }
}

use crate::sim::not;
/// Shuffles the event responder order for consecutive speed-tied items in place using their associated activation orders.
fn resolve_speed_tie<T: Clone + Copy>(prng: &mut Prng, vector: &mut [T], tied_monster_indices: &mut Vec<usize>) {
    if tied_monster_indices.len() < 2 {
        return;
    }
    let mut i = 0usize;
    let vector_copy = vector.to_owned();
    let offset = tied_monster_indices[0];
    'iteration_over_tied_indices: while not!(tied_monster_indices.is_empty()) {
        let number_tied = tied_monster_indices.len() as u16;
        // Roll an n-sided die and put the monster corresponding to the roll at the front of the tied order.
        let prng_roll = prng.generate_u16_in_range(0..=number_tied - 1) as usize;
        vector[i + offset] = vector_copy[tied_monster_indices.remove(prng_roll)];
        // Once there is only one remaining tied monster, put it at the end of the queue.
        if tied_monster_indices.len() == 1 {
            vector[i + offset + 1] = vector_copy[tied_monster_indices[0]];
            break 'iteration_over_tied_indices;
        }
        i += 1;
    }
}
