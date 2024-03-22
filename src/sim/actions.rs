use crate::matchup;

use super::event_dex::*;
use super::*;

/// A Monster's turn is initiated by an **Action**. Actions can cause an Effect or use the EventDispatcher to dispatch 
/// an event.  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Action;

/// **Effects** are triggered Actions or by other Effects. This results in a chain reaction that _should_ eventually cease.
/// Effects are _atomic_, that is, you are not supposed to do half of an Effect, this may leave the Battle in an invalid state. 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effect;

impl Action {
    /// **Action** A monster's turn may be initiated by this Action.
    ///
    /// Calculates and applies the effects of a damaging move
    /// corresponding to `move_uid` being used on `target_uid`
    pub fn use_damaging_move(battle: &mut BattleState, move_uid: MoveUID, target_uid: MonsterUID) -> SimResult {
        let attacker_uid = move_uid.owner_uid;
        let calling_context = MoveUsed::new(move_uid, target_uid);

        battle.message_log.push(format![
            "{attacker} used {_move}",
            attacker = battle.monster(attacker_uid).name(),
            _move = battle.move_(move_uid).species.name
        ]);

        if EventDispatcher::dispatch_trial_event(battle, attacker_uid, calling_context, OnTryMove) == Outcome::Failure {
            battle.message_log.push_str("The move failed!");
            return Ok(NOTHING);
        }

        let level = battle.monster(attacker_uid).level;
        let move_power = battle.move_(move_uid).base_power();

        let attackers_attacking_stat;
        let targets_defense_stat;

        match battle.move_(move_uid).category() {
            MoveCategory::Physical => {
                attackers_attacking_stat = battle.monster(attacker_uid).stats[Stat::PhysicalAttack];
                targets_defense_stat = battle.monster(target_uid).stats[Stat::PhysicalDefense];
            }
            MoveCategory::Special => {
                attackers_attacking_stat = battle.monster(attacker_uid).stats[Stat::SpecialAttack];
                targets_defense_stat = battle.monster(target_uid).stats[Stat::SpecialDefense];
            }
            MoveCategory::Status => unreachable!("The damaging_move function is not expected to receive status moves."),
        }

        let random_multiplier = battle.prng.generate_random_u16_in_range(85..=100);
        let random_multiplier = ClampedPercent::from(random_multiplier);

        let stab_multiplier = {
            let move_type = battle.move_(move_uid).species.type_;
            if battle.monster(attacker_uid).is_type(move_type) { Percent(125) } else { Percent(100) }
        };

        let move_type = battle.move_(move_uid).species.type_;
        let target_primary_type = battle.monster(target_uid).species.primary_type;
        let target_secondary_type = battle.monster(target_uid).species.secondary_type;

        let type_matchup_multiplier = if let Some(target_secondary_type) = target_secondary_type {
            matchup!(move_type against target_primary_type / target_secondary_type)
        } else {
            matchup!(move_type against target_primary_type)
        };

        // If the opponent is immune, damage calculation is skipped.
        if type_matchup_multiplier.is_matchup_ineffective() {
            battle.message_log.push_str("It was ineffective...");
            return Ok(NOTHING);
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
        Effect::deal_damage(battle, target_uid, damage);
        EventDispatcher::dispatch_event(battle, attacker_uid, calling_context, OnDamageDealt, NOTHING, None);

        let type_effectiveness = match type_matchup_multiplier {
            Percent(25) | Percent(50) => "not very effective",
            Percent(100) => "effective",
            Percent(200) | Percent(400) => "super effective",
            value => {
                let type_multiplier_as_float = value.0 as f64 / 100.0f64;
                unreachable!("Type Effectiveness Multiplier is unexpectedly {type_multiplier_as_float}")
            }
        };
        battle.message_log.push(format!["It was {type_effectiveness}!"]);
        battle.message_log.push(format!["{target} took {damage} damage!", target = battle.monster(target_uid).name(),]);
        battle.message_log.push(format![
            "{target} has {num_hp} health left.",
            target = battle.monster(target_uid).name(),
            num_hp = battle.monster(target_uid).current_health
        ]);

        Ok(NOTHING)
    }

    pub fn use_status_move(battle: &mut BattleState, move_uid: MoveUID, target_uid: MonsterUID) -> SimResult {
        let attacker_uid = move_uid.owner_uid;
        let calling_context = MoveUsed::new(move_uid, target_uid);

        battle.message_log.push(format![
            "{attacker} used {move_}",
            attacker = battle.monster(attacker_uid).name(),
            move_ = battle.move_(move_uid).species.name
        ]);

        if EventDispatcher::dispatch_trial_event(battle, attacker_uid, MoveUsed::new(move_uid, target_uid), OnTryMove) == Outcome::Failure {
            battle.message_log.push_str("The move failed!");
            return Ok(NOTHING);
        }

        {
            let move_ = *battle.move_(move_uid);
            move_.on_activate(battle, attacker_uid, target_uid);
        }

        EventDispatcher::dispatch_event(battle, attacker_uid, calling_context, OnStatusMoveUsed, NOTHING, None);

        Ok(NOTHING)
    }

    pub fn perform_switch_out(battle: &mut BattleState, active_monster_uid: MonsterUID, benched_monster_uid: MonsterUID) -> SimResult {
        battle.team_mut(active_monster_uid.team_uid).active_monster_uid = benched_monster_uid;
        battle.message_log.push(format![
            "{active_monster} switched out! Go {benched_monster}!", 
            active_monster = battle.monster(active_monster_uid).name(),
            benched_monster = battle.monster(benched_monster_uid).name()
        ]);
        Ok(NOTHING)
    }
}

impl Effect {
    /// Deducts `damage` from HP of target corresponding to `target_uid`.
    ///
    /// This function should be used when an amount of damage has already been calculated,
    /// and the only thing left to do is to deduct it from the HP of the target.
    pub fn deal_damage(battle: &mut BattleState, target_uid: MonsterUID, damage: u16) {
        battle.monster_mut(target_uid).current_health = battle.monster(target_uid).current_health.saturating_sub(damage);
        if battle.monster(target_uid).current_health == 0 { battle.monster_mut(target_uid).is_fainted = true; };
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves activation of any ability.
    ///
    /// Returns a `Outcome` indicating whether the ability succeeded.
    pub fn activate_ability(battle: &mut BattleState, ability_holder_uid: MonsterUID) -> Outcome {
        let calling_context = AbilityUsed::new(ability_holder_uid);

        if EventDispatcher::dispatch_trial_event(battle, ability_holder_uid, calling_context, OnTryActivateAbility) == Outcome::Success {
            let ability = *battle.ability(ability_holder_uid);
            ability.on_activate(battle, ability_holder_uid);
            EventDispatcher::dispatch_event(battle, ability_holder_uid, calling_context, OnAbilityActivated, NOTHING, None);
            Outcome::Success
        } else {
            Outcome::Failure
        }
    }

    /// Resolves raising the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat raising succeeded.
    pub fn raise_stat(battle: &mut BattleState, monster_uid: MonsterUID, stat: Stat, number_of_stages: u8) -> Outcome {
        if EventDispatcher::dispatch_trial_event(battle, monster_uid, NOTHING, OnTryRaiseStat) == Outcome::Success {
            let effective_stages = battle.monster_mut(monster_uid).stat_modifiers.raise_stat(stat, number_of_stages);

            battle.message_log.push(format![
                "{monster}\'s {stat} was raised by {stages} stage(s)!",
                monster = battle.monster(monster_uid).name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            battle.message_log.push(format!["{monster}'s stats were not raised.", monster = battle.monster(monster_uid).name()]);

            Outcome::Failure
        }
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves lowering the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat lowering succeeded.
    pub fn lower_stat(battle: &mut BattleState, monster_uid: MonsterUID, stat: Stat, number_of_stages: u8) -> Outcome {
        if EventDispatcher::dispatch_trial_event(battle, monster_uid, NOTHING, OnTryLowerStat) == Outcome::Success {
            let effective_stages = battle.monster_mut(monster_uid).stat_modifiers.lower_stat(stat, number_of_stages);

            battle.message_log.push(format![
                "{monster}\'s {stat} was lowered by {stages} stage(s)!",
                monster = battle.monster(monster_uid).name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            battle.message_log.push(format!["{monster}'s stats were not lowered.", monster = battle.monster(monster_uid).name()]);

            Outcome::Failure
        }
    }
}