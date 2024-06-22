use super::MechanicID;
use monsim_macros::Event;
use monsim_utils::{Nothing, Outcome, Percent};

use crate::MonsterID;

use super::{contexts::*, Event, EventHandler, EventListener};

#[derive(Event)]
#[returns(Outcome)]
#[context(MoveUseContext)]
#[handler(on_try_move_handler)]
pub struct OnTryMoveEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(MoveUseContext)]
#[handler(on_move_used_handler)]
pub struct OnMoveUsedEvent;

#[derive(Event)]
#[returns(u16)]
#[context(MoveHitContext)]
#[handler(on_calculate_accuracy_handler)]
pub struct OnCalculateAccuracyEvent;

#[derive(Event)]
#[returns(i8)]
#[context(MoveHitContext)]
#[handler(on_calculate_accuracy_stage_handler)]
pub struct OnCalculateAccuracyStageEvent;

#[derive(Event)]
#[returns(i8)]
#[context(MoveHitContext)]
#[handler(on_calculate_evasion_stage_handler)]
pub struct OnCalculateEvasionStageEvent;

#[derive(Event)]
#[returns(u8)]
#[context(MoveHitContext)]
#[handler(on_calculate_crit_stage_handler)]
pub struct OnCalculateCritStageEvent;

#[derive(Event)]
#[returns(Percent)]
#[context(MoveHitContext)]
#[handler(on_calculate_crit_damage_multiplier_handler)]
pub struct OnCalculateCritDamageMultiplierEvent;

#[derive(Event)]
#[returns(Outcome)]
#[context(MoveHitContext)]
#[handler(on_try_move_hit_handler)]
pub struct OnTryMoveHitEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(MoveHitContext)]
#[handler(on_move_hit_handler)]
pub struct OnMoveHitEvent;

#[derive(Event)]
#[returns(u16)]
#[context(MoveHitContext)]
#[handler(on_calculate_attack_stat_handler)]
pub struct OnCalculateAttackStatEvent;

#[derive(Event)]
#[returns(i8)]
#[context(MoveHitContext)]
#[handler(on_calculate_attack_stage_handler)]
pub struct OnCalculateAttackStageEvent;

#[derive(Event)]
#[returns(u16)]
#[context(MoveHitContext)]
#[handler(on_calculate_defense_stat_handler)]
pub struct OnCalculateDefenseStatEvent;

#[derive(Event)]
#[returns(i8)]
#[context(MoveHitContext)]
#[handler(on_calculate_defense_stage_handler)]
pub struct OnCalculateDefenseStageEvent;

#[derive(Event)]
#[returns(u16)]
#[context(MoveHitContext)]
#[handler(on_modify_damage_handler)]
pub struct OnModifyDamageEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(Nothing)]
#[handler(on_damage_dealt_handler)]
pub struct OnDamageDealtEvent;

#[derive(Event)]
#[returns(Outcome)]
#[context(AbilityActivationContext)]
#[handler(on_try_activate_ability_handler)]
pub struct OnTryActivateAbilityEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(AbilityActivationContext)]
#[handler(on_ability_activated_handler)]
pub struct OnAbilityActivatedEvent;

#[derive(Event)]
#[returns(Outcome)]
#[context(StatChangeContext)]
#[handler(on_try_stat_change_handler)]
pub struct OnTryStatChangeEvent;

#[derive(Event)]
#[returns(i8)]
#[context(StatChangeContext)]
#[handler(on_modify_stat_change_handler)]
pub struct OnModifyStatChangeEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(StatChangeContext)]
#[handler(on_stat_changed_handler)]
pub struct OnStatChangedEvent;

#[derive(Event)]
#[returns(Outcome)]
#[context(Nothing)]
#[handler(on_try_inflict_volatile_status_handler)]
pub struct OnTryInflictVolatileStatusEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(Nothing)]
#[handler(on_volatile_status_inflicted_handler)]
pub struct OnVolatileStatusInflictedEvent;

#[derive(Event)]
#[returns(Outcome)]
#[context(Nothing)]
#[handler(on_try_inflict_persistent_status_handler)]
pub struct OnTryInflictPersistentStatusEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(Nothing)]
#[handler(on_persistent_status_inflicted_handler)]
pub struct OnPersistentStatusInflictedEvent;

#[derive(Event)]
#[returns(Outcome)]
#[context(ItemUseContext)]
#[handler(on_try_use_held_item_handler)]
pub struct OnTryUseHeldItemEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(ItemUseContext)]
#[handler(on_held_item_used_handler)]
pub struct OnHeldItemUsedEvent;

#[derive(Event)]
#[returns(Nothing)]
#[context(Nothing)]
#[handler(on_turn_end_handler)]
#[no_broadcaster]
pub struct OnTurnEndEvent;
