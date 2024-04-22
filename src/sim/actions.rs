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
    /// **Action** A monster's turn may be initiated by this Action.
    ///
    /// Calculates and applies the effects of a damaging move
    /// corresponding to `move_uid` being used on `target_uid`
    pub fn use_damaging_move(
        sim: &mut BattleSimulator,
        context: MoveUseContext,
    ) -> SimResult {
        let MoveUseContext { move_user: attacker, move_used, target: defender } = context;

        sim.push_message(format![
            "{attacker} used {_move}",
            attacker = sim[attacker].name(),
            _move = sim[move_used].name()
        ]);

        if sim.trigger_try_event(OnTryMove, attacker, context).failed() {
           sim.push_message("The move failed!");
            return Ok(NOTHING);
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
        Reaction::deal_damage(sim, defender, damage);
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

        Ok(NOTHING)
    }

    pub fn use_status_move(
        sim: &mut BattleSimulator,
        context: MoveUseContext,
    ) -> SimResult {
        let MoveUseContext { move_user, move_used, target: _ } = context;

        sim.push_message(format![
            "{move_user} used {move_}",
            move_user = sim[move_user].name(),
            move_ = sim[move_used].name()
        ]);

        if sim.trigger_try_event(OnTryMove, move_user, context).failed() {
            sim.push_message("The move failed!");
            return Ok(NOTHING);
        }
        
        sim.activate_move_effect(context);

        sim.trigger_event(OnStatusMoveUsed, move_user, context, NOTHING, None);

        Ok(NOTHING)
    }

    pub fn perform_switch_out(sim: &mut BattleSimulator, active_monster: MonsterUID, benched_monster: MonsterUID) -> SimResult {
        sim.battle.team_mut(active_monster.team_uid).active_monster_uid = benched_monster;
        sim.push_message(format![
            "{active_monster} switched out! Go {benched_monster}!", 
            active_monster = sim[active_monster].name(),
            benched_monster = sim[benched_monster].name()
        ]);
        Ok(NOTHING)
    }
}

impl Reaction {
    /// Deducts `damage` from HP of target corresponding to `target_uid`.
    ///
    /// This function should be used when an amount of damage has already been calculated,
    /// and the only thing left to do is to deduct it from the HP of the target.
    pub fn deal_damage(sim: &mut BattleSimulator, defender: MonsterUID, damage: u16) {
        sim[defender].current_health = sim[defender].current_health.saturating_sub(damage);
        if sim[defender].current_health == 0 { sim[defender].is_fainted = true; };
    }

    /// Reaction Effct: Activates the ability of `ability_owner`, resolving the consequences of the ability activation.
    ///
    /// Returns a `Outcome` indicating whether the ability succeeded.
    #[must_use]
    pub fn activate_ability(
        sim: &mut BattleSimulator,
        ability_owner: MonsterUID,
    ) -> Outcome {
        let context = AbilityUseContext::new(ability_owner);

        if sim.trigger_try_event(OnTryActivateAbility, ability_owner, context).succeeded() {
            let ability = sim[AbilityUID::from_owner(ability_owner)];
            ability.activate(sim, context);
            sim.trigger_event(OnAbilityActivated, ability_owner, context, NOTHING, None);
            Outcome::Success
        } else {
            Outcome::Failure
        }
    }

    /// Resolves raising the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat raising succeeded.
    pub fn raise_stat(
        sim: &mut BattleSimulator,
        affected_monster: MonsterUID, 
        stat: Stat, 
        number_of_stages: u8
    ) -> Outcome {
        if sim.trigger_try_event(OnTryRaiseStat, affected_monster, NOTHING).succeeded() {
            let effective_stages = sim[affected_monster].stat_modifiers.raise_stat(stat, number_of_stages);

            sim.push_message(format![
                "{monster}\'s {stat} was raised by {stages} stage(s)!",
                monster = sim[affected_monster].name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            sim.push_message(format!["{monster}'s stats were not raised.", monster = sim[affected_monster].name()]);

            Outcome::Failure
        }
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves lowering the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat lowering succeeded.
    pub fn lower_stat(
        sim: &mut BattleSimulator,
        affected_monster: MonsterUID, 
        stat: Stat, 
        number_of_stages: u8
    ) -> Outcome {
        if sim.trigger_try_event(OnTryLowerStat, affected_monster, NOTHING).succeeded() {
            let effective_stages = sim[affected_monster].stat_modifiers.lower_stat(stat, number_of_stages);

            sim.push_message(format![
                "{monster}\'s {stat} was lowered by {stages} stage(s)!",
                monster = sim[affected_monster].name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            sim.push_message(format!["{monster}'s stats were not lowered.", monster = sim[affected_monster].name()]);

            Outcome::Failure
        }
    }
}