use monsim_macros::{mon, mov};
use monsim_utils::{not, ClampedPercent, Count, MaxSizedVec, Nothing, Outcome, Percent, NOTHING};

use crate::{
    dual_type_matchup,
    sim::event_dispatcher,
    status::{PersistentStatus, VolatileStatus},
    AbilityActivationContext, BattleState, BoardPosition, FieldPosition, ItemUseContext, MonsterID, MoveCategory, MoveHitContext, MoveUseContext,
    PersistentStatusSpecies, Stat, SwitchContext, VolatileStatusSpecies,
};

/// The Simulator simulates the use of a move `move_use_context.move_used_id` by
/// `move_use_context.move_user_id` on all Monsters in `move_use_context.target_ids`
/// (there may be more than one).
pub(crate) fn use_move(battle: &mut BattleState, move_use_context: MoveUseContext) {
    let MoveUseContext {
        move_user_id,
        move_used_id,
        target_ids,
    } = move_use_context;
    assert!(mov![move_used_id].current_power_points > 0, "A move was used that had zero power points");

    let try_use_move_outcome = event_dispatcher::trigger_on_try_move_event(battle, move_user_id, move_use_context);
    if try_use_move_outcome.failed() {
        battle.queue_message("The move failed!");
        return;
    }

    // There are no remaining targets for this move. They fainted before the move was used.
    if target_ids.is_empty() {
        battle.queue_message(format!["{}'s {} has no targets...", mon![move_user_id].name(), mov![move_used_id].name()]);
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
        let subcontext = MoveHitContext {
            move_user_id,
            move_used_id,
            target_id,
        };
        let mut actual_number_of_hits = 0;
        for _ in {
            match mov![move_used_id].hits_per_target() {
                Count::Fixed(number_of_hits) => 1..=number_of_hits,
                Count::RandomInRange { min, max } => {
                    let number_of_hits = battle.roll_random_number_in_range(min as u16..=max as u16);
                    1..=number_of_hits as u8
                }
            }
        } {
            {
                let target = if move_user_id == target_id {
                    "itself".to_owned()
                } else {
                    mon![target_id].name()
                };
                battle.queue_message(format![
                    "{attacker} used {move_} on {target}",
                    attacker = mon![move_user_id].name(),
                    move_ = mov![move_used_id].name(),
                ]);
            }

            if mon![target_id].is_fainted() {
                continue;
            }

            let move_hit_outcome = mov![move_used_id].on_hit_effect()(battle, subcontext);
            if move_hit_outcome.succeeded() {
                actual_number_of_hits += 1;
            }
        }

        match mov![move_used_id].hits_per_target() {
            Count::Fixed(n) if n > 1 => {
                battle.queue_message(format!["The move hit {} time(s)", n]);
            }
            Count::RandomInRange { min: _, max: _ } => {
                battle.queue_message(format!["The move hit {} time(s)", actual_number_of_hits]);
            }
            _ => {}
        }
    }

    mov![mut move_used_id].current_power_points -= 1;

    #[cfg(feature = "debug")]
    battle.queue_message(format![
        "{}'s {}'s PP is now {}",
        mon![move_user_id].name(),
        mov![move_used_id].name(),
        mov![move_used_id].current_power_points()
    ]);

    match mov![move_used_id].category() {
        MoveCategory::Physical | MoveCategory::Special => {
            event_dispatcher::trigger_on_damaging_move_used_event(battle, move_user_id, move_use_context);
        }
        MoveCategory::Status => event_dispatcher::trigger_on_status_move_used_event(battle, move_user_id, move_use_context),
    }
}

/// The simulator switches out the Monster given by `switch_context.active_monster_id` and switches in
/// the Monster given by `switch_context.benched_monster_id`
pub(crate) fn switch_monsters(battle: &mut BattleState, switch_context: SwitchContext) {
    let SwitchContext {
        active_monster_id,
        benched_monster_id,
    } = switch_context;
    let active_monster_field_position = mon![active_monster_id]
        .field_position()
        .expect("Expected the monster to be switched out to be on the field.");
    switch_out_monster(battle, active_monster_id);
    switch_in_monster(battle, benched_monster_id, active_monster_field_position);
}

/// The Simulator switchees in the Monster given by `benched_monster_id` into a field position that is presumed to
/// be empty, given by `field_position`. The caller must ensure that the field position is indeed empty.
pub(crate) fn switch_in_monster(battle: &mut BattleState, benched_monster_id: MonsterID, field_position: FieldPosition) {
    mon![mut benched_monster_id].board_position = BoardPosition::Field(field_position);
    battle.queue_message(format!["Go {}!", mon![benched_monster_id].name()]);
}

pub(crate) fn switch_out_monster(battle: &mut BattleState, active_monster_id: MonsterID) {
    let active_monster = mon![mut active_monster_id];
    active_monster.board_position = BoardPosition::Bench;
    active_monster.volatile_statuses = MaxSizedVec::empty();

    battle.queue_message(format!["Come back {}!", mon![active_monster_id].name()]);
}

/// The Simulator simulates dealing damage via a Move given by `move_hit_context.move_used_id` by a Monster given by
/// `move_hit_context.move_user_id` on a Monster given by `move_hit_context.target_id` using the default damage formula.
///
/// This is done by calculating the damage first using the canonical damage formula then calling `deal_raw_damage`
/// with the resulting damage.
///
/// Returns an `Outcome` signifying whether the move succeeded.

pub fn deal_calculated_damage(battle: &mut BattleState, move_hit_context: MoveHitContext) -> Outcome<Nothing> {
    let MoveHitContext {
        move_user_id: attacker_id,
        move_used_id,
        target_id: defender_id,
    } = move_hit_context;

    let try_move_hit_outcome = event_dispatcher::trigger_on_try_move_hit_event(battle, attacker_id, move_hit_context);
    if try_move_hit_outcome.failed() {
        battle.queue_message(format!["The move failed to hit {}!", mon![defender_id].name()]);
        return Outcome::Failure;
    }

    let base_accuracy = mov![move_used_id].base_accuracy();

    // If base_accuracy is `None`, the move is never-miss.
    if let Some(base_accuracy) = base_accuracy {
        // TODO: More sophisticated accuracy calculation
        let modified_accuracy = event_dispatcher::trigger_on_modify_accuracy_event(battle, attacker_id, move_hit_context, base_accuracy);
        if not!(battle.roll_chance(modified_accuracy, 100)) {
            battle.queue_message("The move missed!");
            return Outcome::Failure;
        }
    } else {
        #[cfg(feature = "debug")]
        battle.queue_message(format!["{} bypassed accuracy check!", mov![move_used_id].name()]);
    }

    let level = mon![attacker_id].level;
    let move_power = mov![move_used_id].base_power();

    let (attackers_attacking_stat, defenders_defense_stat) = match mov![move_used_id].category() {
        MoveCategory::Physical => (mon![attacker_id].stat(Stat::PhysicalAttack), mon![defender_id].stat(Stat::PhysicalDefense)),
        MoveCategory::Special => (mon![attacker_id].stat(Stat::SpecialAttack), mon![defender_id].stat(Stat::SpecialDefense)),
        _ => unreachable!("Expected physical or special move."),
    };

    let attackers_attacking_stat = event_dispatcher::trigger_on_calculate_attack_stat_event(battle, attacker_id, move_hit_context, attackers_attacking_stat);
    let defenders_defense_stat = event_dispatcher::trigger_on_calculate_defense_stat_event(battle, attacker_id, move_hit_context, defenders_defense_stat);

    let random_multiplier = battle.roll_random_number_in_range(85..=100);
    let random_multiplier = ClampedPercent::from(random_multiplier);

    let stab_multiplier = {
        let move_type = mov![move_used_id].type_();
        if mon![attacker_id].is_type(move_type) {
            Percent(125)
        } else {
            Percent(100)
        }
    };

    let move_type = mov![move_used_id].type_();
    let target_type = mon![defender_id].species.type_();

    let type_effectiveness = dual_type_matchup(move_type, target_type);

    // If the opponent is immune, damage calculation is skipped.
    if type_effectiveness.is_matchup_ineffective() {
        battle.queue_message("It was ineffective...");
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
    TODO: There was quite a bit written here about trying to organise multiple hits so that type effectiveness can be printed
    after the last hit. I think this only applies to multihit moves like Bullet Seed, and not multitarget moves like Growl or
    Bubble. So that will make it interesting to try to tackle this issue.
    */
    battle.queue_message(format!["It was {}!", type_effectiveness.as_text()]);

    let damage = event_dispatcher::trigger_on_modify_damage_event(battle, attacker_id, move_hit_context, damage);

    let _ = deal_raw_damage(battle, defender_id, damage);

    event_dispatcher::trigger_on_move_hit_event(battle, attacker_id, move_hit_context);

    Outcome::Success(NOTHING)
}

/// The Simulator simulates dealing damage exactly equal to `damage` to the Monster given by `target_id`. The actual damage dealt
/// may be less if the target has less HP than `damage`. Note that the Simulator currently assumes that the only way to faint a
/// monster is by dealing damage equal to its max health, _i.e._ through this function. As such, this function also takes care of
/// fainting the target monster if it does in fact faint due to the damage taken.
///
/// Returns the actual damage dealt. This function cannot fail.
#[must_use]
pub fn deal_raw_damage(battle: &mut BattleState, target_id: MonsterID, damage: u16) -> u16 {
    let original_health = mon![target_id].current_health;
    let actual_damage;
    mon![mut target_id].current_health = original_health.saturating_sub(damage);
    battle.queue_message(format!["{} took {damage} damage!", mon![target_id].name()]);
    battle.queue_message(format![
        "{} has {remaining_hp} health left.",
        mon![target_id].name(),
        remaining_hp = mon![target_id].current_health
    ]);
    if mon![target_id].is_fainted() {
        // If the target fainted, the actual damage was less than the damage intended to be inflicted
        actual_damage = original_health;
        battle.queue_message(format!["{} fainted!", mon![target_id].name()]);
        switch_out_monster(battle, target_id);
    } else {
        actual_damage = damage;
    };
    event_dispatcher::trigger_on_damage_dealt_event(battle, target_id, NOTHING);
    actual_damage
}

/// The Simulator simulates the activation of the Ability given by owned by the Monster given by `ability_activation_context.abilty_owner_id`.
#[must_use]
pub fn activate_ability<F>(battle: &mut BattleState, ability_owner_id: MonsterID, on_activate_effect: F) -> Outcome<Nothing>
where
    F: FnOnce(&mut BattleState, AbilityActivationContext) -> Outcome<Nothing>,
{
    let ability_activation_context = AbilityActivationContext::from_owner(ability_owner_id);
    let try_activate_ability_outcome = event_dispatcher::trigger_on_try_activate_ability_event(battle, ability_owner_id, ability_activation_context);
    if try_activate_ability_outcome.succeeded() {
        let activation_outcome = on_activate_effect(battle, ability_activation_context);
        event_dispatcher::trigger_on_ability_activated_event(battle, ability_owner_id, ability_activation_context);
        activation_outcome
    } else {
        Outcome::Failure
    }
}

/// The Simulator simulates the raising of stat `stat` of Monster given by `affected_monster_id` by `number_of_stages` stages
#[must_use]
pub fn raise_stat(battle: &mut BattleState, affected_monster_id: MonsterID, stat: Stat, number_of_stages: u8) -> Outcome<Nothing> {
    let try_raise_stat_outcome = event_dispatcher::trigger_on_try_raise_stat_event(battle, affected_monster_id, NOTHING);
    if try_raise_stat_outcome.succeeded() {
        let effective_stages = mon![mut affected_monster_id].stat_modifiers.raise_stat(stat, number_of_stages);

        battle.queue_message(format![
            "{monster}\'s {stat} was raised by {effective_stages} stage(s)!",
            monster = mon![affected_monster_id].name(),
        ]);

        Outcome::Success(NOTHING)
    } else {
        battle.queue_message(format!["{monster}'s stats cannot get any higher.", monster = mon![affected_monster_id].name()]);

        Outcome::Failure
    }
}

/// The Simulator simulates the lowering of stat `stat` of Monster given by `affected_monster_id` by `number_of_stages` stages
#[must_use]
pub fn lower_stat(battle: &mut BattleState, affected_monster_id: MonsterID, stat: Stat, number_of_stages: u8) -> Outcome<Nothing> {
    let try_lower_stat_outcome = event_dispatcher::trigger_on_try_lower_stat_event(battle, affected_monster_id, NOTHING);
    if try_lower_stat_outcome.succeeded() {
        let effective_stages = mon![mut affected_monster_id].stat_modifiers.lower_stat(stat, number_of_stages);

        battle.queue_message(format![
            "{monster}\'s {stat} was lowered by {effective_stages} stage(s)!",
            monster = mon![affected_monster_id].name(),
        ]);

        Outcome::Success(NOTHING)
    } else {
        battle.queue_message(format!["{monster}'s stats cannot get any lower.", monster = mon![affected_monster_id].name()]);

        Outcome::Failure
    }
}

/// The Simulator simulates the acquiring of a volatile status condition of species `status_species` by the Monster given by `affected_monster_id`. This will only
/// be successful if the Monster _does not_ already have a volatile status condition of the same species (in addition to anything else on the field preventing it).
///
/// Returns an `Outcome` representing whether adding the status succeeded.
#[must_use]
pub fn add_volatile_status(battle: &mut BattleState, affected_monster_id: MonsterID, status_species: &'static VolatileStatusSpecies) -> Outcome<Nothing> {
    // conflict. A structural change is needed to resolve this correctly.
    let try_add_status = event_dispatcher::trigger_on_try_add_volatile_status_event(battle, affected_monster_id, NOTHING);
    if try_add_status.succeeded() {
        let affected_monster_does_not_already_have_status = mon![affected_monster_id].volatile_status(*status_species).is_none();
        if affected_monster_does_not_already_have_status {
            let volatile_status = VolatileStatus::from_species(&mut battle.prng, status_species);
            mon![mut affected_monster_id].volatile_statuses.push(volatile_status);
            battle.queue_message((status_species.on_acquired_message)(mon![affected_monster_id]));
            Outcome::Success(NOTHING)
        } else {
            Outcome::Failure
        }
    } else {
        Outcome::Failure
    }
}

/// The Simulator simulates the acquiring of a persistent status condition of species `status_species` by the Monster given by `affected_monster_id`. This will only
/// be successful if the Monster does not already have a persistent status condition (in addition to anything else on the field preventing it).
///
/// Returns an `Outcome` representing whether adding the status succeeded.
#[must_use]
pub fn add_persistent_status(battle: &mut BattleState, affected_monster_id: MonsterID, status_species: &'static PersistentStatusSpecies) -> Outcome<Nothing> {
    let try_add_status = event_dispatcher::trigger_on_try_add_permanent_status_event(battle, affected_monster_id, NOTHING);
    if try_add_status.succeeded() {
        let affected_monster_does_not_already_have_status = mon![affected_monster_id].persistent_status.is_none();
        if affected_monster_does_not_already_have_status {
            mon![mut affected_monster_id].persistent_status = Some(PersistentStatus::new(status_species));
            battle.queue_message((status_species.on_acquired_message)(mon![affected_monster_id]));
            Outcome::Success(NOTHING)
        } else {
            Outcome::Failure
        }
    } else {
        Outcome::Failure
    }
}

/// The Simulator simulates the use of the item owned by the Monster given by `item_holder_id`. Note that this will only succeed if the Monster has a held item
/// at the time of calling (in addition to anything else on the field preventing its activation).
pub fn use_item<T, F>(battle: &mut BattleState, item_holder_id: MonsterID, on_activate_effect: F) -> Outcome<T>
where
    F: FnOnce(&mut BattleState, MonsterID) -> T,
{
    let item_use_context = ItemUseContext::from_holder(item_holder_id);
    let try_use_item = event_dispatcher::trigger_on_try_use_held_item_event(battle, item_holder_id, item_use_context);
    if try_use_item.succeeded() {
        let held_item = mon![item_holder_id].held_item;
        if let Some(held_item) = held_item {
            if held_item.species.is_consumable {
                // If an item is marked as consumable, it is remembered as a "consumed item" for the sake of moves like Recycle.
                // Canonically, only one item can be remembered in this way, hence `consumed_item` being an `Option<Item>`.
                mon![mut item_holder_id].consumed_item = mon![item_holder_id].held_item;
                mon![mut item_holder_id].held_item = None;
            }
            let on_use_outcome = on_activate_effect(battle, item_holder_id);
            event_dispatcher::trigger_on_held_item_used_event(battle, item_holder_id, item_use_context);
            Outcome::Success(on_use_outcome)
        } else {
            Outcome::Failure
        }
    } else {
        Outcome::Failure
    }
}
