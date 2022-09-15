// FIXME: Unlock these to check for dead code from time to time.
#![allow(dead_code)]
#![allow(unused_imports)]

// Modules
mod utility_macros;
mod type_chart;
pub mod entities;
mod battle_constants;
pub mod battle_state;

// Usings
use crate::field;
use entities::{BattlerID, EventHandlerSet, MoveID};
use rand::Rng;
use type_chart::get_matchup;
use battle_constants::*;
use battle_state::BattleState;

pub struct Battle {
    battle_state: BattleState,
}

impl Battle {
    pub fn new(battle_state: BattleState) -> Self {
        Battle { battle_state }
    }

    pub fn run_sim(&mut self) {
        self.battle_state =
            Self::run_damaging_move(self.battle_state.clone(), 1, 2, MoveID::First);
    }

    pub fn run_damaging_move(
        b_state: BattleState,
        source_id: BattlerID,
        target_id: BattlerID,
        move_id: MoveID,
    ) -> BattleState {
        let (b_state, is_move_successful) =
            Self::run_event(b_state.clone(), field![on_try_move], true);

        if is_move_successful {
            println!(
                "{} used {}!",
                b_state.monster(source_id).nickname,
                b_state.move_(source_id, move_id).name
            );

            // Move accuracy calculation
            let mut move_accuracy = b_state.move_(source_id, move_id).base_accuracy;

            // Ignore the accuracy calculation if the move is a Never Miss move (denoted by an accuracy of 0u8)
            let mut did_move_hit = true;
            if move_accuracy != NEVER_MISS {
                let (b_state, source_accuracy_stages) = Self::run_event(
                    b_state.clone(),
                    field![on_move_calc_accuracy_stages],
                    b_state.monster(source_id).accuracy_stages,
                );
                let (mut b_state, target_evasion_stages) = Self::run_event(
                    b_state.clone(),
                    field![on_move_calc_evasion_stages],
                    b_state.monster(target_id).evasion_stages,
                );
                let overall_accuracy_stages =
                    num::clamp(source_accuracy_stages - target_evasion_stages, -6, 6);
                let accuracy_multiplier = ACCURACY_STAGE_TO_MULTIPLIER
                    .get(&overall_accuracy_stages)
                    .expect("E0001: accuracy stage lookup failed.");
                move_accuracy = ((move_accuracy as f32) * accuracy_multiplier) as u8;

                // Check if the move hits. To do this we roll a 100 sided die and check if it lands on a
                // number less than or equal to the move accuracy.
                let prng_roll = b_state.prng.gen_range::<u8, _>(1..=100);
                did_move_hit = prng_roll <= move_accuracy;
            }

            if did_move_hit {
                println!("The move hit!");

                // Calculating the effective type of the move being used.
                let (b_state, move_type) = Self::run_event(
                    b_state.clone(),
                    field![on_calc_move_type],
                    b_state.move_(source_id, move_id)._type,
                );

                // Calculating how much the type effectiveness affects the move.
                let (b_state, type_multiplier) = Self::run_event(
                    b_state.clone(),
                    field![on_calc_type_multiplier],
                    get_matchup(move_type, b_state.monster(target_id).primary_type)
                        * get_matchup(move_type, b_state.monster(target_id).secondary_type),
                );

                // The level of the monster using the move.
                let source_level = b_state.monster(source_id).level as u16;

                // The effective power of the move being used.
                let (b_state, move_power) = Self::run_event(
                    b_state.clone(),
                    field![on_calc_move_power],
                    b_state.move_(source_id, move_id).base_power,
                );

                // The attacking and defending stats of the move user and b_state.monster(target_id).
                let (b_state, source_attack) = Self::run_event(
                    b_state.clone(),
                    field![on_calc_attacking_stat],
                    b_state.monster(source_id).stats.att,
                );
                let (mut b_state, target_defense) = Self::run_event(
                    b_state.clone(),
                    field![on_calc_defending_stat],
                    b_state.monster(source_id).stats.def,
                );

                // The move's power is weakened when using spread moves.
                // TODO: Add the 0.85x multiplier for multiple targets when that becomes a thing.
                let targets_multiplier = 1.0f32;

                // The damage multiplier due to weather effects.
                let weather_multiplier = 1.0f32;

                // A random damage multiplier from 0.85x to 1.00x inclusive.
                let random_multiplier = b_state.prng.gen_range(85..=100) as f32 / 100.0;

                // The damage multiplier due to the move user being the same type as the move being used.
                #[allow(non_snake_case)]
                let STAB_multiplier = {
                    if move_type == b_state.monster(source_id).primary_type
                        || move_type == b_state.monster(source_id).secondary_type
                    {
                        1.5f32
                    } else {
                        1.0f32
                    }
                };

                // The damage multiplier due to a critical hit.
                // TODO: There should be a few crit events to decide how the crit goes.
                let (mut b_state, critical_hit_multiplier) =
                    Self::run_event(b_state.clone(), field![on_calc_crit_multiplier], 1.0f32);

                // Apply the damage formula to calculate potential dealt, we only bother calculating it if the move is effective.
                let mut damage: u16 = 0;
                if type_multiplier != INEFFECTIVE {
                    damage = 2 * source_level / 5;
                    damage += 2;
                    damage *= move_power;
                    damage = (damage as f32 * source_attack as f32 / target_defense as f32) as u16;
                    damage /= 50;
                    damage += 2;
                    damage = (damage as f32 * targets_multiplier) as u16;
                    damage = (damage as f32 * weather_multiplier) as u16;
                    damage = (damage as f32 * critical_hit_multiplier) as u16;
                    damage = (damage as f32 * random_multiplier) as u16;
                    damage = (damage as f32 * STAB_multiplier) as u16;
                    damage = (damage as f32 * type_multiplier) as u16;
                    damage = std::cmp::max(damage, MIN_POSSIBLE_DAMAGE);
                }

                Self::do_damage(damage, &mut b_state, target_id);
                // Matching type multiplier as an integer from 1-100 because apparently
                // float matches are being deprecated.
                match (type_multiplier * 100.0) as u16 {
                    0 => println!("The move had no effect!"),
                    25 => println!("The move was not very effective..."),
                    50 => println!("The move was not very effective..."),
                    200 => println!("The move was super effective!"),
                    400 => println!("The move was super effective!"),
                    _ => (),
                }
                return b_state;
            } else {
                println!("But the move missed!");
                return b_state;
            }
        } else {
            println!("The move failed!");
            return b_state;
        }
    }

    fn do_damage(damage: u16, b_state: &mut BattleState, target_id: BattlerID) {
        b_state.monster_mut(target_id).current_health -= damage;
        println!(
            "{} took {} damage!\n{} has {} hp left.",
            b_state.monster(target_id).nickname,
            damage,
            b_state.monster(target_id).nickname,
            b_state.monster(target_id).current_health
        );
    }

    pub fn run_event<R>(
        battle_state: BattleState,
        field: fn(EventHandlerSet) -> Option<fn() -> R>,
        default: R,
    ) -> (BattleState, R) {
        let mut relay = default;
        //TODO: Implement EventHandler Ordering.
        let event_handlers = battle_state.get_event_handlers::<R>(field);

        for event_handler in event_handlers.iter() {
            relay = event_handler();
        }

        (battle_state, relay)
    }
}

