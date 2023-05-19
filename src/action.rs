use crate::prng::Lcrng;

use super::{
    battle_context::BattleContext,
    event::{event_dex::*, EventResolver},
    game_mechanics::{monster::Stat, move_::MoveCategory, BattlerUID, MoveUID},
    global_constants::{type_matchup, FAILURE, INEFFECTIVE, SUCCESS},
    SimError, TurnOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action;

impl Action {
    pub fn damaging_move(
        context: &mut BattleContext,
        prng: &mut Lcrng,
        move_uid: MoveUID,
        target_uid: BattlerUID,
    ) -> TurnOutcome {
        let attacker_uid = move_uid.battler_uid;
        let attacker = context.monster(attacker_uid);
        let move_ = context.move_(move_uid);

        context
            .message_buffer
            .push(format!["{} used {}", attacker.nickname, move_.species.name]);

        if EventResolver::broadcast_try_event(context, prng, attacker_uid, &OnTryMove) == FAILURE {
            context
                .message_buffer
                .push(String::from("The move failed!"));
            return Ok(());
        }

        let level = context.monster(attacker_uid).level;
        let move_power = context.move_(move_uid).base_power();

        let attackers_attacking_stat;
        let targets_defense_stat;

        match context.move_(move_uid).category() {
            MoveCategory::Physical => {
                attackers_attacking_stat =
                    context.monster(attacker_uid).stats[Stat::PhysicalAttack];
                targets_defense_stat = context.monster(target_uid).stats[Stat::PhysicalDefense];
            }
            MoveCategory::Special => {
                attackers_attacking_stat = context.monster(attacker_uid).stats[Stat::SpecialAttack];
                targets_defense_stat = context.monster(target_uid).stats[Stat::SpecialDefense];
            }
            MoveCategory::Status => {
                return Err(SimError::InvalidStateError(
                    "The damaging_move function is not expected to receive status moves.",
                ))
            }
        }

        let random_multiplier = prng.generate_number_in_range(85..=100);
        let random_multiplier = random_multiplier as f64 / 100.0;

        let stab_multiplier = {
            let move_type = context.move_(move_uid).species.type_;
            if context.monster(attacker_uid).is_type(move_type) {
                1.25f64
            } else {
                1.00f64
            }
        };

        let move_type = context.move_(move_uid).species.type_;
        let target_primary_type = context.monster(target_uid).species.primary_type;
        let target_secondary_type = context.monster(target_uid).species.secondary_type;

        let type_matchup_multiplier = type_matchup(move_type, target_primary_type)
            * type_matchup(move_type, target_secondary_type);

        // If the opponent is immune, damage calculation is skipped.
        if type_matchup_multiplier == INEFFECTIVE {
            context
                .message_buffer
                .push(String::from("It was ineffective..."));
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
        Action::damage(context, target_uid, damage);
        EventResolver::broadcast_event(context, prng, attacker_uid, &OnDamageDealt, (), None);

        let type_matchup_multiplier_times_hundred =
            f64::floor(type_matchup_multiplier * 100.0) as u16;
        // INFO: We cannot match against floats so we match against 100 x the multiplier rounded to an int.
        let type_effectiveness = match type_matchup_multiplier_times_hundred {
            25 | 50 => "not very effective",
            100 => "effective",
            200 | 400 => "super effective",
            _ => panic!(
                "type multiplier is unexpectedly {}",
                type_matchup_multiplier
            ),
        };
        context
            .message_buffer
            .push(format!["It was {}!", type_effectiveness]);
        context.message_buffer.push(format![
            "{} took {} damage!",
            context.monster(target_uid).nickname,
            damage
        ]);
        context.message_buffer.push(format![
            "{} has {} health left.",
            context.monster(target_uid).nickname,
            context.monster(target_uid).current_health
        ]);

        Ok(())
    }

    pub fn damage(context: &mut BattleContext, target_uid: BattlerUID, damage: u16) {
        context.monster_mut(target_uid).current_health = context
            .monster(target_uid)
            .current_health
            .saturating_sub(damage);
    }

    pub fn activate_ability(
        context: &mut BattleContext,
        prng: &mut Lcrng,
        owner_uid: BattlerUID,
    ) -> bool {
        if EventResolver::broadcast_try_event(context, prng, owner_uid, &OnTryActivateAbility) {
            let ability = *context.ability(owner_uid);
            ability.on_activate(context, owner_uid);
            EventResolver::broadcast_event(context, prng, owner_uid, &OnAbilityActivated, (), None);
            SUCCESS
        } else {
            FAILURE
        }
    }
}
