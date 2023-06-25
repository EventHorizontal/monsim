pub mod choice;
pub mod context;
pub mod game_mechanics;
pub mod battle_constants;
pub mod prng;
pub mod helpers;

mod event;
mod ordering;

pub use helpers::*;
pub use action::SecondaryAction;
pub use battle_builder_macro::build_battle;
pub use choice::*;
pub use context::*;
pub use event::{
    event_dex, ActivationOrder, EventResolver, CompositeEventResponder, EventFilterOptions,
    InBattleEvent, EventResponder, TargetFlags, DEFAULT_RESPONSE,
};
pub use game_mechanics::*;
pub use battle_constants::*;

use prng::Prng;

type TurnResult = Result<(), SimError>;

#[derive(Debug, PartialEq, Eq)]
pub enum SimError {
    InvalidStateError(String),
    InputError(String),
}

#[derive(Debug)]
pub struct BattleSimulator {
    pub battle: Battle,
    pub turn_number: u8,
}

impl BattleSimulator {
    pub fn new(battle: Battle) -> Self {
        BattleSimulator {
            battle,
            turn_number: 0,
        }
    }

    pub fn simulate_turn(&mut self, mut chosen_actions: ChosenActions) -> TurnResult {
        // `simulate_turn` should only call primary actions, by design.
        use action::PrimaryAction;

        self.increment_turn_number()?;

        self.battle
            .push_messages(&[&format!["Turn {}", self.turn_number], &EMPTY_LINE]);

        ordering::context_sensitive_sort_by_activation_order(&mut self.battle, &mut chosen_actions);

        let mut result = Ok(());
        for chosen_action in chosen_actions.into_iter() {
            self.battle.current_action = Some(chosen_action);
            result = match chosen_action {
                ActionChoice::Move {
                    move_uid,
                    target_uid,
                } => match self.battle.move_(move_uid).category() {
                    MoveCategory::Physical | MoveCategory::Special => PrimaryAction::damaging_move(
                        &mut self.battle,
                        move_uid,
                        target_uid,
                    ),
                    MoveCategory::Status => PrimaryAction::status_move(
                        &mut self.battle,
                        move_uid,
                        target_uid,
                    ),
                },
            };
            let maybe_fainted_battler = self.battle.battlers().find(|battler| battler.fainted());
            if let Some(battler) = maybe_fainted_battler {
                self.battle.push_messages(&[
                    &format!["{} fainted!", battler.monster.nickname],
                    &EMPTY_LINE,
                ]);
                self.battle.sim_state = SimState::BattleFinished;
                break;
            };
            self.battle.push_message(&EMPTY_LINE);
        }

        result
    }

    /// Tries to increment turn number and fails if the turn number exceeds 255 after
    /// addition, returning a `SimError::InvalidStateError`.
    fn increment_turn_number(&mut self) -> TurnResult {
        match self.turn_number.checked_add(1) {
            Some(turn_number) => self.turn_number = turn_number,
            None => {
                return Err(SimError::InvalidStateError(String::from(
                    "Turn limit exceeded (Limit = 255 turns)",
                )))
            }
        };
        Ok(())
    }
}

mod action {
    use super::event::broadcast_contexts::*;
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
        pub fn damaging_move(
            battle: &mut Battle,
            move_uid: MoveUID,
            target_uid: BattlerUID,
        ) -> TurnResult {
            let attacker_uid = move_uid.battler_uid;
            let calling_context = MoveContext::new(move_uid, target_uid);

            battle.push_message(&format![
                "{} used {}",
                battle.monster(attacker_uid).nickname,
                battle.move_(move_uid).species.name
            ]);

            if EventResolver::broadcast_trial_event(
                    battle, 
                    attacker_uid, 
                    calling_context, 
                    &OnTryMove, 
            ) == Outcome::Failure {
                battle.push_message(&"The move failed!");
                return Ok(());
            }

            let level = battle.monster(attacker_uid).level;
            let move_power = battle.move_(move_uid).base_power();

            let attackers_attacking_stat;
            let targets_defense_stat;

            match battle.move_(move_uid).category() {
                MoveCategory::Physical => {
                    attackers_attacking_stat =
                        battle.monster(attacker_uid).stats[Stat::PhysicalAttack];
                    targets_defense_stat = battle.monster(target_uid).stats[Stat::PhysicalDefense];
                }
                MoveCategory::Special => {
                    attackers_attacking_stat =
                        battle.monster(attacker_uid).stats[Stat::SpecialAttack];
                    targets_defense_stat = battle.monster(target_uid).stats[Stat::SpecialDefense];
                }
                MoveCategory::Status => {
                    return Err(SimError::InvalidStateError(String::from(
                        "The damaging_move function is not expected to receive status moves.",
                    )))
                }
            }

            let random_multiplier = battle.prng.generate_u16_in_range(85..=100);
            let random_multiplier = ClampedPercent::from(random_multiplier);

            let stab_multiplier = {
                let move_type = battle.move_(move_uid).species.elemental_type;
                if battle.monster(attacker_uid).is_type(move_type) {
                    Percent(125)
                } else {
                    Percent(100)
                }
            };

            let move_type = battle.move_(move_uid).species.elemental_type;
            let target_primary_type = battle.monster(target_uid).species.primary_type;
            let target_secondary_type = battle.monster(target_uid).species.secondary_type;

            let type_matchup_multiplier = if let Some(target_secondary_type) = target_secondary_type
            {
                type_matchup(move_type, target_primary_type)
                    * type_matchup(move_type, target_secondary_type)
            } else {
                type_matchup(move_type, target_primary_type)
            };

            // If the opponent is immune, damage calculation is skipped.
            if type_matchup_multiplier == INEFFECTIVE {
                battle.push_message(&"It was ineffective...");
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
            SecondaryAction::damage(battle, target_uid, damage);
            EventResolver::broadcast_event(
                battle, 
                attacker_uid, 
                MoveContext::new(move_uid, target_uid), 
                &OnDamageDealt, 
                (), 
                None
            );

            let type_effectiveness = match type_matchup_multiplier {
                Percent(25) | Percent(50) => "not very effective",
                Percent(100) => "effective",
                Percent(200) | Percent(400) => "super effective",
                value => {
                    let type_multiplier_as_float = value.0 as f64 / 100.0f64;
                    return Err(SimError::InvalidStateError(format![
                        "Type Effectiveness Multiplier is unexpectedly {type_multiplier_as_float}",
                    ]))
                }
            };
            battle.push_message(&format!["It was {}!", type_effectiveness]);
            battle.push_message(&format![
                "{} took {} damage!",
                battle.monster(target_uid).nickname,
                damage
            ]);
            battle.push_message(&format![
                "{} has {} health left.",
                battle.monster(target_uid).nickname,
                battle.monster(target_uid).current_health
            ]);

            Ok(())
        }

        pub fn status_move(
            battle: &mut Battle,
            move_uid: MoveUID,
            target_uid: BattlerUID,
        ) -> TurnResult {
            let attacker_uid = move_uid.battler_uid;
            let attacker = battle.monster(attacker_uid);
            let move_ = *battle.move_(move_uid);
            let calling_context = MoveContext::new(move_uid, target_uid);

            battle.push_message(&format![
                "{} used {}",
                attacker.nickname, move_.species.name
            ]);

            if EventResolver::broadcast_trial_event(
                    battle, 
                    attacker_uid, 
                    MoveContext::new(move_uid, target_uid), 
                    &OnTryMove, 
            ) == Outcome::Failure {
                battle.push_message(&"The move failed!");
                return Ok(());
            }

            move_.on_activate(battle, attacker_uid, target_uid);
            EventResolver::broadcast_event(
                battle, 
                attacker_uid, 
                calling_context, 
                &OnStatusMoveUsed, 
                (), 
                None
            );

            Ok(())
        }
    }

    impl SecondaryAction {
        /// **Secondary Action** This action can only be triggered by other Actions.
        ///
        /// Deducts `damage` from HP of target corresponding to `target_uid`.
        ///
        /// This function should be used when an amount of damage has already been calculated,
        /// and the only thing left to do is to deduct it from the HP of the target.
        pub fn damage(battle: &mut Battle, target_uid: BattlerUID, damage: u16) {
            battle.monster_mut(target_uid).current_health = battle
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
            battle: &mut Battle,
            ability_holder_uid: BattlerUID,
        ) -> Outcome {
            let calling_context = AbilityContext::new(ability_holder_uid);
            if EventResolver::broadcast_trial_event(
                battle, 
                ability_holder_uid, 
                calling_context, 
                &OnTryActivateAbility, 
            ) == Outcome::Success {
                let ability = *battle.ability(ability_holder_uid);
                ability.on_activate(battle, ability_holder_uid);
                EventResolver::broadcast_event(
                    battle, 
                    ability_holder_uid, 
                    calling_context, 
                    &OnAbilityActivated, 
                    (), 
                    None
                );
                Outcome::Success
            } else {
                Outcome::Failure
            }
        }

        /// **Secondary Action** This action can only be triggered by other Actions.
        ///
        /// Resolves raising the `stat` stat of the battler corresponding to `battler_uid` by `number_of_stages`. The stat cannot be HP.
        ///
        /// Returns a `bool` indicating whether the stat raising succeeded.
        pub fn raise_stat(
            battle: &mut Battle,
            battler_uid: BattlerUID,
            stat: Stat,
            number_of_stages: u8,
        ) -> Outcome {
            if EventResolver::broadcast_trial_event(
                battle, 
                battler_uid, 
                (), 
                &OnTryRaiseStat, 
            ) == Outcome::Success {
                let effective_stages = battle
                    .monster_mut(battler_uid)
                    .stat_modifiers
                    .raise_stat(stat, number_of_stages);
                battle.push_message(&format![
                    "{}\'s {:?} was raised by {} stage(s)!",
                    battle.monster(battler_uid).name(),
                    stat,
                    effective_stages
                ]);
                Outcome::Success
            } else {
                battle.push_message(&format![
                    "{}'s stats were not raised.",
                    battle.monster(battler_uid).name()
                ]);
                Outcome::Failure
            }
        }

        /// **Secondary Action** This action can only be triggered by other Actions.
        ///
        /// Resolves lowering the `stat` stat of the battler corresponding to `battler_uid` by `number_of_stages`. The stat cannot be HP.
        ///
        /// Returns a `bool` indicating whether the stat lowering succeeded.
        pub fn lower_stat(
            battle: &mut Battle,
            battler_uid: BattlerUID,
            stat: Stat,
            number_of_stages: u8,
        ) -> Outcome {
            if EventResolver::broadcast_trial_event(
                battle, 
                battler_uid, 
                (), 
                &OnTryLowerStat, 
            ) == Outcome::Success {
                let effective_stages = battle
                    .monster_mut(battler_uid)
                    .stat_modifiers
                    .lower_stat(stat, number_of_stages);
                battle.push_message(&format![
                    "{}\'s {:?} was lowered by {} stage(s)!",
                    battle.monster(battler_uid).name(),
                    stat,
                    effective_stages
                ]);
                Outcome::Success
            } else {
                battle.push_message(&format![
                    "{}'s stats were not lowered.",
                    battle.monster(battler_uid).name()
                ]);
                Outcome::Failure
            }
        }
    }
}
