use super::{battle_context::BattleContext, game_mechanics::{MoveUID, BattlerUID, move_::MoveCategory, monster::Stat}, global_constants::{SUCCESS, FAILURE, type_matchup, INEFFECTIVE}, event::{EventResolver, event_dex::*}, BattleError, BattleResult};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action;

impl Action {
    pub fn display_message(_context: &mut BattleContext, message: &dyn Display) -> () {
        println!("{}", message);
    }
    
    pub fn damaging_move(context: &mut BattleContext, move_uid: MoveUID, target_uid: BattlerUID) -> BattleResult {
        
        let attacker_uid = move_uid.battler_uid;
        let attacker = context.read_monster(attacker_uid);
        let move_ = context.read_move(move_uid);

        Action::display_message(context,&format!["{} used {}", attacker.nickname, move_.species.name]);
        
        if EventResolver::broadcast_try_event(context, attacker_uid, &OnTryMove) == FAILURE {
            Action::display_message(context, &"The move failed!");
            return Ok(());
        }
            
        let level = context.read_monster(attacker_uid).level;
        let move_power = context.read_move(move_uid).base_power();
        
        let attackers_attacking_stat;
        let targets_defense_stat;

        match context.read_move(move_uid).category() {
            MoveCategory::Physical => {
                attackers_attacking_stat = context.read_monster(attacker_uid).stats[Stat::PhysicalAttack];
                targets_defense_stat = context.read_monster(target_uid).stats[Stat::PhysicalDefense];
            }
            MoveCategory::Special => {
                attackers_attacking_stat = context.read_monster(attacker_uid).stats[Stat::SpecialAttack];
                targets_defense_stat = context.read_monster(target_uid).stats[Stat::SpecialDefense];
            },
            MoveCategory::Status => return Err(BattleError::WrongState("The function is not supposed to be able to receive a Status move.")), 
        }

        let random_multiplier = context.prng.generate_number_in_range(85..=100);
        let random_multiplier = random_multiplier as f64 / 100.0;

        let stab_multiplier = {
            let move_type = context.read_move(move_uid).species.type_;
            if context.read_monster(attacker_uid).is_type(move_type) {
                1.25f64
            } else {
                1.00f64
            }
        };
        
        let move_type = context.read_move(move_uid).species.
        type_;
        let target_primary_type = context.read_monster(target_uid).species.primary_type;
        let target_secondary_type = context.read_monster(target_uid).species.secondary_type;

        let type_matchup_multiplier = type_matchup(move_type, target_primary_type) * type_matchup(move_type, target_secondary_type);

        // If the opponent is immune, damage calculation is skipped.
        if type_matchup_multiplier == INEFFECTIVE {
            Action::display_message(context, &"It was ineffective...");
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
        EventResolver::broadcast_event(context, attacker_uid, &OnDamageDealt, (), None);

        let target = context.read_monster(target_uid); // We need to reread data to make sure it is updated.
        Action::display_message(context, &format!("It was {}!", {
            let type_matchup_multiplier_times_hundred = f64::floor(type_matchup_multiplier * 100.0) as u16;
            // INFO: We cannot match against floats so we match against 100 x the multiplier rounded to an int.
            match type_matchup_multiplier_times_hundred {
                25 | 50 => "not very effective",
                100 => "effective",
                200 | 400 => "super effective",
                _ => panic!("type multiplier is unexpectedly {}", type_matchup_multiplier)
            }
        }));
        Action::display_message(context, &format!("{} took {} damage!", target.nickname, damage));
        Action::display_message(context, &format!("{} has {} health left.", target.nickname, target.current_health));

        Ok(())
    }
    
    pub fn damage(context: &mut BattleContext, target_uid: BattlerUID, damage: u16) -> () {
        context.write_monster(
            target_uid, 
            &mut |mut it| { it.current_health = it.current_health.saturating_sub(damage); it }
        );
    }

    pub fn activate_ability(context: &mut BattleContext, owner_uid: BattlerUID) -> bool {
        if EventResolver::broadcast_try_event(context, owner_uid, &OnTryActivateAbility) {
            let ability = context.read_ability(owner_uid);
            ability.on_activate(context, owner_uid);
            EventResolver::broadcast_event(context, owner_uid, &OnAbilityActivated, (), None);
            SUCCESS
        } else {
            FAILURE
        }
    }

}