use monsim_macros::{mon, mov};
use monsim_utils::{not, ClampedPercent, Count, MaxSizedVec, Nothing, Outcome, Percent, NOTHING};

use crate::{
    dual_type_matchup,
    sim::event_dispatcher,
    status::{PersistentStatus, VolatileStatus},
    AbilityActivationContext, Battle, BoardPosition, FieldPosition, ItemUseContext, MonsterID, MoveCategory, MoveHitContext, MoveUseContext,
    PersistentStatusSpecies, Stat, StatChangeContext, SwitchContext, TypeEffectiveness, VolatileStatusSpecies,
};

/// The Simulator simulates the use of a move `move_use_context.move_used_id` by
/// `move_use_context.move_user_id` on all Monsters in `move_use_context.target_ids`
/// (there may be more than one).
pub(crate) fn use_move(battle: &mut Battle, move_use_context: MoveUseContext) {
    let MoveUseContext {
        move_user_id,
        move_used_id,
        target_ids,
    } = move_use_context;
    assert!(mov![move_used_id].current_power_points > 0, "A move was used that had zero power points");

    let try_use_move_outcome = event_dispatcher::trigger_on_try_move_event(battle, move_user_id, move_use_context);
    if try_use_move_outcome.is_failure() {
        battle.queue_message("The move failed!");
        return;
    }

    // There are no remaining targets for this move. They fainted before the move was used.
    if target_ids.is_empty() {
        battle.queue_message(format!["{}'s {} has no targets...", mon![move_user_id].name(), mov![move_used_id].name()]);
        return;
    }

    for target_id in target_ids {
        let number_of_hits = match mov![move_used_id].hits_per_target() {
            Count::Fixed(number_of_hits) => number_of_hits,
            Count::RandomInRange { min, max } => battle.roll_random_number_in_range(min as u16..=max as u16) as u8,
        };

        let move_hit_context = MoveHitContext {
            move_user_id,
            move_used_id,
            target_id,
            number_of_hits,
        };

        battle.queue_message(format![
            "{attacker} used {move_} on {target}",
            attacker = mon![move_user_id].name(),
            move_ = mov![move_used_id].name(),
            target = if move_user_id == target_id {
                "itself".to_owned()
            } else {
                mon![target_id].name()
            }
        ]);

        let _move_hit_outcome = mov![move_used_id].on_use_effect()(battle, move_hit_context);
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
pub(crate) fn switch_monsters(battle: &mut Battle, switch_context: SwitchContext) {
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
pub(crate) fn switch_in_monster(battle: &mut Battle, benched_monster_id: MonsterID, field_position: FieldPosition) {
    mon![mut benched_monster_id].board_position = BoardPosition::Field(field_position);
    battle.queue_message(format!["Go {}!", mon![benched_monster_id].name()]);
}

pub(crate) fn switch_out_monster(battle: &mut Battle, active_monster_id: MonsterID) {
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

pub fn deal_calculated_damage(battle: &mut Battle, move_hit_context: MoveHitContext) -> Outcome<Nothing> {
    let MoveHitContext {
        move_user_id: attacker_id,
        move_used_id,
        target_id: defender_id,
        number_of_hits,
    } = move_hit_context;

    // INFO: I am initialising this to appease the type system, but this for loop should always run at least once.
    let mut overall_type_effectiveness = TypeEffectiveness::Effective;
    let mut miss_count = 0;
    let mut fail_count = 0;

    'hits: for i in 1..=number_of_hits {
        if mon![defender_id].is_fainted() {
            fail_count += number_of_hits - i;
            break 'hits;
        }

        let try_move_hit_outcome = event_dispatcher::trigger_on_try_move_hit_event(battle, attacker_id, move_hit_context);
        if try_move_hit_outcome.is_failure() {
            battle.queue_message(format!["The move failed to hit {}!", mon![defender_id].name()]);
            fail_count += 1;
            continue 'hits;
        }

        if i > 1 {
            battle.queue_message(format!["{} hit {} again!", mov![move_used_id].name(), mon![defender_id].name()]);
        }

        let base_accuracy = mov![move_used_id].base_accuracy();

        // If base_accuracy is `None`, the move is never-miss.
        if let Some(base_accuracy) = base_accuracy {
            // TODO: More sophisticated accuracy calculation
            let modified_accuracy = event_dispatcher::trigger_on_modify_accuracy_event(battle, attacker_id, move_hit_context, base_accuracy);
            if not!(battle.roll_chance(modified_accuracy, 100)) {
                battle.queue_message("The move missed!");
                miss_count += 1;
                continue 'hits;
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

        let attackers_attacking_stat =
            event_dispatcher::trigger_on_calculate_attack_stat_event(battle, attacker_id, move_hit_context, attackers_attacking_stat);
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
        overall_type_effectiveness = type_effectiveness;

        // If the opponent is immune, damage calculation is skipped.
        if type_effectiveness.is_matchup_ineffective() {
            // INVESTIGATE: What happens if a multihit move procs an immunity? For now I'd say skip the rest of the hits.
            fail_count += 1;
            battle.queue_message("It was ineffective...");
            break 'hits;
        }

        let type_matchup_multiplier: Percent = type_effectiveness.into();

        // The (WIP) bona-fide damage formula.
        // TODO: Introduce more damage multipliers as we implement them.
        let mut damage = (2 * level) / 5;
        damage += 2;
        damage *= move_power;
        damage = (damage as f64 * (attackers_attacking_stat as f64 / defenders_defense_stat as f64)) as u16;
        damage /= 50;
        damage += 2;
        damage = (damage as f64 * random_multiplier) as u16;
        damage = (damage as f64 * stab_multiplier) as u16;
        damage = (damage as f64 * type_matchup_multiplier) as u16;
        damage = event_dispatcher::trigger_on_modify_damage_event(battle, attacker_id, move_hit_context, damage);

        let _ = deal_raw_damage(battle, defender_id, damage);

        event_dispatcher::trigger_on_move_hit_event(battle, attacker_id, move_hit_context);
    }

    let did_every_hit_miss_or_fail = fail_count + miss_count == number_of_hits;
    if not!(did_every_hit_miss_or_fail) {
        battle.queue_message(format!["It was {}!", overall_type_effectiveness.as_text()]);
    }

    if fail_count == number_of_hits {
        return Outcome::Failure;
    }

    let actual_hit_count = number_of_hits - miss_count - fail_count;

    match mov![move_used_id].hits_per_target() {
        Count::Fixed(n) if n > 1 => {
            battle.queue_message(format!["The move hit {} time(s).", actual_hit_count]);
        }
        Count::RandomInRange { min: _, max: _ } => {
            battle.queue_message(format!["The move hit {} time(s).", actual_hit_count]);
        }
        _ => {}
    }

    // INFO: I am assuming that we want to say "the move failed to hit" if every hit misses.
    // This is definitely true for the case where we try to hit only once, but
    // not sure if this is the right call for `number_of_hits > 1`.
    if overall_type_effectiveness.is_matchup_ineffective() || did_every_hit_miss_or_fail {
        Outcome::Failure
    } else {
        Outcome::Success(())
    }
}

/// The Simulator simulates dealing damage exactly equal to `damage` to the Monster given by `target_id`. The actual damage dealt
/// may be less if the target has less HP than `damage`. Note that the Simulator currently assumes that the only way to faint a
/// monster is by dealing damage equal to its max health, _i.e._ through this function. As such, this function also takes care of
/// fainting the target monster if it does in fact faint due to the damage taken.
///
/// Returns the actual damage dealt. This function cannot fail.
#[must_use]
pub fn deal_raw_damage(battle: &mut Battle, target_id: MonsterID, damage: u16) -> u16 {
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
        // If the target faints, the intended damage was either greater than or equal to
        // the target's health, but no more than it's orginal health could have been deducted.
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
pub fn activate_ability<F>(battle: &mut Battle, ability_owner_id: MonsterID, on_activate_effect: F) -> Outcome<Nothing>
where
    F: FnOnce(&mut Battle, AbilityActivationContext) -> Outcome<Nothing>,
{
    let ability_activation_context = AbilityActivationContext::from_owner(ability_owner_id);
    let try_activate_ability_outcome = event_dispatcher::trigger_on_try_activate_ability_event(battle, ability_owner_id, ability_activation_context);
    if try_activate_ability_outcome.is_success() {
        let activation_outcome = on_activate_effect(battle, ability_activation_context);
        event_dispatcher::trigger_on_ability_activated_event(battle, ability_owner_id, ability_activation_context);
        activation_outcome
    } else {
        Outcome::Failure
    }
}

pub fn change_stat(battle: &mut Battle, affected_monster_id: MonsterID, stat: Stat, number_of_stages: i8) -> Outcome<i8> {
    let stat_change_context = StatChangeContext {
        affected_monster_id,
        number_of_stages,
        stat,
    };

    // We want to trigger events with the actual number of stages so we call the `on_modify_stat_change` first.
    let modified_number_of_stages = event_dispatcher::trigger_on_modify_stat_change_event(battle, affected_monster_id, stat_change_context);

    let stat_change_context = StatChangeContext {
        affected_monster_id,
        number_of_stages: modified_number_of_stages,
        stat,
    };

    let stat_change_outcome = event_dispatcher::trigger_on_try_stat_change_event(battle, affected_monster_id, stat_change_context);

    if stat_change_outcome.is_success() {
        // After modification, the stat change may change from a rise to a lower or vice versa.
        let effective_stages = match modified_number_of_stages.cmp(&0) {
            std::cmp::Ordering::Less => {
                let effective_stages = mon![mut affected_monster_id]
                    .stat_modifiers
                    .lower_stat(stat, (-modified_number_of_stages) as u8);
                if effective_stages == 0 {
                    battle.queue_message(format!["{monster}'s stats cannot get any lower.", monster = mon![affected_monster_id].name()]);
                } else {
                    battle.queue_message(format![
                        "{monster}\'s {stat} was lowered by {effective_stages} stage(s)!",
                        monster = mon![affected_monster_id].name(),
                    ]);
                }
                -(effective_stages as i8)
            }
            std::cmp::Ordering::Equal => {
                battle.queue_message(format!["But {}'s stats ended up unchanged!", mon![affected_monster_id].name()]);
                0
            }
            std::cmp::Ordering::Greater => {
                let effective_stages = mon![mut affected_monster_id].stat_modifiers.raise_stat(stat, modified_number_of_stages as u8);
                if effective_stages == 0 {
                    battle.queue_message(format!["{monster}'s stats cannot get any higher.", monster = mon![affected_monster_id].name()]);
                } else {
                    battle.queue_message(format![
                        "{monster}\'s {stat} was raised by {effective_stages} stage(s)!",
                        monster = mon![affected_monster_id].name(),
                    ]);
                }
                effective_stages as i8
            }
        };

        event_dispatcher::trigger_on_stat_changed_event(battle, affected_monster_id, stat_change_context);

        Outcome::Success(effective_stages)
    } else {
        Outcome::Failure
    }
}

/// The Simulator simulates the acquiring of a volatile status condition of species `status_species` by the Monster given by `affected_monster_id`. This will only
/// be successful if the Monster _does not_ already have a volatile status condition of the same species (in addition to anything else on the field preventing it).
///
/// Returns an `Outcome` representing whether adding the status succeeded.
#[must_use]
pub fn add_volatile_status(battle: &mut Battle, affected_monster_id: MonsterID, status_species: &'static VolatileStatusSpecies) -> Outcome<Nothing> {
    // conflict. A structural change is needed to resolve this correctly.
    let try_inflict_status = event_dispatcher::trigger_on_try_inflict_volatile_status_event(battle, affected_monster_id, NOTHING);
    if try_inflict_status.is_success() {
        let affected_monster_does_not_already_have_status = mon![affected_monster_id].volatile_status(*status_species).is_none();
        if affected_monster_does_not_already_have_status {
            let volatile_status = VolatileStatus::from_species(&mut battle.prng, status_species);
            mon![mut affected_monster_id].volatile_statuses.push(volatile_status);
            battle.queue_message((status_species.on_acquired_message)(mon![affected_monster_id]));
            event_dispatcher::trigger_on_volatile_status_inflicted_event(battle, affected_monster_id, NOTHING);
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
pub fn add_persistent_status(battle: &mut Battle, affected_monster_id: MonsterID, status_species: &'static PersistentStatusSpecies) -> Outcome<Nothing> {
    let try_inflict_status = event_dispatcher::trigger_on_try_inflict_persistent_status_event(battle, affected_monster_id, NOTHING);
    if try_inflict_status.is_success() {
        let affected_monster_does_not_already_have_status = mon![affected_monster_id].persistent_status.is_none();
        if affected_monster_does_not_already_have_status {
            let persistent_status = PersistentStatus::from_species(status_species);
            mon![mut affected_monster_id].persistent_status = Some(persistent_status);
            battle.queue_message((status_species.on_acquired_message)(mon![affected_monster_id]));
            event_dispatcher::trigger_on_persistent_status_inflicted_event(battle, affected_monster_id, NOTHING);
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
pub fn use_item<T, F>(battle: &mut Battle, item_holder_id: MonsterID, on_activate_effect: F) -> Outcome<T>
where
    F: FnOnce(&mut Battle, MonsterID) -> T,
{
    let item_use_context = ItemUseContext::from_holder(item_holder_id);
    let try_use_item = event_dispatcher::trigger_on_try_use_held_item_event(battle, item_holder_id, item_use_context);
    if try_use_item.is_success() {
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

pub(crate) fn shift_monster(battle: &mut Battle, monster_id: MonsterID, destination_position: FieldPosition) -> Outcome<Nothing> {
    if mon![monster_id].field_position().is_some() {
        mon![mut monster_id].board_position = BoardPosition::Field(destination_position);
        battle.queue_message(format!["{} was shifted to {}", mon![monster_id].name(), destination_position]);
        Outcome::Success(NOTHING)
    } else {
        Outcome::Failure
    }
}
