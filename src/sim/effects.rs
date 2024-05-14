use monsim_macros::{abl, mon, mov};
use crate::{events, matchup};
use self::targetting::BoardPosition;
use super::*;

/// `R`: A type that encodes any necessary information about how the `Effect` played
/// out, _e.g._ an `Outcome` representing whether the `Effect` succeeded.
///
/// `C`: Any information necessary for the resolution of the effect, provided 
/// directly, such as the user of the move, the move used and the target 
/// in case of a move's effect. 
pub type Effect<R,C> = fn(/* simulator */ &mut BattleSimulator, /* effector id, i.e. the Monster doing the effect */ MonsterID, /* context */ C) -> R;

// internal `Effects` that are only supposed to be used by the engine -----------------------------------------

/// The simulator simulates the use of a move `MoveUseContext.move_used` by 
/// `MoveUseContext.move_user` on `MoveUseContext.target`.
pub fn use_move(sim: &mut BattleSimulator, effector_id: MonsterID, context: MoveUseContext) {
    let MoveUseContext { move_user_id, move_used_id, target_ids } = context;
    assert!(mov![move_used_id].current_power_points > 0, "A move was used that had zero power points");
    
    sim.push_message(format![
        "{attacker} used {move_}",
        attacker = mon![move_user_id].name(),
        move_ = mov![move_used_id].name()
    ]);
    
    // There are no remaining targets for this move. They fainted before the move was used.
    if target_ids.is_empty() {
        sim.push_message(
            format!["{}'s {} has no targets...", mon![move_user_id].name(), mov![move_used_id].name()]
        );
        return;
    }
    
    let try_use_move_outcome = events::trigger_on_try_move_event(sim, move_user_id, context);
    if try_use_move_outcome.failed() {
        sim.push_message("The move failed!");
        return;
    }

    /*
    INFO: We are currently dealing with the fact moves hit 0-6 times by thinking
    of them as having an `on_hit_effect` rather an `on_use_effect`. We may need
    rethink this if it turns out some moves do want to control their `on_use_effect`.
    Event multihit moves seem like they want to just have a `number_of_hits` variable
    and just repeat its on_hit_effect 
    */
    for target_id in target_ids {
        let subcontext = MoveHitContext { move_user_id, move_used_id, target_id };
        let mut actual_number_of_hits = 0;
        for _ in {
            match mov![move_used_id].hits_per_target() {
                Hits::Once => 1..=1,
                Hits::MultipleTimes(number_of_hits) => 1..=number_of_hits,
                Hits::RandomlyInRange { min, max } => {
                    let number_of_hits = sim.generate_random_number_in_range_inclusive(min as u16..=max as u16);
                    1..=number_of_hits as u8
                },
            }
        } {
            mov![move_used_id].on_hit_effect()(&mut *sim, effector_id, subcontext);
            actual_number_of_hits += 1;
            if mon![target_id].is_fainted() {
                sim.push_message(format!["{} fainted!", mon![target_id].name()]);
                break;
            }
        } 

        if matches!(mov![move_used_id].hits_per_target(), Hits::MultipleTimes(_) | Hits::RandomlyInRange { min: _, max: _ }) {
            sim.push_message(format!["The move hit {} time(s)", actual_number_of_hits]);
        }
    }
    
    mov![mut move_used_id].current_power_points -= 1;
    
    #[cfg(feature="debug")]
    sim.push_message(format![
        "{}'s {}'s PP is now {}",
        mon![move_user_id].name(),
        mov![move_used_id].name(),
        mov![move_used_id].current_power_points()
    ]);

    match mov![move_used_id].category() {
        MoveCategory::Physical | MoveCategory::Special => {
            events::trigger_on_damaging_move_used_event(sim, move_user_id, context);
        },
        MoveCategory::Status => {
            events::trigger_on_status_move_used_event(sim, move_user_id, context)
        }
    }
}

/// The simulator switches out the Monster given by `context.active_monster_id` and switches in 
/// the Monster given by `context.benched_monster_id`
pub(crate) fn switch_monsters(sim: &mut BattleSimulator, _effector_id: MonsterID, context: SwitchContext) {
    let SwitchContext { active_monster_id, benched_monster_id } = context;

    // Swap board positions of the two Monsters. (We just assume benched_monster_id corresponds to a benched monster at this point).
    mon![mut benched_monster_id].board_position = mon![active_monster_id].board_position;
    mon![mut active_monster_id].board_position = BoardPosition::Bench; 
    
    sim.push_message(format![
        "{} switched out! Go {}!", 
        mon![active_monster_id].name(),
        mon![benched_monster_id].name()
    ]);
}

/// The simulator switchees in the Monster given by `context.0` into a presumed empty field position given by `context.1`. The caller is expected
/// to check that the field position is indeed empty. 
pub(crate) fn switch_in_monster(sim: &mut BattleSimulator, _effector_id: MonsterID, (benched_monster_id, field_position): (MonsterID, FieldPosition)) {
    mon![mut benched_monster_id].board_position = BoardPosition::Field(field_position);
    sim.push_message(format![
        "Go {}!",
        mon![benched_monster_id].name()
    ]);
}

// public `Effects` usable by users of the crate. ------------------------------------------

/// The simulator simulates dealing damage of a move given by `MoveUseContext.move_used` by 
/// `MoveUseContext.move_user` on `MoveUseContext.target` using the default damage formula.
/// 
/// This is done by calculating the damage first using the formula then calling `deal_direct_damage`
/// with the resulting damage.

pub fn deal_default_damage(sim: &mut BattleSimulator, effector_id: MonsterID, context: MoveHitContext) {
    let MoveHitContext { move_user_id: attacker_id, move_used_id, target_id: defender_id } = context;

    let try_move_hit_outcome = events::trigger_on_try_move_hit_event(sim, attacker_id, context);
    if try_move_hit_outcome.failed() {
        sim.push_message(format!["The move failed to hit {}!", mon![defender_id].name()]);
        return;
    }

    let level = mon![attacker_id].level;
    let move_power = mov![move_used_id].base_power();

    let (attackers_attacking_stat, defenders_defense_stat) = match mov![move_used_id].category() {
        MoveCategory::Physical => {
            (
                mon![attacker_id].stat(Stat::PhysicalAttack),
                mon![defender_id].stat(Stat::PhysicalDefense)
            )
        }
        MoveCategory::Special => {
            (
                mon![attacker_id].stat(Stat::SpecialAttack),
                mon![defender_id].stat(Stat::SpecialDefense)
            )
        }
        _ => unreachable!("Expected physical or special move."),
    };

    let random_multiplier = sim.generate_random_number_in_range_inclusive(85..=100);
    let random_multiplier = ClampedPercent::from(random_multiplier);

    let stab_multiplier = {
        let move_type = mov![move_used_id].type_();
        if mon![attacker_id].is_type(move_type) { Percent(125) } else { Percent(100) }
    };

    let move_type = mov![move_used_id].type_();
    let target_primary_type = mon![defender_id].species.primary_type();
    let target_secondary_type = mon![defender_id].species.secondary_type();

    let type_matchup_multiplier = if let Some(target_secondary_type) = target_secondary_type {
        matchup!(move_type against target_primary_type / target_secondary_type)
    } else {
        matchup!(move_type against target_primary_type)
    };

    // If the opponent is immune, damage calculation is skipped.
    if type_matchup_multiplier.is_matchup_ineffective() {
        sim.push_message("It was ineffective...");
        return;
    }

    // The (WIP) bona-fide damage formula.
    let mut damage = (2 * level) / 5;
    damage += 2;
    damage *= move_power;
    damage = (damage as f64 * (attackers_attacking_stat as f64 / defenders_defense_stat as f64)) as u16;
    damage /= 50;
    damage += 2;
    damage = (damage as f64 * random_multiplier) as u16;
    damage = (damage as f64 * stab_multiplier) as u16;
    damage = (damage as f64 * type_matchup_multiplier) as u16;
    // TODO: Introduce more damage multipliers as we implement them.

    // Do the calculated damage to the target
    let _ = deal_direct_damge(sim, effector_id, (defender_id, damage));

    let type_effectiveness = match type_matchup_multiplier {
        Percent(25) | Percent(50) => "not very effective",
        Percent(100) => "effective",
        Percent(200) | Percent(400) => "super effective",
        value => {
            let type_multiplier_as_float = value.0 as f64 / 100.0f64;
            unreachable!("Type Effectiveness Multiplier is unexpectedly {type_multiplier_as_float}")
        }
    };
    /*
    TODO: I was wondering if I could move the "it was __ effective" text out to `use_move` but I encountered two
    problems:
        1. type_effectiveness only exists within this context so we in order to 
        push the message in `use_move` we would at the very least need to return
        it if not recalculate it. It also wouldn't make sense in all contexts (such
        as if the move were a status move).
        2. we do need in general to be able to display messages at the _end_ of all 
        the hits of a move, and upon thinking about it for 5 minutes I realise we 
        need to have an `on_use` and an `on_hit` (they should have reasonable defaults 
        so that most of the time they don't need to be re-implemented). 
    Taking this stuff inot account, we need a better separation of "move use" and "move hit" if not just
    so that one can do something after (and before) of all of the hits.
    */
    sim.push_message(format!["It was {type_effectiveness}!"]);
    sim.push_message(format![
        "{} took {damage} damage!", 
        mon![defender_id].name()
    ]);
    sim.push_message(format![
        "{} has {num_hp} health left.",
        mon![defender_id].name(),
        num_hp = mon![defender_id].current_health
    ]);
}

/// The simulator simulates dealing damage equalling `Context.1` to the target `Context.0`.
/// 
/// Returns the actual damage dealt.

#[must_use]
fn deal_direct_damge(sim: &mut BattleSimulator, effector_id: MonsterID, context: (MonsterID, u16)) -> u16 {
    let (target_id, mut damage) = context;
    let original_health = mon![target_id].current_health;
    mon![mut target_id].current_health = original_health.saturating_sub(damage);
    if mon![target_id].is_fainted() { 
        damage = original_health;
        mon![mut target_id].board_position = BoardPosition::Bench;
    };
    events::trigger_on_damage_dealt_event(sim, effector_id, NOTHING);
    damage
}

/// The simulator simulates the activation of the ability `AbilityUseContext.ability_used` owned by
/// the monster `AbilityUseContext.abilty_owner`.
#[must_use]
pub fn activate_ability(sim: &mut BattleSimulator, effector_id: MonsterID, context: AbilityUseContext) -> Outcome {
    let AbilityUseContext { ability_used_id, ability_owner_id } = context;

    let try_activate_ability_outcome = events::trigger_on_try_activate_ability_event(sim, ability_owner_id, context);
    if try_activate_ability_outcome.succeeded() {
        let ability = abl![ability_used_id];
        (ability.on_activate_effect())(sim, effector_id, context);
        events::trigger_on_ability_activated_event(sim, ability_owner_id, context);
        Outcome::Success
    } else {
        Outcome::Failure
    }
}

/// The simulator simulates the raising of stat `Context.1` of monster `Context.0` by `Context.2` stages
#[must_use]
pub fn raise_stat(
    sim: &mut BattleSimulator,
    _effector_id: MonsterID,
    (affected_monster_id, stat, number_of_stages): (MonsterID, Stat, u8), 
) -> Outcome {
    let try_raise_stat_outcome = events::trigger_on_try_raise_stat_event(sim, affected_monster_id, NOTHING);
    if try_raise_stat_outcome.succeeded() {
        let effective_stages = mon![mut affected_monster_id].stat_modifiers.raise_stat(stat, number_of_stages);

        sim.push_message(format![
            "{monster}\'s {stat} was raised by {effective_stages} stage(s)!",
            monster = mon![affected_monster_id].name(),
        ]);

        Outcome::Success
    } else {
        sim.push_message(format!["{monster}'s stats cannot get any higher.", monster = mon![affected_monster_id].name()]);

        Outcome::Failure
    }
}

/// The simulator simulates the lowering of stat `Context.1` of monster `Context.0` by `Context.2` stages
#[must_use]
pub fn lower_stat(
    sim: &mut BattleSimulator,
    _effector_id: MonsterID,
    (affected_monster_id, stat, number_of_stages): (MonsterID, Stat, u8), 
) -> Outcome {
    let try_lower_stat_outcome = events::trigger_on_try_lower_stat_event(sim, affected_monster_id, NOTHING);
    if try_lower_stat_outcome.succeeded() {
        let effective_stages = mon![mut affected_monster_id].stat_modifiers.lower_stat(stat, number_of_stages);

        sim.push_message(format![
            "{monster}\'s {stat} was lowered by {effective_stages} stage(s)!",
            monster = mon![affected_monster_id].name(),
        ]);

        Outcome::Success
    } else {
        sim.push_message(format!["{monster}'s stats were not lowered.", monster = mon![affected_monster_id].name()]);

        Outcome::Failure
    }
}