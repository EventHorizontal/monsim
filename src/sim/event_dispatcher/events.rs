use super::*;
use monsim_utils::{Percent, NOTHING};

pub(crate) fn trigger_on_try_move_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_try_move,
        event_context,
    )
}

pub(crate) fn trigger_on_move_used_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_damaging_move_used,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_calculate_accuracy_stage_event(
    battle: &mut Battle,
    broadcaster_id: MonsterID,
    event_context: MoveHitContext,
    original_accuracy_stage: i8,
) -> i8 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_accuracy_stage,
        event_context,
        original_accuracy_stage,
        None,
    )
}

pub(crate) fn trigger_on_calculate_evasion_stage_event(
    battle: &mut Battle,
    broadcaster_id: MonsterID,
    event_context: MoveHitContext,
    original_evasion_stage: i8,
) -> i8 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_evasion_stage,
        event_context,
        original_evasion_stage,
        None,
    )
}

pub(crate) fn trigger_on_calculate_crit_stage_event(
    battle: &mut Battle,
    broadcaster_id: MonsterID,
    event_context: MoveHitContext,
    original_crit_stage: u8,
) -> u8 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_crit_stage,
        event_context,
        original_crit_stage,
        None,
    )
}

pub(crate) fn trigger_on_calculate_crit_damage_multiplier_event(
    battle: &mut Battle,
    broadcaster_id: MonsterID,
    event_context: MoveHitContext,
    default: Percent,
) -> Percent {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_crit_damage_multiplier,
        event_context,
        default,
        None,
    )
}

pub(crate) fn trigger_on_try_move_hit_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_try_move_hit,
        event_context,
    )
}

pub(crate) fn trigger_on_move_hit_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_move_hit,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_calculate_attack_stat_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_attack_stat,
        event_context,
        default,
        None,
    )
}

pub(crate) fn trigger_on_calculate_attack_stage_event(
    battle: &mut Battle,
    broadcaster_id: MonsterID,
    event_context: MoveHitContext,
    original_attack_stage: i8,
) -> i8 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_attack_stage,
        event_context,
        original_attack_stage,
        None,
    )
}

pub(crate) fn trigger_on_calculate_defense_stat_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_defense_stat,
        event_context,
        default,
        None,
    )
}

pub(crate) fn trigger_on_calculate_defense_stage_event(
    battle: &mut Battle,
    broadcaster_id: MonsterID,
    event_context: MoveHitContext,
    original_defense_stage: i8,
) -> i8 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_defense_stage,
        event_context,
        original_defense_stage,
        None,
    )
}

pub(crate) fn trigger_on_modify_damage_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, current_damage: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_modify_damage,
        event_context,
        current_damage,
        None,
    )
}

pub(crate) fn trigger_on_damage_dealt_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_damage_dealt,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_activate_ability_event(
    battle: &mut Battle,
    broadcaster_id: MonsterID,
    event_context: AbilityActivationContext,
) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_try_activate_ability,
        event_context,
    )
}

pub(crate) fn trigger_on_ability_activated_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: AbilityActivationContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_ability_activated,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_modify_base_accuracy_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, base_accuracy: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_calculate_accuracy,
        event_context,
        base_accuracy,
        None,
    )
}

pub(crate) fn trigger_on_try_stat_change_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: StatChangeContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_try_stat_change,
        event_context,
    )
}

pub(crate) fn trigger_on_modify_stat_change_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: StatChangeContext) -> i8 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_modify_stat_change,
        event_context,
        event_context.number_of_stages,
        None,
    )
}

pub(crate) fn trigger_on_stat_changed_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: StatChangeContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_stat_changed,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_inflict_volatile_status_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_try_inflict_volatile_status,
        event_context,
    )
}

pub(crate) fn trigger_on_volatile_status_inflicted_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_volatile_status_inflicted,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_inflict_persistent_status_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_try_inflict_persistent_status,
        event_context,
    )
}

pub(crate) fn trigger_on_persistent_status_inflicted_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_persistent_status_inflicted,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_use_held_item_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_try_use_held_item,
        event_context,
    )
}

pub(crate) fn trigger_on_held_item_used_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_held_item_used,
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_turn_end_event(battle: &mut Battle, broadcaster_id: Nothing, event_context: Nothing) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_cache| &mut event_handler_cache.on_turn_end,
        event_context,
        NOTHING,
        None,
    )
}
