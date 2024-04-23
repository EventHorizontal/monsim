#![allow(non_upper_case_globals)]

use std::ops::Deref;

use crate::matchup;

use super::event_dex::*;
use super::*;

/// `R`: A type that encodes any necessary information about how the `Effect` played
/// out, _e.g._ an `Outcome` representing whether the `Effect` succeeded.
///
/// `C`: Any information necessary for the resolution of the effect, provided 
/// directly, such as the user of the move, the move used and the target 
/// in case of a move's effect. 
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Effect<R, C>(fn(&mut BattleSimulator, C) -> R);

impl<R, C> Deref for Effect<R, C> {
    type Target = fn(&mut BattleSimulator, C) -> R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R, C> Effect<R, C> {
    pub const fn from(effect: fn(&mut BattleSimulator, C) -> R) -> Self {
        Self(effect)
    }
}

// TODO: / INFO: Removed the `SimResult` return type on Actions. This we be added back in if/when it is actually needed, when the simulator could actually
// throw an error.

// internal `Effects` that are only supposed to be used by the engine.

/// The simulator simulates the use of a move `MoveUseContext.move_used` by 
/// `MoveUseContext.move_user` on `MoveUseContext.target`.
pub(crate) const UseMove: Effect<Nothing, MoveUseContext> = Effect(use_move);

fn use_move(sim: &mut BattleSimulator, context: MoveUseContext) {
    let MoveUseContext { move_user, move_used, target: _ } = context;

    sim.push_message(format![
        "{attacker} used {_move}",
        attacker = sim[move_user].name(),
        _move = sim[move_used].name()
    ]);

    if sim.trigger_try_event(OnTryMove, move_user, context).failed() {
        sim.push_message("The move failed!");
        return;
    }

    sim.activate_move_effect(context);

    sim.trigger_event(OnMoveUsed, move_user, context, NOTHING, None);
}

pub(crate) const PerformSwitchOut: Effect<Nothing, SwitchContext> = Effect(perform_switch_out);

fn perform_switch_out(sim: &mut BattleSimulator, context: SwitchContext) {
    let SwitchContext { active_monster, benched_monster } = context;

    sim.battle.team_mut(active_monster.team_uid).active_monster_uid = benched_monster;
    
    sim.push_message(format![
        "{active_monster} switched out! Go {benched_monster}!", 
        active_monster = sim[active_monster].name(),
        benched_monster = sim[benched_monster].name()
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
    let MoveUseContext { move_user: attacker, move_used, target: defender } = context;

    if sim.trigger_try_event(OnTryMove, attacker, context).failed() {
        sim.push_message("The move failed!");
        return;
    }

    let level = sim[attacker].level;
    let move_power = sim[move_used].base_power();

    let (attackers_attacking_stat, defenders_defense_stat) = match sim[move_used].category() {
        MoveCategory::Physical => {
            (
                sim[attacker].stats[Stat::PhysicalAttack],
                sim[defender].stats[Stat::PhysicalDefense]
            )
        }
        MoveCategory::Special => {
            (
                sim[attacker].stats[Stat::SpecialAttack],
                sim[defender].stats[Stat::SpecialDefense]
            )
        }
        _ => unreachable!("Expected physical or special move."),
    };

    let random_multiplier = sim.generate_random_number_in_range_inclusive(85..=100);
    let random_multiplier = ClampedPercent::from(random_multiplier);

    let stab_multiplier = {
        let move_type = sim[move_used].type_();
        if sim[attacker].is_type(move_type) { Percent(125) } else { Percent(100) }
    };

    let move_type = sim[move_used].type_();
    let target_primary_type = sim[defender].species.primary_type;
    let target_secondary_type = sim[defender].species.secondary_type;

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
    DealDirectDamage(sim, (defender, damage));
    sim.trigger_event(OnDamageDealt, attacker, NOTHING, NOTHING, None);

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
        "{defender} took {damage} damage!", 
        defender = sim[defender].name()
    ]);
    sim.push_message(format![
        "{defender} has {num_hp} health left.",
        defender = sim[defender].name(),
        num_hp = sim[defender].current_health
    ]);
}

/// The simulator simulates dealing damage equalling `Context.1` to the target `Context.0`.
pub const DealDirectDamage: Effect<u16, (MonsterUID, u16)> = Effect(deal_direct_damge);

#[must_use]
fn deal_direct_damge(sim: &mut BattleSimulator, context: (MonsterUID, u16)) -> u16 {
    let (target, mut damage) = context;
        let original_health = sim[target].current_health;
        sim[target].current_health = original_health.saturating_sub(damage);
        if sim[target].current_health == 0 { 
            sim[target].is_fainted = true;
            damage = original_health 
        };
        damage
}

/// The simulator simulates the activation of the ability `AbilityUseContext.ability_used` owned by
/// the monster `AbilityUseContext.abilty_owner`.
pub const ActivateAbility: Effect<Outcome, AbilityUseContext> = Effect(activate_ability);

#[must_use]
pub fn activate_ability(sim: &mut BattleSimulator, context: AbilityUseContext) -> Outcome {
    let AbilityUseContext { ability_used, ability_owner } = context;

    if sim.trigger_try_event(OnTryActivateAbility, ability_owner, context).succeeded() {
        let ability = sim[ability_used];
        (ability.on_activate_effect())(sim, context);
        sim.trigger_event(OnAbilityActivated, ability_owner, context, NOTHING, None);
        Outcome::Success
    } else {
        Outcome::Failure
    }
}

/// The simulator simulates the raising of stat `Context.1` of monster `Context.0` by `Context.2` stages
pub const RaiseStat: Effect<Outcome, (MonsterUID, Stat, u8)> = Effect(raise_stat);

#[must_use]
pub fn raise_stat(
    sim: &mut BattleSimulator,
    (affected_monster, stat, number_of_stages): (MonsterUID, Stat, u8), 
) -> Outcome {
    if sim.trigger_try_event(OnTryRaiseStat, affected_monster, NOTHING).succeeded() {
        let effective_stages = sim[affected_monster].stat_modifiers.raise_stat(stat, number_of_stages);

        sim.push_message(format![
            "{monster}\'s {stat} was raised by {effective_stages} stage(s)!",
            monster = sim[affected_monster].name(),
        ]);

        Outcome::Success
    } else {
        sim.push_message(format!["{monster}'s stats cannot get any higher.", monster = sim[affected_monster].name()]);

        Outcome::Failure
    }
}

/// The simulator simulates the lowering of stat `Context.1` of monster `Context.0` by `Context.2` stages
pub const LowerStat: Effect<Outcome, (MonsterUID, Stat, u8)> = Effect(lower_stat);

#[must_use]
pub fn lower_stat(
    sim: &mut BattleSimulator,
    (affected_monster, stat, number_of_stages): (MonsterUID, Stat, u8), 
) -> Outcome {
    if sim.trigger_try_event(OnTryLowerStat, affected_monster, NOTHING).succeeded() {
        let effective_stages = sim[affected_monster].stat_modifiers.lower_stat(stat, number_of_stages);

        sim.push_message(format![
            "{monster}\'s {stat} was lowered by {effective_stages} stage(s)!",
            monster = sim[affected_monster].name(),
        ]);

        Outcome::Success
    } else {
        sim.push_message(format!["{monster}'s stats were not lowered.", monster = sim[affected_monster].name()]);

        Outcome::Failure
    }
}