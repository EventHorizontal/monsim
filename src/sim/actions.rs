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
pub struct Reaction;

impl Action {
    /// **Action** A monster's turn may be initiated by invoking this effect.
    ///
    /// Calculates and applies the effects of a damaging move `move_used` 
    /// used by `move_user` on `target`.
    pub fn use_damaging_move(battle: &mut Battle, context: MoveUsed) -> SimResult {
        
        let MoveUsed { move_user, move_used, target: defender } = context;
        let (prng, entities, message_log, event_dispatcher) = battle.split();

        battle.message_log.push(format![
            "{move_user} used {_move}",
            move_user = monster![move_user].name(),
            _move = move_![move_used].species.name
        ]);

        if event_dispatcher.dispatch_trial_event(prng, (entities, message_log), OnTryMove(move_user, context)).failed() {
            battle.message_log.push_str("The move failed!");
            return Ok(NOTHING);
        }

        let level = monster![move_user].level;
        let move_power = move_![move_used].base_power();

        let (attackers_attacking_stat, defenders_defense_stat) = match move_![move_used].category() {
            MoveCategory::Physical => {
                (
                    monster![move_user].stats[Stat::PhysicalAttack],
                    monster![defender].stats[Stat::PhysicalDefense]
                )
            }
            MoveCategory::Special => {
                (
                    monster![move_user].stats[Stat::SpecialAttack],
                    monster![defender].stats[Stat::SpecialDefense]
                )
            }
            _ => unreachable!("Expected physical or special move."),
        };

        let random_multiplier = prng.generate_random_number_in_range(85,100);
        let random_multiplier = ClampedPercent::from(random_multiplier);

        let stab_multiplier = {
            let move_type = move_![move_used].species.type_;
            if monster![move_user].is_type(move_type) { Percent(125) } else { Percent(100) }
        };

        let move_type = move_![move_used].species.type_;
        let target_primary_type = monster![defender].species.primary_type;
        let target_secondary_type = monster![defender].species.secondary_type;

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
        damage *= attackers_attacking_stat / defenders_defense_stat;
        damage /= 50;
        damage += 2;
        damage = (damage as f64 * random_multiplier) as u16;
        damage = (damage as f64 * stab_multiplier) as u16;
        damage = (damage as f64 * type_matchup_multiplier) as u16;
        // TODO: Introduce more damage multipliers as we implement them.

        // Do the calculated damage to the target
        Reaction::deal_damage(battle, defender, damage);
        EventDispatcher::dispatch_event(battle, OnDamageDealt(move_user, NOTHING), NOTHING, None);

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
        battle.message_log.push(format![
            "{defender} took {damage} damage!", 
            defender = monster![defender].name()
        ]);
        battle.message_log.push(format![
            "{defender} has {num_hp} health left.",
            defender = monster![defender].name(),
            num_hp = monster![defender].current_health
        ]);

        Ok(NOTHING)
    }

    pub fn use_status_move(battle: &mut Battle, context: MoveUsed) -> SimResult {
        let MoveUsed { move_user, move_used, target } = context;

        battle.message_log.push(format![
            "{move_user} used {move_}",
            move_user = monster![move_user].name(),
            move_ = move_![move_used].species.name
        ]);

        if EventDispatcher::dispatch_trial_event(battle, OnTryMove(move_user, context)).failed() {
            battle.message_log.push_str("The move failed!");
            return Ok(NOTHING);
        }

        {
            let move_ = *move_![move_used];
            move_.on_activate(battle, move_user, target);
        }

        EventDispatcher::dispatch_event(battle, OnStatusMoveUsed(move_user, context), NOTHING, None);

        Ok(NOTHING)
    }

    pub fn perform_switch_out(battle: &mut Battle, active_monster: MonsterUID, benched_monster: MonsterUID) -> SimResult {
        battle.entities.team_mut(active_monster.team_uid).active_monster_uid = benched_monster;
        battle.message_log.push(format![
            "{active_monster} switched out! Go {benched_monster}!", 
            active_monster = monster![active_monster].name(),
            benched_monster = monster![benched_monster].name()
        ]);
        Ok(NOTHING)
    }
}

impl Reaction {
    /// Deducts `damage` from HP of target corresponding to `target_uid`.
    ///
    /// This function should be used when an amount of damage has already been calculated,
    /// and the only thing left to do is to deduct it from the HP of the target.
    pub fn deal_damage(battle: &mut Battle, defender: MonsterUID, damage: u16) {
        monster![mut defender].current_health = monster![defender].current_health.saturating_sub(damage);
        if monster![defender].current_health == 0 { monster![mut defender].is_fainted = true; };
    }

    /// **Reaction** This action can only be triggered by other Actions or Reactions.
    ///
    /// Resolves activation of any ability.
    ///
    /// Returns a `Outcome` indicating whether the ability succeeded.
    pub fn activate_ability(battle: &mut Battle, context: AbilityActivated) -> Outcome {
        let AbilityActivated { ability_holder } = context;

        if battle.dispatch_trial_event(OnTryActivateAbility, context, ability_holder).succeeded() {
            let ability = *ability![ability_holder];
            ability.on_activate(battle, ability_holder);
            battle.dispatch_event(OnAbilityActivated, context, ability_holder, NOTHING, None);
            Outcome::Success
        } else {
            Outcome::Failure
        }
    }

    /// Resolves raising the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat raising succeeded.
    pub fn raise_stat(battle: &mut Battle, affected_monster: MonsterUID, stat: Stat, number_of_stages: u8) -> Outcome {
        if EventDispatcher::dispatch_trial_event(battle, affected_monster, NOTHING, OnTryRaiseStat).succeeded() {
            let effective_stages = monster![mut affected_monster].stat_modifiers.raise_stat(stat, number_of_stages);

            battle.message_log.push(format![
                "{monster}\'s {stat} was raised by {stages} stage(s)!",
                monster = monster![affected_monster].name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            battle.message_log.push(format!["{monster}'s stats were not raised.", monster = monster![affected_monster].name()]);

            Outcome::Failure
        }
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves lowering the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat lowering succeeded.
    pub fn lower_stat(battle: &mut Battle, affected_monster: MonsterUID, stat: Stat, number_of_stages: u8) -> Outcome {
        if EventDispatcher::dispatch_trial_event(battle, affected_monster, NOTHING, OnTryLowerStat).succeeded() {
            let effective_stages = monster![mut affected_monster].stat_modifiers.lower_stat(stat, number_of_stages);

            battle.message_log.push(format![
                "{monster}\'s {stat} was lowered by {stages} stage(s)!",
                monster = monster![affected_monster].name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            battle.message_log.push(format!["{monster}'s stats were not lowered.", monster = monster![affected_monster].name()]);

            Outcome::Failure
        }
    }
}