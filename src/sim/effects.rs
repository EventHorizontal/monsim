use monsim_macros::{abl, mon, mov};
use crate::events;
use self::{status::{PersistentStatus, VolatileStatus, VolatileStatusSpecies}, targetting::BoardPosition};
use super::*;

/// `R`: A type that encodes any necessary information about how the `Effect` played
/// out, _e.g._ an `Outcome` representing whether the `Effect` succeeded.
///
/// `C`: Any information necessary for the resolution of the effect, provided 
/// directly, such as the user of the move, the move used and the target 
/// in case of a move's effect. 
pub type Effect<R,C> = fn(/* simulator */ &mut BattleSimulator, /* context */ C) -> R;

// internal `Effects` that are only supposed to be used by the engine -----------------------------------------

/// The simulator simulates the use of a move `MoveUseContext.move_used` by 
/// `MoveUseContext.move_user` on `MoveUseContext.target`.
pub fn use_move(sim: &mut BattleSimulator, context: MoveUseContext) {
    let MoveUseContext { move_user_id, move_used_id, target_ids } = context;
    assert!(mov![move_used_id].current_power_points > 0, "A move was used that had zero power points");
    
    let try_use_move_outcome = events::trigger_on_try_move_event(sim, move_user_id, context);
    if try_use_move_outcome.failed() {
        sim.push_message("The move failed!");
        return;
    }
   
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
                Count::Fixed(number_of_hits) => 1..=number_of_hits,
                Count::RandomInRange { min, max } => {
                    let number_of_hits = sim.generate_random_number_in_range_inclusive(min as u16..=max as u16);
                    1..=number_of_hits as u8
                },
            }
        } {
            let move_hit_outcome = mov![move_used_id].on_hit_effect()(sim, subcontext);
            if move_hit_outcome.succeeded() {
                actual_number_of_hits += 1;  
            }
            if mon![target_id].is_fainted() {
                break;
            }
        } 

        if actual_number_of_hits == 0 {
            match target_ids.count() {
                1 => {
                    sim.push_message("But the move failed!")
                },
                2.. => {
                    sim.push_message(format!["But the move failed on {}!", sim.battle.monster(target_id).name()]);
                }
                _ => {}
            } 
        }

        match mov![move_used_id].hits_per_target() {
            Count::Fixed(n) if n > 1 => {
                sim.push_message(format!["The move hit {} time(s)", n]);
            },
            Count::RandomInRange { min: _, max: _ } => {
                sim.push_message(format!["The move hit {} time(s)", actual_number_of_hits]);
            },
            _ => {}
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
pub(crate) fn switch_monsters(sim: &mut BattleSimulator, context: SwitchContext) {
    let SwitchContext { active_monster_id, benched_monster_id } = context;
    let active_monster_field_position = mon![active_monster_id].field_position()
        .expect("Expected the monster to be switched out to be on the field.");
    switch_out_monster(sim, active_monster_id);
    switch_in_monster(sim, (benched_monster_id, active_monster_field_position));
}

/// The simulator switchees in the Monster given by `context.0` into a presumed empty field position given by `context.1`. The caller is expected
/// to check that the field position is indeed empty. 
pub(crate) fn switch_in_monster(sim: &mut BattleSimulator, (benched_monster_id, field_position): (MonsterID, FieldPosition)) {
    mon![mut benched_monster_id].board_position = BoardPosition::Field(field_position);
    sim.push_message(format![
        "Go {}!",
        mon![benched_monster_id].name()
    ]);
}

pub(crate) fn switch_out_monster(sim: &mut BattleSimulator, active_monster_id: MonsterID) {
    
    let active_monster = mon![mut active_monster_id];
    active_monster.board_position = BoardPosition::Bench; 
    active_monster.volatile_statuses = MaxSizedVec::empty();
    
    sim.push_message(format![
        "Come back {}!",
        mon![active_monster_id].name()
    ]);
}

// public `Effects` usable by users of the crate. ------------------------------------------

/// The simulator simulates dealing damage of a move given by `MoveUseContext.move_used` by 
/// `MoveUseContext.move_user` on `MoveUseContext.target` using the default damage formula.
/// 
/// This is done by calculating the damage first using the formula then calling `deal_direct_damage`
/// with the resulting damage.
/// 
/// Returns an `Outcome` signifying whether the move succeeded.

pub fn deal_default_damage(sim: &mut BattleSimulator, context: MoveHitContext) -> Outcome<Nothing> {
    let MoveHitContext { move_user_id: attacker_id, move_used_id, target_id: defender_id } = context;

    let try_move_hit_outcome = events::trigger_on_try_move_hit_event(sim, attacker_id, context);
    if try_move_hit_outcome.failed() {
        sim.push_message(format!["The move failed to hit {}!", mon![defender_id].name()]);
        return Outcome::Failure;
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

    let attackers_attacking_stat = trigger_on_calculate_attack_stat_event(sim, attacker_id, context, attackers_attacking_stat);
    let defenders_defense_stat = trigger_on_calculate_defense_stat_event(sim, attacker_id, context, defenders_defense_stat);

    let random_multiplier = sim.generate_random_number_in_range_inclusive(85..=100);
    let random_multiplier = ClampedPercent::from(random_multiplier);

    let stab_multiplier = {
        let move_type = mov![move_used_id].type_();
        if mon![attacker_id].is_type(move_type) { Percent(125) } else { Percent(100) }
    };

    let move_type = mov![move_used_id].type_();
    let target_type = mon![defender_id].species.type_();

    let type_effectiveness = dual_type_matchup(move_type, target_type);

    // If the opponent is immune, damage calculation is skipped.
    if type_effectiveness.is_matchup_ineffective() {
        sim.push_message("It was ineffective...");
        return Outcome::Failure;
    }

    let type_matchup_multiplier: Percent = type_effectiveness.into();

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
    Taking this stuff into account, we need a better separation of "move use" and "move hit" if not just
    so that one can do something after (and before) of all of the hits.
    */
    sim.push_message(format!["It was {}!", type_effectiveness.as_text()]);

    let damage = events::trigger_on_modify_damage_event(sim, attacker_id, context, damage);

    let _ = deal_raw_damage(sim, (defender_id, damage));

    Outcome::Success(NOTHING)
}

/// The simulator simulates dealing damage equalling `Context.1` to the target `Context.0`. The simulator currently
///  assumes that the only way to faint a monster is by dealing damage equal to its max health, i.e. through this
/// function. So the fainting logic is in this function.
/// 
/// Returns the actual damage dealt.

#[must_use]
pub fn deal_raw_damage(sim: &mut BattleSimulator, context: (MonsterID, u16)) -> u16 {
    let (target_id, mut damage) = context;
    let original_health = mon![target_id].current_health;
    mon![mut target_id].current_health = original_health.saturating_sub(damage);
    sim.push_message(format![
        "{} took {damage} damage!", 
        mon![target_id].name()
    ]);
    sim.push_message(format![
        "{} has {num_hp} health left.",
        mon![target_id].name(),
        num_hp = mon![target_id].current_health
    ]);
    if mon![target_id].is_fainted() { 
        damage = original_health;
        sim.push_message(format!["{} fainted!", mon![target_id].name()]);
        switch_out_monster(sim, target_id);
    };
    events::trigger_on_damage_dealt_event(sim, target_id, NOTHING);
    damage
}

/// The simulator simulates the activation of the ability `AbilityUseContext.ability_used` owned by
/// the monster `AbilityUseContext.abilty_owner`.
#[must_use]
pub fn activate_ability(sim: &mut BattleSimulator, context: AbilityUseContext) -> Outcome<Nothing> {
    let AbilityUseContext { ability_used_id, ability_owner_id } = context;

    let try_activate_ability_outcome = events::trigger_on_try_activate_ability_event(sim, ability_owner_id, context);
    if try_activate_ability_outcome.succeeded() {
        let ability = abl![ability_used_id];
        (ability.on_activate_effect())(sim, context);
        events::trigger_on_ability_activated_event(sim, ability_owner_id, context);
        Outcome::Success(NOTHING)
    } else {
        Outcome::Failure
    }
}

/// The simulator simulates the raising of stat `Context.1` of monster `Context.0` by `Context.2` stages
#[must_use]
pub fn raise_stat(sim: &mut BattleSimulator, (affected_monster_id, stat, number_of_stages): (MonsterID, Stat, u8)) -> Outcome<Nothing> {
    let try_raise_stat_outcome = events::trigger_on_try_raise_stat_event(sim, affected_monster_id, NOTHING);
    if try_raise_stat_outcome.succeeded() {
        let effective_stages = mon![mut affected_monster_id].stat_modifiers.raise_stat(stat, number_of_stages);

        sim.push_message(format![
            "{monster}\'s {stat} was raised by {effective_stages} stage(s)!",
            monster = mon![affected_monster_id].name(),
        ]);

        Outcome::Success(NOTHING)
    } else {
        sim.push_message(format!["{monster}'s stats cannot get any higher.", monster = mon![affected_monster_id].name()]);

        Outcome::Failure
    }
}

/// The simulator simulates the lowering of stat `Context.1` of monster `Context.0` by `Context.2` stages
#[must_use]
pub fn lower_stat(sim: &mut BattleSimulator, (affected_monster_id, stat, number_of_stages): (MonsterID, Stat, u8)) -> Outcome<Nothing> {
    let try_lower_stat_outcome = events::trigger_on_try_lower_stat_event(sim, affected_monster_id, NOTHING);
    if try_lower_stat_outcome.succeeded() {
        let effective_stages = mon![mut affected_monster_id].stat_modifiers.lower_stat(stat, number_of_stages);

        sim.push_message(format![
            "{monster}\'s {stat} was lowered by {effective_stages} stage(s)!",
            monster = mon![affected_monster_id].name(),
        ]);

        Outcome::Success(NOTHING)
    } else {
        sim.push_message(format!["{monster}'s stats cannot get any lower.", monster = mon![affected_monster_id].name()]);

        Outcome::Failure
    }
}

/// Returns an `Outcome` representing whether adding the status succeeded.
#[must_use]
pub fn add_volatile_status(sim: &mut BattleSimulator, (affected_monster_id, status_species): (MonsterID, &'static VolatileStatusSpecies)) -> Outcome<Nothing> {
    // conflict. A structural change is needed to resolve this correctly.
    let try_add_status = events::trigger_on_try_add_volatile_status_event(sim, affected_monster_id, NOTHING);
    if try_add_status.succeeded() {
        let affected_monster_does_not_already_have_status = mon![affected_monster_id].volatile_status(*status_species).is_none();
        if affected_monster_does_not_already_have_status {
            
            // HACK: We currently pass in the remaining turns because `prng` is inside `battle` so it causes multiple mutable references into `sim`.
            // Resolving this will require some structural changes. Prng and Battle are too tightly coupled I think.
            let lifetime_in_turns = match status_species.lifetime_in_turns {
                Count::Fixed(n) => n,
                Count::RandomInRange { min, max } => sim.battle.prng.generate_random_number_in_range(min as u16..=max as u16) as u8,
            };
            
            mon![mut affected_monster_id].volatile_statuses.push(VolatileStatus::new(lifetime_in_turns, status_species));
            sim.push_message((status_species.on_acquired_message)(mon![affected_monster_id]));
            Outcome::Success(NOTHING)
        } else {
            Outcome::Failure
        }
    } else {
        Outcome::Failure
    }
}

#[must_use]
pub fn add_persistent_status(sim: &mut BattleSimulator, (affected_monster_id, status_species): (MonsterID, &'static PersistentStatusSpecies)) -> Outcome<Nothing> {
    let try_add_status = events::trigger_on_try_add_permanent_status_event(sim, affected_monster_id, NOTHING);
    if try_add_status.succeeded() {
        let affected_monster_does_not_already_have_status = mon![affected_monster_id].persistent_status.is_none();
        if affected_monster_does_not_already_have_status {
            mon![mut affected_monster_id].persistent_status = Some(PersistentStatus::new(&status_species));
            sim.push_message((status_species.on_acquired_message)(mon![affected_monster_id]));
            Outcome::Success(NOTHING)
        } else {
            Outcome::Failure
        }
    } else {
        Outcome::Failure
    }
}

pub fn use_item<T, F>(sim: &mut BattleSimulator, item_holder_id: MonsterID, on_use_effect: F) -> Outcome<T> 
    where F: FnOnce(&mut BattleSimulator, MonsterID) -> T
{
    let context = ItemUseContext::from_holder(item_holder_id);
    let try_use_item = events::trigger_on_try_use_held_item_event(sim, item_holder_id, context);
    if try_use_item.succeeded() {
        let held_item = sim.battle.monster(item_holder_id).held_item.clone();
        if let Some(held_item) = held_item {
            if held_item.species.is_consumable {
                // If an item is marked as consumable, it is remembered as a "consumed item" for the sake of moves like Recycle.
                // Canonically, only one item can be remembered in this way, hence `consumed_item` being an `Option<Item>`.
                sim.battle.monster_mut(item_holder_id).consumed_item = sim.battle.monster(item_holder_id).held_item;
                sim.battle.monster_mut(item_holder_id).held_item = None;
            }
            let on_use_outcome = on_use_effect(sim, item_holder_id);
            events::trigger_on_held_item_used_event(sim, item_holder_id, context);
            Outcome::Success(on_use_outcome)
        } else {
            Outcome::Failure
        }
    } else {
        Outcome::Failure
    }
}