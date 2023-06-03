use crate::sim::prng::Prng;

use super::{
    battle_context::BattleContext,
    event::{event_dex::*, EventResolver},
    game_mechanics::{monster::Stat, move_::MoveCategory, BattlerUID, MoveUID},
    global_constants::{type_matchup, FAILURE, INEFFECTIVE, SUCCESS},
    SimError, TurnOutcome,
};

/// Primary Actions are functions that are meant to be called by the
/// simulator to initiate a monster's turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryAction;

impl PrimaryAction {
    /// Primary action: A monster's turn may be initiated by this Action.
    ///
    /// Calculates and applies the effects of a damaging move
    /// corresponding to `move_uid` being used on `target_uid`
    pub fn damaging_move(
        ctx: &mut BattleContext,
        prng: &mut Prng,
        move_uid: MoveUID,
        target_uid: BattlerUID,
    ) -> TurnOutcome {
        let attacker_uid = move_uid.battler_uid;
        let attacker = ctx.monster(attacker_uid);
        let move_ = ctx.move_(move_uid);

        ctx.push_message(&format![
            "{} used {}",
            attacker.nickname, move_.species.name
        ]);

        if EventResolver::broadcast_try_event(ctx, prng, attacker_uid, &OnTryMove) == FAILURE {
            ctx.push_message(&"The move failed!");
            return Ok(());
        }

        let level = ctx.monster(attacker_uid).level;
        let move_power = ctx.move_(move_uid).base_power();

        let attackers_attacking_stat;
        let targets_defense_stat;

        match ctx.move_(move_uid).category() {
            MoveCategory::Physical => {
                attackers_attacking_stat = ctx.monster(attacker_uid).stats[Stat::PhysicalAttack];
                targets_defense_stat = ctx.monster(target_uid).stats[Stat::PhysicalDefense];
            }
            MoveCategory::Special => {
                attackers_attacking_stat = ctx.monster(attacker_uid).stats[Stat::SpecialAttack];
                targets_defense_stat = ctx.monster(target_uid).stats[Stat::SpecialDefense];
            }
            MoveCategory::Status => {
                return Err(SimError::InvalidStateError(String::from(
                    "The damaging_move function is not expected to receive status moves.",
                )))
            }
        }

        let random_multiplier = prng.generate_u16_in_range(85..=100);
        let random_multiplier = random_multiplier as f64 / 100.0;

        let stab_multiplier = {
            let move_type = ctx.move_(move_uid).species.type_;
            if ctx.monster(attacker_uid).is_type(move_type) {
                1.25f64
            } else {
                1.00f64
            }
        };

        let move_type = ctx.move_(move_uid).species.type_;
        let target_primary_type = ctx.monster(target_uid).species.primary_type;
        let target_secondary_type = ctx.monster(target_uid).species.secondary_type;

        let type_matchup_multiplier = type_matchup(move_type, target_primary_type)
            * type_matchup(move_type, target_secondary_type);

        // If the opponent is immune, damage calculation is skipped.
        if type_matchup_multiplier == INEFFECTIVE {
            ctx.push_message(&"It was ineffective...");
            return Ok(());
        }

        // The (WIP) bona-fide damage formula.
        let mut damage = (2 * level) / 5;
        damage += 2;
        damage *= move_power;
        damage *= attackers_attacking_stat / targets_defense_stat;
        damage /= 50;
        damage += 2;
        damage = (damage as f64 * random_multiplier) as u16;
        damage = (damage as f64 * stab_multiplier) as u16;
        damage = (damage as f64 * type_matchup_multiplier) as u16;
        // TODO: Introduce more damage multipliers as we implement them.

        // Do the calculated damage to the target
        SecondaryAction::damage(ctx, target_uid, damage);
        EventResolver::broadcast_event(ctx, prng, attacker_uid, &OnDamageDealt, (), None);

        let type_matchup_multiplier_times_hundred =
            f64::floor(type_matchup_multiplier * 100.0) as u16;
        // INFO: We cannot match against floats so we match against 100 x the multiplier rounded to an int.
        let type_effectiveness = match type_matchup_multiplier_times_hundred {
            25 | 50 => "not very effective",
            100 => "effective",
            200 | 400 => "super effective",
            value => {
                return Err(SimError::InvalidStateError(format![
                    "Type Effectiveness Multiplier is unexpectedly {}",
                    value
                ]))
            }
        };
        ctx.push_message(&format!["It was {}!", type_effectiveness]);
        ctx.push_message(&format![
            "{} took {} damage!",
            ctx.monster(target_uid).nickname,
            damage
        ]);
        ctx.push_message(&format![
            "{} has {} health left.",
            ctx.monster(target_uid).nickname,
            ctx.monster(target_uid).current_health
        ]);

        Ok(())
    }

    pub fn status_move(
        ctx: &mut BattleContext,
        prng: &mut Prng,
        move_uid: MoveUID,
        target_uid: BattlerUID,
    ) -> TurnOutcome {
        let attacker_uid = move_uid.battler_uid;
        let attacker = ctx.monster(attacker_uid);
        let move_ = *ctx.move_(move_uid);

        ctx.push_message(&format![
            "{} used {}",
            attacker.nickname, move_.species.name
        ]);

        if EventResolver::broadcast_try_event(ctx, prng, attacker_uid, &OnTryMove) == FAILURE {
            ctx.push_message(&"The move failed!");
            return Ok(());
        }

        move_.on_activate(ctx, prng, attacker_uid, target_uid);
        EventResolver::broadcast_event(ctx, prng, attacker_uid, &OnStatusMoveUsed, (), None);

        Ok(())
    }
}

/// Secondary Actions are meant to be called by other Actions (both Primary
/// and Secondary). This leads to a chain-reaction of Actions. It is up to the
/// user to avoid making loops of actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecondaryAction;

impl SecondaryAction {
    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Deducts `damage` from HP of target corresponding to `target_uid`.
    ///
    /// This function should be used when an amount of damage has already been calculated,
    /// and the only thing left to do is to deduct it from the HP of the target.
    pub fn damage(ctx: &mut BattleContext, target_uid: BattlerUID, damage: u16) {
        ctx.monster_mut(target_uid).current_health = ctx
            .monster(target_uid)
            .current_health
            .saturating_sub(damage);
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves activation of any ability.
    ///
    /// Returns a `bool` indicating whether the ability succeeded.
    pub fn activate_ability(
        ctx: &mut BattleContext,
        prng: &mut Prng,
        owner_uid: BattlerUID,
    ) -> bool {
        if EventResolver::broadcast_try_event(ctx, prng, owner_uid, &OnTryActivateAbility) {
            let ability = *ctx.ability(owner_uid);
            ability.on_activate(ctx, owner_uid);
            EventResolver::broadcast_event(ctx, prng, owner_uid, &OnAbilityActivated, (), None);
            SUCCESS
        } else {
            FAILURE
        }
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves raising the `stat` stat of the battler corresponding to `battler_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat raising succeeded.
    pub fn raise_stat(
        ctx: &mut BattleContext,
        prng: &mut Prng,
        battler_uid: BattlerUID,
        stat: Stat,
        number_of_stages: u8,
    ) -> bool {
        if EventResolver::broadcast_try_event(ctx, prng, battler_uid, &OnTryRaiseStat) {
            let effective_stages = ctx
                .monster_mut(battler_uid)
                .stat_modifiers
                .raise_stat(stat, number_of_stages);
            ctx.push_message(&format![
                "{}\'s {:?} was raised by {} stage(s)!",
                ctx.monster(battler_uid).name(),
                stat,
                effective_stages
            ]);
            SUCCESS
        } else {
            ctx.push_message(&format![
                "{}'s stats were not raised.",
                ctx.monster(battler_uid).name()
            ]);
            FAILURE
        }
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves lowering the `stat` stat of the battler corresponding to `battler_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat lowering succeeded.
    pub fn lower_stat(
        ctx: &mut BattleContext,
        prng: &mut Prng,
        battler_uid: BattlerUID,
        stat: Stat,
        number_of_stages: u8,
    ) -> bool {
        if EventResolver::broadcast_try_event(ctx, prng, battler_uid, &OnTryLowerStat) {
            let effective_stages = ctx
                .monster_mut(battler_uid)
                .stat_modifiers
                .lower_stat(stat, number_of_stages);
            ctx.push_message(&format![
                "{}\'s {:?} was lowered by {} stage(s)!",
                ctx.monster(battler_uid).name(),
                stat,
                effective_stages
            ]);
            SUCCESS
        } else {
            ctx.push_message(&format![
                "{}'s stats were not lowered.",
                ctx.monster(battler_uid).name()
            ]);
            FAILURE
        }
    }
}
