use super::*;
use monsim_utils::NOTHING;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerSet {
    /// This EventHandler is triggered when a move is about to be used. This EventHandler is to return an `Outcome`
    /// indicating whether the move should succeed.
    pub on_try_move: Option<EventHandler<Outcome<Nothing>, MoveUseContext>>,
    /// This EventHandler is triggered when a move is used successfully.
    pub on_move_used: Option<EventHandler<Nothing, MoveUseContext>>,
    /// This EventHandler is meant only to be a base for `on_damaging_move_used` and `on_status_move_used`.
    pub on_damaging_move_used: Option<EventHandler<Nothing, MoveUseContext>>,
    /// This EventHandler is triggered when a status move is used successfully.
    pub on_status_move_used: Option<EventHandler<Nothing, MoveUseContext>>,
    /// This EventHandler is triggered after the accuracy to be used in move miss calculation is calculated. This
    /// EventHandler is to return a `u16` representing a possibly modified _base_ accuracy to be used by the move.
    /// If the EventHandler wishes to leave the accuracy unchanged, say if a certain condition is met, then it can
    /// pass back the original accuracy, which is relayed to this EventHandler.
    pub on_calculate_accuracy: Option<EventHandler<u16, MoveHitContext>>,
    pub on_calculate_accuracy_stage: Option<EventHandler<i8, MoveHitContext>>,
    pub on_calculate_evasion_stage: Option<EventHandler<i8, MoveHitContext>>,
    /// This EventHandler is triggered when a individual move hit is about to be performed. This EventHandler is to
    /// return an `Outcome` indicating whether the hit should succeed.
    pub on_try_move_hit: Option<EventHandler<Outcome<Nothing>, MoveHitContext>>,
    /// This EventHandler is triggered when a hit has been performed successfully.
    pub on_move_hit: Option<EventHandler<Nothing, MoveHitContext>>,
    /// This EventHandler is triggered when a move is calculating the attack stat to be used. This EventHandler is to
    /// return a `u16` indicating a possibly modified attack stat to be used. If the EventHandler wishes to
    /// leave the attack unchanged, say if a certain condition is met, then it can pass back the original attack
    /// stat, which is relayed to this EventHandler.
    pub on_calculate_attack_stat: Option<EventHandler<u16, MoveHitContext>>,
    /// This EventHandler is triggered when a move is calculating the defense stat to be used. This EventHandler is to
    /// return a `u16` indicating a possibly modified defense stat to be used. If the EventHandler wishes to
    /// leave the defense unchanged, say if a certain condition is met, then it can pass back the original defense
    /// stat, which is relayed to this EventHandler.
    pub on_calculate_defense_stat: Option<EventHandler<u16, MoveHitContext>>,
    /// This EventHandler is triggered after a move's damage is calculated, giving the opportunity for the final damage
    /// to be modified. This EventHandler is to return a `u16` indicating a possibly modified damage value. If the
    /// EventHandler wishes to leave the damage unchanged, say if a certain condition is met, then it can pass back
    /// the original damage, which is relayed to this EventHandler.
    pub on_modify_damage: Option<EventHandler<u16, MoveHitContext>>,
    /// This EventHandler is triggered after a move's damage has been dealt successfully.
    pub on_damage_dealt: Option<EventHandler<Nothing, Nothing>>,
    /// This EventHandler is triggered when an ability is about to be activated. The EventHandler is to
    /// return an `Outcome` indicating whether the ability activation should succeed.
    pub on_try_activate_ability: Option<EventHandler<Outcome<Nothing>, AbilityActivationContext>>,
    /// This EventHandler is triggered after an ability successfully activates.
    pub on_ability_activated: Option<EventHandler<Nothing, AbilityActivationContext>>,
    /// This EventHandler is triggered when a stat is about to be changed. This EventHandler is to return an `Outcome`
    /// representing whether the stat change should succeed.
    pub on_try_stat_change: Option<EventHandler<Outcome<Nothing>, StatChangeContext>>,
    /// This EventHandler is triggered when a stat is changed, allowing for the stat change to be modified.
    pub on_modify_stat_change: Option<EventHandler<i8, StatChangeContext>>,
    /// This EventHandler is triggered after a stat is changed.
    pub on_stat_changed: Option<EventHandler<Nothing, StatChangeContext>>,
    /// This EventHandler is triggered when a volatile status is about to be inflicted on a Monster. This EventHandler
    /// is to return and `Outcome` representing whether the infliction of the volatile status should succeed.
    pub on_try_inflict_volatile_status: Option<EventHandler<Outcome<Nothing>, Nothing>>,
    /// This EventHandler is triggered after a volatile status has been successfully inflicted.
    pub on_volatile_status_inflicted: Option<EventHandler<Nothing, Nothing>>,
    /// This EventHandler is triggered when a persistent status is about to be inflicted on a Monster. This EventHandler
    /// is to return and `Outcome` representing whether the infliction of the persistent status should succeed.
    pub on_try_inflict_persistent_status: Option<EventHandler<Outcome<Nothing>, Nothing>>,
    /// This EventHandler is triggered after a presistent status has been successfully inflicted.
    pub on_persistent_status_inflicted: Option<EventHandler<Nothing, Nothing>>,
    /// This EventHandler is triggered when a held item is about to be used. This EventHandler
    /// is to return and `Outcome` representing whether the use of the held item should succeed.
    pub on_try_use_held_item: Option<EventHandler<Outcome<Nothing>, ItemUseContext>>,
    /// This EventHandler is triggered when a held item is used successfully.
    pub on_held_item_used: Option<EventHandler<Nothing, ItemUseContext>>,
    /// This EventHandler is triggered at the end of each turn. This is a _temporal event_, such that it has no broadcaster.
    pub on_turn_end: Option<EventHandler<Nothing, Nothing, Nothing>>,
}

pub(super) const DEFAULT_EVENT_HANDLERS: EventHandlerSet = EventHandlerSet {
    on_try_move: None,
    on_move_used: None,
    on_damaging_move_used: None,
    on_status_move_used: None,
    on_calculate_accuracy: None,
    on_calculate_accuracy_stage: None,
    on_calculate_evasion_stage: None,
    on_try_move_hit: None,
    on_move_hit: None,
    on_calculate_attack_stat: None,
    on_calculate_defense_stat: None,
    on_modify_damage: None,
    on_damage_dealt: None,
    on_try_activate_ability: None,
    on_ability_activated: None,
    on_try_stat_change: None,
    on_modify_stat_change: None,
    on_stat_changed: None,
    on_try_inflict_volatile_status: None,
    on_volatile_status_inflicted: None,
    on_try_inflict_persistent_status: None,
    on_persistent_status_inflicted: None,
    on_try_use_held_item: None,
    on_held_item_used: None,
    on_turn_end: None,
};

impl EventHandlerSet {
    pub const fn empty() -> Self {
        DEFAULT_EVENT_HANDLERS
    }
}

pub(crate) fn trigger_on_try_move_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(battle, broadcaster_id, |event_handler_set| vec![(event_handler_set.on_try_move)], event_context)
}

pub(crate) fn trigger_on_damaging_move_used_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_damaging_move_used), (event_handler_set.on_move_used)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_status_move_used_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_status_move_used), (event_handler_set.on_move_used)],
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
        |event_handler_set| vec![(event_handler_set.on_calculate_accuracy_stage)],
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
        |event_handler_set| vec![(event_handler_set.on_calculate_evasion_stage)],
        event_context,
        original_evasion_stage,
        None,
    )
}

pub(crate) fn trigger_on_try_move_hit_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_move_hit)],
        event_context,
    )
}

pub(crate) fn trigger_on_move_hit_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_move_hit)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_calculate_attack_stat_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_calculate_attack_stat)],
        event_context,
        default,
        None,
    )
}

pub(crate) fn trigger_on_calculate_defense_stat_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_calculate_defense_stat)],
        event_context,
        default,
        None,
    )
}

pub(crate) fn trigger_on_modify_damage_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, current_damage: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_modify_damage)],
        event_context,
        current_damage,
        None,
    )
}

pub(crate) fn trigger_on_damage_dealt_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_damage_dealt)],
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
        |event_handler_set| vec![(event_handler_set.on_try_activate_ability)],
        event_context,
    )
}

pub(crate) fn trigger_on_ability_activated_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: AbilityActivationContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_ability_activated)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_modify_base_accuracy_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: MoveHitContext, base_accuracy: u16) -> u16 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_calculate_accuracy)],
        event_context,
        base_accuracy,
        None,
    )
}

pub(crate) fn trigger_on_try_stat_change_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: StatChangeContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_stat_change)],
        event_context,
    )
}

pub(crate) fn trigger_on_modify_stat_change_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: StatChangeContext) -> i8 {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_modify_stat_change)],
        event_context,
        event_context.number_of_stages,
        None,
    )
}

pub(crate) fn trigger_on_stat_changed_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: StatChangeContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_stat_changed)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_inflict_volatile_status_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_inflict_volatile_status)],
        event_context,
    )
}

pub(crate) fn trigger_on_volatile_status_inflicted_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_volatile_status_inflicted)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_inflict_persistent_status_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_inflict_persistent_status)],
        event_context,
    )
}

pub(crate) fn trigger_on_persistent_status_inflicted_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: Nothing) {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_persistent_status_inflicted)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_try_use_held_item_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Outcome<Nothing> {
    EventDispatcher::dispatch_trial_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_try_use_held_item)],
        event_context,
    )
}

pub(crate) fn trigger_on_held_item_used_event(battle: &mut Battle, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_held_item_used)],
        event_context,
        NOTHING,
        None,
    )
}

pub(crate) fn trigger_on_turn_end_event(battle: &mut Battle, broadcaster_id: Nothing, event_context: Nothing) -> Nothing {
    EventDispatcher::dispatch_event(
        battle,
        broadcaster_id,
        |event_handler_set| vec![(event_handler_set.on_turn_end)],
        event_context,
        NOTHING,
        None,
    )
}
