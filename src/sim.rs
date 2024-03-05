pub mod battle;
pub mod battle_constants;
pub mod choice;
pub mod game_mechanics;
pub mod prng;

mod event;
mod ordering;

use std::{error::Error, fmt::Display};

pub use action::SecondaryAction;
pub use battle::*;
pub use battle_builder_macro::build_battle;
pub use battle_constants::*;
pub use choice::*;
pub use event::{
    broadcast_contexts::*, event_dex, ActivationOrder, EventHandlerDeck, EventFilteringOptions, EventDispatcher, EventHandler, InBattleEvent, TargetFlags,
    DEFAULT_RESPONSE,
};
pub use game_mechanics::*;
pub use monsim_utils::{self as utils, Outcome, Percent, ClampedPercent, Ally, Opponent};
pub(crate) use utils::{not, NOTHING, Nothing}; // For internal use

use prng::Prng;

type TurnResult = Result<(), SimError>;

#[derive(Debug, PartialEq, Eq)]
pub enum SimError {
    InvalidStateReached(String),
}

impl Error for SimError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimError::InvalidStateReached(message) => write!(f, "{}", message),
        }
    }
}

/// The main engine behind `monsim`. This struct is a namespace for all the simulator functionality. It contains no data, just functions that transform a `Battle` from one state to another.
#[derive(Debug)]
pub struct BattleSimulator;

impl BattleSimulator {

    pub fn simulate_turn(battle: &mut Battle, chosen_actions: ChosenActionsForTurn) -> TurnResult {
        // `simulate_turn` should only call primary actions, by design.
        use action::PrimaryAction;
        
        assert!(not!(battle.is_finished), "The simulator cannot be called on a finished battle.");

        battle.message_log.set_last_turn_cursor_to_log_length();
        battle.increment_turn_number()
            .map_err(|message| { SimError::InvalidStateReached(String::from(message))})?;
        
        battle.message_log.extend(&[
            "---", 
            EMPTY_LINE,
            &format!["Turn {turn_number}", turn_number = battle.turn_number], 
            EMPTY_LINE
            ]
        );

        let mut chosen_actions = chosen_actions.as_array();
        ordering::sort_action_choices_by_activation_order(battle, &mut chosen_actions);

        'turn: for chosen_action in chosen_actions.into_iter() {
            
            match chosen_action {
                FullySpecifiedAction::Move { move_uid, target_uid } => match battle.move_(move_uid).category() {
                    MoveCategory::Physical | MoveCategory::Special => PrimaryAction::damaging_move(battle, move_uid, target_uid),
                    MoveCategory::Status => PrimaryAction::status_move(battle, move_uid, target_uid),
                },
                FullySpecifiedAction::SwitchOut { switcher_uid, switchee_uid } => {
                    PrimaryAction::switch_out(battle, switcher_uid, switchee_uid)
                }
            }?;

            // Check if a Monster fainted this turn
            let maybe_fainted_acitve_monster = battle.monsters()
                .find(|monster| battle.monster(monster.uid).is_fainted && battle.is_active_monster(monster.uid));
            
            if let Some(fainted_active_monster) = maybe_fainted_acitve_monster {
                
                battle.message_log.extend(&[
                    &format!["{fainted_monster} fainted!", fainted_monster = fainted_active_monster.name()], 
                    EMPTY_LINE
                ]);
                
                // Check if any of the teams is out of usable Monsters
                let are_all_ally_team_monsters_fainted = battle.ally_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted });
                let are_all_opponent_team_monsters_fainted = battle.opponent_team()
                    .monsters()
                    .iter()
                    .all(|monster| { monster.is_fainted });
                
                if are_all_ally_team_monsters_fainted {
                    battle.is_finished = true;
                    battle.message_log.push_str("Opponent Team won!");
                    break 'turn;
                } 
                if are_all_opponent_team_monsters_fainted {
                    battle.is_finished = true;
                    battle.message_log.push_str("Ally Team won!");
                    break 'turn;
                }
            };

            battle.message_log.push_str(EMPTY_LINE);
        }

        if battle.is_finished {
            battle.message_log.extend(&[EMPTY_LINE, "The battle ended."]);
        }
        battle.message_log.extend(&["---", EMPTY_LINE]);

        Ok(NOTHING)
    }
    
    pub(crate) fn switch_out_between_turns(battle: &mut Battle, active_monster_uid: MonsterUID, benched_monster_uid: MonsterUID) -> TurnResult {
        action::PrimaryAction::switch_out(battle, active_monster_uid, benched_monster_uid)
    }
}

mod action {
    use crate::matchup;

    use super::event_dex::*;
    use super::*;

    /// Primary Actions are functions that are meant to be called by the
    /// simulator to initiate a monster's turn.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct PrimaryAction;

    /// Secondary Actions are meant to be called by other Actions (both Primary
    /// and Secondary). This leads to a chain-reaction of Actions. It is up to the
    /// user to avoid making loops of actions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SecondaryAction;

    impl PrimaryAction {
        /// Primary action: A monster's turn may be initiated by this Action.
        ///
        /// Calculates and applies the effects of a damaging move
        /// corresponding to `move_uid` being used on `target_uid`
        pub fn damaging_move(battle: &mut Battle, move_uid: MoveUID, target_uid: MonsterUID) -> TurnResult {
            let attacker_uid = move_uid.owner_uid;
            let calling_context = MoveUsed::new(move_uid, target_uid);

            battle.message_log.push(format![
                "{attacker} used {_move}",
                attacker = battle.monster(attacker_uid).name(),
                _move = battle.move_(move_uid).species.name
            ]);

            if EventDispatcher::dispatch_trial_event(battle, attacker_uid, calling_context, &OnTryMove) == Outcome::Failure {
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

            let random_multiplier = battle.prng.generate_u16_in_range(85..=100);
            let random_multiplier = ClampedPercent::from(random_multiplier);

            let stab_multiplier = {
                let move_type = battle.move_(move_uid).species.elemental_type;
                if battle.monster(attacker_uid).is_type(move_type) { Percent(125) } else { Percent(100) }
            };

            let move_type = battle.move_(move_uid).species.elemental_type;
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
            SecondaryAction::damage(battle, target_uid, damage);
            EventDispatcher::dispatch_event(battle, attacker_uid, calling_context, &OnDamageDealt, NOTHING, None);

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

        pub fn status_move(battle: &mut Battle, move_uid: MoveUID, target_uid: MonsterUID) -> TurnResult {
            let attacker_uid = move_uid.owner_uid;
            let calling_context = MoveUsed::new(move_uid, target_uid);

            battle.message_log.push(format![
                "{attacker} used {move_}",
                attacker = battle.monster(attacker_uid).name(),
                move_ = battle.move_(move_uid).species.name
            ]);

            if EventDispatcher::dispatch_trial_event(battle, attacker_uid, MoveUsed::new(move_uid, target_uid), &OnTryMove) == Outcome::Failure {
                battle.message_log.push_str("The move failed!");
                return Ok(NOTHING);
            }

            {
                let move_ = *battle.move_(move_uid);
                move_.on_activate(battle, attacker_uid, target_uid);
            }

            EventDispatcher::dispatch_event(battle, attacker_uid, calling_context, &OnStatusMoveUsed, NOTHING, None);

            Ok(NOTHING)
        }

        pub fn switch_out(battle: &mut Battle, active_monster_uid: MonsterUID, benched_monster_uid: MonsterUID) -> TurnResult {
            battle.team_mut(active_monster_uid.team_id).active_monster_uid = benched_monster_uid;
            battle.message_log.push(format![
                "{active_monster} switched out! Go {benched_monster}!", 
                active_monster = battle.monster(active_monster_uid).name(),
                benched_monster = battle.monster(benched_monster_uid).name()
            ]);
            Ok(NOTHING)
        }
    }

    impl SecondaryAction {
        /// **SEventDispatcheron** This action can only be triggered by other Actions.
        ///
        /// Deducts `damage` from HP of target corresponding to `target_uid`.
        ///EventDispatcher
        /// This function should be used when an amount of damage has already been calculated,
        /// and the only thing left to do is to deduct it from the HP of the target.
        pub fn damage(battle: &mut Battle, target_uid: MonsterUID, damage: u16) {
            battle.monster_mut(target_uid).current_health = battle.monster(target_uid).current_health.saturating_sub(damage);
            if battle.monster(target_uid).current_health == 0 { battle.monster_mut(target_uid).is_fainted = true; };
        }

        /// **Secondary Action** This action can only be triggered by other Actions.
        ///
        /// Resolves activation of any ability.
        ///
        /// Returns a `Outcome` indicating whether the ability succeeded.
        pub fn activate_ability(battle: &mut Battle, ability_holder_uid: MonsterUID) -> Outcome {
            let calling_context = AbilityUsed::new(ability_holder_uid);

            if EventDispatcher::dispatch_trial_event(battle, ability_holder_uid, calling_context, &OnTryActivateAbility) == Outcome::Success {
                let ability = *battle.ability(ability_holder_uid);
                ability.on_activate(battle, ability_holder_uid);
                EventDispatcher::dispatch_event(battle, ability_holder_uid, calling_context, &OnAbilityActivated, NOTHING, None);
                Outcome::Success
            } else {
                Outcome::Failure
            }
        }

        /// **Secondary Action** This action can only be triggered by other Actions.
        ///
        /// Resolves raising the `stat` stat of the monster corresponding to `monster_uid` by `number_of_stages`. The stat cannot be HP.
        ///
        /// Returns a `bool` indicating whether the stat raising succeeded.
        pub fn raise_stat(battle: &mut Battle, monster_uid: MonsterUID, stat: Stat, number_of_stages: u8) -> Outcome {
            if EventDispatcher::dispatch_trial_event(battle, monster_uid, NOTHING, &OnTryRaiseStat) == Outcome::Success {
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
        pub fn lower_stat(battle: &mut Battle, monster_uid: MonsterUID, stat: Stat, number_of_stages: u8) -> Outcome {
            if EventDispatcher::dispatch_trial_event(battle, monster_uid, NOTHING, &OnTryLowerStat) == Outcome::Success {
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
}
