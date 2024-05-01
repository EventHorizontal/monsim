#![allow(non_upper_case_globals)]

use std::ops::Deref;

use monsim_macros::{abl, mon, mov};

use crate::matchup;

use self::targetting::BoardPosition;

use super::event_dex::*;
use super::*;

type EffectFunction<R,C> = fn(&mut BattleSimulator, C) -> R;

/// `R`: A type that encodes any necessary information about how the `Effect` played
/// out, _e.g._ an `Outcome` representing whether the `Effect` succeeded.
///
/// `C`: Any information necessary for the resolution of the effect, provided 
/// directly, such as the user of the move, the move used and the target 
/// in case of a move's effect. 
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Effect<R, C>(EffectFunction<R, C>);

impl<R, C> Deref for Effect<R, C> {
    type Target = EffectFunction<R, C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R, C> Effect<R, C> {
    pub const fn from(effect: EffectFunction<R, C>) -> Self {
        Self(effect)
    }
}

// TODO: / INFO: Removed the `SimResult` return type on Actions. This we 
// be added back in if/when it is actually needed, when the simulator could actually
// throw an error.

// internal `Effects` that are only supposed to be used by the engine.

/// The simulator simulates the use of a move `MoveUseContext.move_used` by 
/// `MoveUseContext.move_user` on `MoveUseContext.target`.
pub(crate) const UseMove: Effect<Nothing, MoveUseContext> = Effect(use_move);

fn use_move(sim: &mut BattleSimulator, context: MoveUseContext) {
    let MoveUseContext { move_user_id, move_used_id, target_id: _ } = context;

    sim.push_message(format![
        "{attacker} used {_move}",
        attacker = mon![move_user_id].name(),
        _move = mov![move_used_id].name()
    ]);

    if mov![move_used_id].current_power_points == 0 {
        sim.push_message("but the move is out of power points!");
        return;
    }

    if sim.trigger_try_event(OnTryMove, move_user_id, context).failed() {
        sim.push_message("The move failed!");
        return;
    }

    sim.activate_move_effect(context);
    mov![mut move_used_id].current_power_points -= 1;
   #[cfg(feature="debug")]
    sim.push_message(format![
        "{}'s {}'s PP is now {}",
        mon![move_user_id].name(),
        mov![move_used_id].name(),
        mov![move_used_id].current_power_points()
    ]);

    sim.trigger_event(OnMoveUsed, move_user_id, context, NOTHING, None);
}

pub(crate) const PerformSwitchOut: Effect<Nothing, SwitchContext> = Effect(perform_switch_out);

fn perform_switch_out(sim: &mut BattleSimulator, context: SwitchContext) {
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

// public `Effects` usable by users of the crate.

/// The simulator simulates dealing damage of a move given by `MoveUseContext.move_used` by 
/// `MoveUseContext.move_user` on `MoveUseContext.target` using the default damage formula.
/// 
/// This is done by calculating the damage first using the formula then calling `DealDirectDamage`
/// with the resulting damage.
pub const DealDefaultDamage: Effect<Nothing, MoveUseContext> = Effect(deal_default_damage);

fn deal_default_damage(sim: &mut BattleSimulator, context: MoveUseContext) {
    let MoveUseContext { move_user_id: attacker_id, move_used_id, target_id: defender_id } = context;

    if sim.trigger_try_event(OnTryMove, attacker_id, context).failed() {
        sim.push_message("The move failed!");
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
    damage *= attackers_attacking_stat / defenders_defense_stat;
    damage /= 50;
    damage += 2;
    damage = (damage as f64 * random_multiplier) as u16;
    damage = (damage as f64 * stab_multiplier) as u16;
    damage = (damage as f64 * type_matchup_multiplier) as u16;
    // TODO: Introduce more damage multipliers as we implement them.

    // Do the calculated damage to the target
    DealDirectDamage(sim, (defender_id, damage));
    sim.trigger_event(OnDamageDealt, attacker_id, NOTHING, NOTHING, None);

    let type_effectiveness = match type_matchup_multiplier {
        Percent(25) | Percent(50) => "not very effective",
        Percent(100) => "effective",
        Percent(200) | Percent(400) => "super effective",
        value => {
            let type_multiplier_as_float = value.0 as f64 / 100.0f64;
            unreachable!("Type Effectiveness Multiplier is unexpectedly {type_multiplier_as_float}")
        }
    };
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
pub const DealDirectDamage: Effect<u16, (MonsterID, u16)> = Effect(deal_direct_damge);

#[must_use]
fn deal_direct_damge(sim: &mut BattleSimulator, context: (MonsterID, u16)) -> u16 {
    let (target_id, mut damage) = context;
    let original_health = mon![target_id].current_health;
    mon![mut target_id].current_health = original_health.saturating_sub(damage);
    if mon![target_id].is_fainted() { 
        damage = original_health 
    };
    damage
}

/// The simulator simulates the activation of the ability `AbilityUseContext.ability_used` owned by
/// the monster `AbilityUseContext.abilty_owner`.
pub const ActivateAbility: Effect<Outcome, AbilityUseContext> = Effect(activate_ability);

#[must_use]
pub fn activate_ability(sim: &mut BattleSimulator, context: AbilityUseContext) -> Outcome {
    let AbilityUseContext { ability_used_id, ability_owner_id } = context;

    if sim.trigger_try_event(OnTryActivateAbility, ability_owner_id, context).succeeded() {
        let ability = abl![ability_used_id];
        (ability.on_activate_effect())(sim, context);
        sim.trigger_event(OnAbilityActivated, ability_owner_id, context, NOTHING, None);
        Outcome::Success
    } else {
        Outcome::Failure
    }
}

/// The simulator simulates the raising of stat `Context.1` of monster `Context.0` by `Context.2` stages
pub const RaiseStat: Effect<Outcome, (MonsterID, Stat, u8)> = Effect(raise_stat);

#[must_use]
pub fn raise_stat(
    sim: &mut BattleSimulator,
    (affected_monster_id, stat, number_of_stages): (MonsterID, Stat, u8), 
) -> Outcome {
    if sim.trigger_try_event(OnTryRaiseStat, affected_monster_id, NOTHING).succeeded() {
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
pub const LowerStat: Effect<Outcome, (MonsterID, Stat, u8)> = Effect(lower_stat);

#[must_use]
pub fn lower_stat(
    sim: &mut BattleSimulator,
    (affected_monster_id
        , stat, number_of_stages): (MonsterID, Stat, u8), 
) -> Outcome {
    if sim.trigger_try_event(OnTryLowerStat, affected_monster_id, NOTHING).succeeded() {
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