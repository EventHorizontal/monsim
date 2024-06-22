use monsim_utils::{Nothing, Outcome, Percent};

use crate::MonsterID;

use super::{contexts::*, Event, EventHandler, EventListener};

pub struct OnTryMoveEvent;

impl Event<Outcome, MoveUseContext> for OnTryMoveEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Outcome, MoveUseContext, MonsterID>> {
        event_listener.on_try_move_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Outcome, MoveUseContext, Nothing>> {
        event_listener.on_try_move_handler()
    }
}

pub struct OnMoveUsedEvent;

impl Event<Nothing, MoveUseContext> for OnMoveUsedEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, MoveUseContext, MonsterID>> {
        event_listener.on_move_used_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, MoveUseContext, Nothing>> {
        event_listener.on_move_used_handler()
    }
}

pub struct OnDamagingMoveUsedEvent;

impl Event<Nothing, MoveUseContext> for OnDamagingMoveUsedEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, MoveUseContext, MonsterID>> {
        event_listener.on_damaging_move_used_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, MoveUseContext, Nothing>> {
        event_listener.on_damaging_move_used_handler()
    }
}

pub struct OnStatusMoveUsedEvent;

impl Event<Nothing, MoveUseContext> for OnStatusMoveUsedEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, MoveUseContext, MonsterID>> {
        event_listener.on_status_move_used_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, MoveUseContext, Nothing>> {
        event_listener.on_status_move_used_handler()
    }
}

pub struct OnCalculateAccuracyEvent;

impl Event<u16, MoveHitContext> for OnCalculateAccuracyEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<u16, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_accuracy_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<u16, MoveHitContext, Nothing>> {
        event_listener.on_calculate_accuracy_handler()
    }
}

pub struct OnCalculateAccuracyStageEvent;

impl Event<i8, MoveHitContext> for OnCalculateAccuracyStageEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<i8, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_accuracy_stage_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<i8, MoveHitContext, Nothing>> {
        event_listener.on_calculate_accuracy_stage_handler()
    }
}

pub struct OnCalculateEvasionStageEvent;

impl Event<i8, MoveHitContext> for OnCalculateEvasionStageEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<i8, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_evasion_stage_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<i8, MoveHitContext, Nothing>> {
        event_listener.on_calculate_evasion_stage_handler()
    }
}

pub struct OnCalculateCritStageEvent;

impl Event<u8, MoveHitContext> for OnCalculateCritStageEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<u8, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_crit_stage_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<u8, MoveHitContext, Nothing>> {
        event_listener.on_calculate_crit_stage_handler()
    }
}

pub struct OnCalculateCritDamageMultiplierEvent;

impl Event<Percent, MoveHitContext> for OnCalculateCritDamageMultiplierEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Percent, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_crit_damage_multiplier_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Percent, MoveHitContext, Nothing>> {
        event_listener.on_calculate_crit_damage_multiplier_handler()
    }
}

pub struct OnTryMoveHitEvent;

impl Event<Outcome, MoveHitContext> for OnTryMoveHitEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Outcome, MoveHitContext, MonsterID>> {
        event_listener.on_try_move_hit_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Outcome, MoveHitContext, Nothing>> {
        event_listener.on_try_move_hit_handler()
    }
}

pub struct OnMoveHitEvent;

impl Event<Nothing, MoveHitContext> for OnMoveHitEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, MoveHitContext, MonsterID>> {
        event_listener.on_move_hit_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, MoveHitContext, Nothing>> {
        event_listener.on_move_hit_handler()
    }
}

pub struct OnCalculateAttackStatEvent;

impl Event<u16, MoveHitContext> for OnCalculateAttackStatEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<u16, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_attack_stat_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<u16, MoveHitContext, Nothing>> {
        event_listener.on_calculate_attack_stat_handler()
    }
}

pub struct OnCalculateAttackStageEvent;

impl Event<i8, MoveHitContext> for OnCalculateAttackStageEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<i8, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_attack_stage_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<i8, MoveHitContext, Nothing>> {
        event_listener.on_calculate_attack_stage_handler()
    }
}

pub struct OnCalculateDefenseStatEvent;

impl Event<u16, MoveHitContext> for OnCalculateDefenseStatEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<u16, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_defense_stat_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<u16, MoveHitContext, Nothing>> {
        event_listener.on_calculate_defense_stat_handler()
    }
}

pub struct OnCalculateDefenseStageEvent;

impl Event<i8, MoveHitContext> for OnCalculateDefenseStageEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<i8, MoveHitContext, MonsterID>> {
        event_listener.on_calculate_defense_stage_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<i8, MoveHitContext, Nothing>> {
        event_listener.on_calculate_defense_stage_handler()
    }
}

pub struct OnModifyDamageEvent;

impl Event<u16, MoveHitContext> for OnModifyDamageEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<u16, MoveHitContext, MonsterID>> {
        event_listener.on_modify_damage_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<u16, MoveHitContext, Nothing>> {
        event_listener.on_modify_damage_handler()
    }
}

pub struct OnDamageDealtEvent;

impl Event<Nothing, Nothing> for OnDamageDealtEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, Nothing, MonsterID>> {
        event_listener.on_damage_dealt_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<Nothing, Nothing, Nothing>> {
        event_listener.on_damage_dealt_handler()
    }
}

pub struct OnTryActivateAbilityEvent;

impl Event<Outcome, AbilityActivationContext> for OnTryActivateAbilityEvent {
    fn get_event_handler_with_receiver(
        &self,
        event_listener: &'static dyn EventListener,
    ) -> Option<EventHandler<Outcome, AbilityActivationContext, MonsterID>> {
        event_listener.on_try_activate_ability_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Outcome, AbilityActivationContext, Nothing>> {
        event_listener.on_try_activate_ability_handler()
    }
}

pub struct OnAbilityActivatedEvent;

impl Event<Nothing, AbilityActivationContext> for OnAbilityActivatedEvent {
    fn get_event_handler_with_receiver(
        &self,
        event_listener: &'static dyn EventListener,
    ) -> Option<EventHandler<Nothing, AbilityActivationContext, MonsterID>> {
        event_listener.on_ability_activated_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, AbilityActivationContext, Nothing>> {
        event_listener.on_ability_activated_handler()
    }
}

pub struct OnTryStatChangeEvent;

impl Event<Outcome, StatChangeContext> for OnTryStatChangeEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Outcome, StatChangeContext, MonsterID>> {
        event_listener.on_try_stat_change_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Outcome, StatChangeContext, Nothing>> {
        event_listener.on_try_stat_change_handler()
    }
}

pub struct OnModifyStatChangeEvent;

impl Event<i8, StatChangeContext> for OnModifyStatChangeEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<i8, StatChangeContext, MonsterID>> {
        event_listener.on_modify_stat_change_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<i8, StatChangeContext, Nothing>> {
        event_listener.on_modify_stat_change_handler()
    }
}

pub struct OnStatChangedEvent;

impl Event<Nothing, StatChangeContext> for OnStatChangedEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, StatChangeContext, MonsterID>> {
        event_listener.on_stat_changed_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, StatChangeContext, Nothing>> {
        event_listener.on_stat_changed_handler()
    }
}

pub struct OnTryInflictVolatileStatusEvent;

impl Event<Outcome, Nothing> for OnTryInflictVolatileStatusEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Outcome, Nothing, MonsterID>> {
        event_listener.on_try_inflict_volatile_status_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<Outcome, Nothing, Nothing>> {
        event_listener.on_try_inflict_volatile_status_handler()
    }
}

pub struct OnVolatileStatusInflictedEvent;

impl Event<Nothing, Nothing> for OnVolatileStatusInflictedEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, Nothing, MonsterID>> {
        event_listener.on_volatile_status_inflicted_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<Nothing, Nothing, Nothing>> {
        event_listener.on_volatile_status_inflicted_handler()
    }
}

pub struct OnTryInflictPersistentStatusEvent;

impl Event<Outcome, Nothing> for OnTryInflictPersistentStatusEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Outcome, Nothing, MonsterID>> {
        event_listener.on_try_inflict_persistent_status_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<Outcome, Nothing, Nothing>> {
        event_listener.on_try_inflict_persistent_status_handler()
    }
}

pub struct OnPersistentStatusInflictedEvent;

impl Event<Nothing, Nothing> for OnPersistentStatusInflictedEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, Nothing, MonsterID>> {
        event_listener.on_persistent_status_inflicted_handler()
    }

    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<Nothing, Nothing, Nothing>> {
        event_listener.on_persistent_status_inflicted_handler()
    }
}

pub struct OnTryUseHeldItemEvent;

impl Event<Outcome, ItemUseContext> for OnTryUseHeldItemEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Outcome, ItemUseContext, MonsterID>> {
        event_listener.on_try_use_held_item_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Outcome, ItemUseContext, Nothing>> {
        event_listener.on_try_use_held_item_handler()
    }
}

pub struct OnHeldItemUsedEvent;

impl Event<Nothing, ItemUseContext> for OnHeldItemUsedEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, ItemUseContext, MonsterID>> {
        event_listener.on_held_item_used_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, ItemUseContext, Nothing>> {
        event_listener.on_held_item_used_handler()
    }
}

pub struct OnTurnEndEvent;

impl Event<Nothing, Nothing, Nothing> for OnTurnEndEvent {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<Nothing, Nothing, MonsterID, Nothing>> {
        event_listener.on_turn_end_handler()
    }

    fn get_event_handler_without_receiver(
        &self,
        event_listener: &'static dyn EventListener<Nothing>,
    ) -> Option<EventHandler<Nothing, Nothing, Nothing, Nothing>> {
        event_listener.on_turn_end_handler()
    }
}
