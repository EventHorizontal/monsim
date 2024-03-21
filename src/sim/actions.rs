use super::event_dex::*;
use super::*;

/// A Monster's turn is initiated by an **Action**. Actions can cause an Effect or use the EventDispatcher to dispatch 
/// an event.  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Action;

/// **Effects** are triggered Actions or by other Effects. This results in a chain reaction that _should_ eventually cease. 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effect;

impl Action {
    /// **Action** A monster's turn may be initiated by this Action.
    ///
    /// Calculates and applies the effects of a damaging move
    /// corresponding to `move_uid` being used on `target_uid`
    pub fn use_damaging_move<'a>(battle: &mut Battle, attacker: MonsterRef<'a>, move_: MoveRef<'a>, target: MonsterRef<'a>) -> SimResult {
        let calling_context = MoveUsed::new(attacker, move_, target);

        battle.message_log.push(format![
            "{attacker} used {_move}",
            attacker = attacker.name(),
            _move = move_.name()
        ]);

        if EventDispatcher::dispatch_trial_event(battle, attacker, calling_context, OnTryMove).succeeded() {
            battle.message_log.push_str("The move failed!");
            return Ok(NOTHING);
        }

        let level = attacker.level.get();
        let move_power = move_.base_power.get();

        let attackers_attacking_stat;
        let targets_defense_stat;

        match move_.category.get() {
            MoveCategory::Physical => {
                attackers_attacking_stat = attacker.stat(Stat::PhysicalAttack);
                targets_defense_stat = target.stat(Stat::PhysicalDefense);
            }
            MoveCategory::Special => {
                attackers_attacking_stat = attacker.stat(Stat::SpecialAttack);
                targets_defense_stat = target.stat(Stat::SpecialDefense);
            }
            _ => unreachable!("`damaging_move` must be called with a move of category Physical or Special."),
        }

        let random_multiplier = battle.prng.generate_u16_in_range(85..=100);
        let random_multiplier = ClampedPercent::from(random_multiplier);

        let stab_multiplier = if attacker.is_type(move_.type_.get()) { Percent(125) } else { Percent(100) };
        let type_matchup_multiplier = type_matchup_dual(move_.type_.get(), target.type_());

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
        Effect::deal_damage(battle, target, damage);
        EventDispatcher::dispatch_event(battle, attacker, calling_context, OnDamageDealt, NOTHING, None);

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
        battle.message_log.push(format!["{target} took {damage} damage!", 
            target = target.name(),
        ]);
        battle.message_log.push(format!["{target} has {num_hp} health left.",
            target = target.name(),
            num_hp = target.current_health.get()
        ]);

        Ok(NOTHING)
    }

    pub fn use_status_move<'a>(battle: &mut Battle, attacker: MonsterRef<'a>, move_: MoveRef<'a>, target: MonsterRef<'a>) -> SimResult {
        let calling_context = MoveUsed::new(attacker, move_, target);

        battle.message_log.push(format!["{attacker} used {move_}",
            attacker = attacker.name(),
            move_ = move_.name()
        ]);

        if EventDispatcher::dispatch_trial_event(battle, attacker, calling_context, OnTryMove).failed() {
            battle.message_log.push_str("The move failed!");
            return Ok(NOTHING);
        } else {
            move_.on_activate(battle, attacker, target)
        }

        EventDispatcher::dispatch_event(battle, attacker, calling_context, OnStatusMoveUsed, NOTHING, None);

        Ok(NOTHING)
    }

    pub fn perform_switch_out(battle: &Battle, active_monster: MonsterRef, benched_monster: MonsterRef) -> SimResult {
        battle.team(active_monster.team()).map(|team| { 
            team.set_active_monster(benched_monster)
        });
        battle.message_log.push(format![
            "{active_monster} switched out! Go {benched_monster}!", 
            active_monster = active_monster.name(),
            benched_monster = benched_monster.name()
        ]);
        Ok(NOTHING)
    }
}

impl Effect {
    /// Deducts `damage` from HP of target corresponding to `target_uid`.
    ///
    /// This function should be used when an amount of damage has already been calculated,
    /// and the only thing left to do is to deduct it from the HP of the target.
    pub fn deal_damage(battle: &mut Battle, target: MonsterRef, damage: u16) {
        let final_hp = target.decrease_hp(damage);
        if final_hp == 0 { target.is_fainted.set(true); };
    }

    /// Resolves activation of any ability.
    ///
    /// Returns a `Outcome` indicating whether the ability succeeded.
    #[must_use]
    pub fn activate_ability(battle: &mut Battle, ability_owner: MonsterRef) -> Outcome {
        
        let ability = ability_owner.ability();
        let calling_context = AbilityUsed::new(ability_owner);

        if EventDispatcher::dispatch_trial_event(battle, ability_owner, calling_context, OnTryActivateAbility).succeeded() {
            ability.on_activate(battle, ability_owner);
            EventDispatcher::dispatch_event(battle, ability_owner, calling_context, OnAbilityActivated, NOTHING, None);
            Outcome::Success
        } else {
            Outcome::Failure
        }
    }

    /// Resolves raising the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat raising succeeded.
    pub fn raise_stat(battle: &mut Battle, monster: MonsterRef, stat: Stat, number_of_stages: u8) -> Outcome {
        if EventDispatcher::dispatch_trial_event(battle, monster, NOTHING, OnTryRaiseStat) == Outcome::Success {
            let effective_stages = monster.raise_stat(stat, number_of_stages);

            battle.message_log.push(format![
                "{monster}\'s {stat} was raised by {stages} stage(s)!",
                monster = monster.name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            battle.message_log.push(format!["{monster}'s stats were not raised.", monster = monster.name()]);

            Outcome::Failure
        }
    }

    /// **Secondary Action** This action can only be triggered by other Actions.
    ///
    /// Resolves lowering the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
    ///
    /// Returns a `bool` indicating whether the stat lowering succeeded.
    pub fn lower_stat(battle: &mut Battle, monster: MonsterRef, stat: Stat, number_of_stages: u8) -> Outcome {
        if EventDispatcher::dispatch_trial_event(battle, monster, NOTHING, OnTryLowerStat) == Outcome::Success {
            
            let effective_stages = monster.lower_stat(stat, number_of_stages);

            battle.message_log.push(format![
                "{monster}\'s {stat} was lowered by {stages} stage(s)!",
                monster = monster.name(),
                stat = stat,
                stages = effective_stages
            ]);

            Outcome::Success
        } else {
            battle.message_log.push(format!["{monster}'s stats were not lowered.", monster = monster.name()]);

            Outcome::Failure
        }
    }
}