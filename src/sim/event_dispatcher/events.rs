use super::*;
use monsim_utils::NOTHING;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerSet {
    pub on_try_move: Option<EventHandler<Outcome<Nothing>, MoveUseContext, MonsterID>>,
    pub on_move_used: Option<EventHandler<Nothing, MoveUseContext, MonsterID>>,
    /// This is meant only to be a base event for `on_damaging_move_used` and `on_status_move_used`.
    pub on_damaging_move_used: Option<EventHandler<Nothing, MoveUseContext, MonsterID>>,
    pub on_status_move_used: Option<EventHandler<Nothing, MoveUseContext, MonsterID>>,
    pub on_try_move_hit: Option<EventHandler<Outcome<Nothing>, MoveHitContext, MonsterID>>,
    pub on_move_hit: Option<EventHandler<Nothing, MoveHitContext, MonsterID>>,
    pub on_calculate_attack_stat: Option<EventHandler<u16, MoveHitContext, MonsterID>>,
    pub on_calculate_defense_stat: Option<EventHandler<u16, MoveHitContext, MonsterID>>,
    pub on_modify_damage: Option<EventHandler<u16, MoveHitContext, MonsterID>>,
    pub on_damage_dealt: Option<EventHandler<Nothing, Nothing, MonsterID>>,
    pub on_try_activate_ability: Option<EventHandler<Outcome<Nothing>, AbilityActivationContext, MonsterID>>,
    pub on_ability_activated: Option<EventHandler<Nothing, AbilityActivationContext, MonsterID>>,
    pub on_modify_accuracy: Option<EventHandler<u16, MoveHitContext, MonsterID>>,
    pub on_try_raise_stat: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
    pub on_try_lower_stat: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
    pub on_try_add_volatile_status: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
    pub on_try_add_permanent_status: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
    pub on_try_use_held_item: Option<EventHandler<Outcome<Nothing>, ItemUseContext, MonsterID>>,
    pub on_held_item_used: Option<EventHandler<Nothing, ItemUseContext, MonsterID>>,
    pub on_turn_end: Option<EventHandler<Nothing, Nothing, Nothing>>,
}

pub(super) const DEFAULT_EVENT_HANDLERS: EventHandlerSet = EventHandlerSet {
    on_try_move: None,
    on_move_used: None,
    on_damaging_move_used: None,
    on_status_move_used: None,
    on_try_move_hit: None,
    on_move_hit: None,
    on_calculate_attack_stat: None,
    on_calculate_defense_stat: None,
    on_modify_damage: None,
    on_damage_dealt: None,
    on_try_activate_ability: None,
    on_ability_activated: None,
    on_modify_accuracy: None,
    on_try_raise_stat: None,
    on_try_lower_stat: None,
    on_try_add_volatile_status: None,
    on_try_add_permanent_status: None,
    on_try_use_held_item: None,
    on_held_item_used: None,
    on_turn_end: None,
};

impl EventHandlerSet {
    pub const fn empty() -> Self {
        DEFAULT_EVENT_HANDLERS
    }
}

pub(crate) fn trigger_on_try_move_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(battle, broadcaster_id, |event_handler_set| vec![(event_handler_set.on_try_move)], event_context)
}

pub(crate) fn trigger_on_damaging_move_used_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_damaging_move_used), (event_handler_set.on_move_used)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_status_move_used_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_status_move_used), (event_handler_set.on_move_used)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_move_hit_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_move_hit)],
        event_context,
    )
}

pub(crate) fn trigger_on_move_hit_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_move_hit)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_calculate_attack_stat_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_calculate_attack_stat)],
        event_context,
        default,
        None,
    )
}

pub(crate) fn trigger_on_calculate_defense_stat_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_calculate_defense_stat)],
        event_context,
        default,
        None,
    )
}

pub(crate) fn trigger_on_modify_damage_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveHitContext, current_damage: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_modify_damage)],
        event_context,
        current_damage,
        None,
    )
}

pub(crate) fn trigger_on_damage_dealt_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: Nothing) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_damage_dealt)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_activate_ability_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: AbilityActivationContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_activate_ability)],
        event_context,
    )
}

pub(crate) fn trigger_on_ability_activated_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: AbilityActivationContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_ability_activated)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_modify_accuracy_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: MoveHitContext, base_accuracy: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_modify_accuracy)],
        event_context,
        base_accuracy,
        None,
    )
}

pub(crate) fn trigger_on_try_raise_stat_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_raise_stat)],
        event_context,
    )
}

pub(crate) fn trigger_on_try_lower_stat_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_lower_stat)],
        event_context,
    )
}

pub(crate) fn trigger_on_try_add_volatile_status_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_add_volatile_status)],
        event_context,
    )
}

pub(crate) fn trigger_on_try_add_permanent_status_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_add_permanent_status)],
        event_context,
    )
}

pub(crate) fn trigger_on_try_use_held_item_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_use_held_item)],
        event_context,
    )
}

pub(crate) fn trigger_on_held_item_used_event(battle: &mut BattleState, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_held_item_used)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_turn_end_event(battle: &mut BattleState, broadcaster_id: Nothing, event_context: Nothing) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_turn_end)],
        event_context,
        NOTHING,
        None,
    )
}
