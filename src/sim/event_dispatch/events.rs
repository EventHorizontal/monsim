use super::*;
use crate::sim::{Effect, ActivationOrder};
pub use event_dex::*;

/// Stores an `Effect` that gets simulated in response to an `Event` being triggered.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EventHandler<R: Copy, C: Copy> {
    pub effect: Effect<R, C>,
    #[cfg(feature = "debug")]
    pub source_code_location: &'static str,
}

impl<R: Copy, C: Copy> Debug for EventHandler<R,C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "debug")]
        let out = {
            f.debug_struct("EventHandler")
                .field("source_code_location", &self.source_code_location)
                .finish()
        };
        #[cfg(not(feature = "debug"))]
        let out = {
            write!(f, "EventHandler debug information only available with feature flag \"debug\" turned on.")
        };
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedEventHandler<R: Copy, C: Copy> {
    pub event_handler: EventHandler<R, C>,
    pub owner_id: MonsterID,
    pub activation_order: ActivationOrder,
    pub filtering_options: EventFilteringOptions,
}

impl EventHandlerDeck {
    pub const fn empty() -> Self {
        DEFAULT_EVENT_HANDLERS
    }
}

pub mod contexts {
    use monsim_utils::MaxSizedVec;

    use crate::{sim::{MonsterID, MoveID}, AbilityID};

    /// `move_user_id`: MonsterID of the Monster using the move.
    /// 
    /// `move_used_id`: MoveID of the Move being used.
    /// 
    /// `target_ids`: MonsterIDs of the Monsters the move is being used on.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveUseContext {
        pub move_user_id: MonsterID,
        pub move_used_id: MoveID,
        pub target_ids: MaxSizedVec<MonsterID, 6>,
    }

    impl MoveUseContext {
        pub fn new(move_used_id: MoveID, target_ids: MaxSizedVec<MonsterID, 6>) -> Self {
            Self {
                move_user_id: move_used_id.owner_id,
                move_used_id,
                target_ids,
            }
        }
    }

    /// `move_user_id`: MonsterID of the Monster hitting.
    /// 
    /// `move_used_id`: MoveID of the Move being used.
    /// 
    /// `target_id`: MonsterID of the Monster being hit.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveHitContext {
        pub move_user_id: MonsterID,
        pub move_used_id: MoveID,
        pub target_id: MonsterID,
    }

    impl MoveHitContext {
        pub fn new(move_used_id: MoveID, target_id: MonsterID) -> Self {
            Self {
                move_user_id: move_used_id.owner_id,
                move_used_id,
                target_id,
            }
        }
    }

    /// `ability_owner_id`: MonsterID of the Monster whose ability is being used.
    /// 
    /// `ability_used_id`: AbilityID of the Ability being used.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AbilityUseContext {
        pub ability_owner_id: MonsterID,
        pub ability_used_id: AbilityID,
    }

    impl AbilityUseContext {
        pub fn new(ability_owner: MonsterID) -> Self {
            Self {
                ability_used_id: AbilityID { owner_id: ability_owner },
                ability_owner_id: ability_owner,
            }
        }
    }

    /// `active_monster_id`: MonsterID of the Monster to be switched out.
    /// 
    /// `benched_monster_id`: MonsterID of the Monster to be switched in.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SwitchContext {
        pub active_monster_id: MonsterID,
        pub benched_monster_id: MonsterID,
    }

    impl SwitchContext {
        pub fn new(active_monster_id: MonsterID, benched_monster_id: MonsterID) -> Self {
            Self {
                active_monster_id,
                benched_monster_id,
            }
        }
    }
}

// Generated.
#[cfg(feature="event_gen")]
mod event_dex {
    use super::*;
    use monsim_macros::generate_events;
    use monsim_utils::NOTHING;
    
    generate_events!{
        event OnTryMove(MoveUseContext) => Outcome,
        event OnMoveUsed(MoveUseContext) => Nothing,
        event OnDamagingMoveUsed(MoveUseContext) => Nothing,
        event OnTryMoveHit(MoveHitContext) => Outcome,
        event OnMoveHit(MoveHitContext) => Nothing,
        event OnDamageDealt(Nothing) => Nothing,
        event OnTryActivateAbility(AbilityUseContext) => Outcome,
        event OnAbilityActivated(AbilityUseContext) => Nothing,
        event OnModifyAccuracy(MoveUseContext) => Percent,
        event OnTryRaiseStat(Nothing) => Outcome,
        event OnTryLowerStat(Nothing) => Outcome,
        event OnStatusMoveUsed(MoveUseContext) => Nothing,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EventID {
        OnTryMove,
        OnMoveUsed,
        OnDamagingMoveUsed,
        OnStatusMoveUsed,
        OnTryMoveHit,
        OnMoveHit,
        OnDamageDealt,
        OnTryActivateAbility,
        OnAbilityActivated,
        OnModifyAccuracy,
        OnTryRaiseStat,
        OnTryLowerStat,
    }

    pub(crate) fn trigger_try_move_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: MoveUseContext,
    ) -> Outcome {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_try_move
            },
            event_context
        )
    }

    pub(crate) fn trigger_move_used_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: MoveUseContext,
    ) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_move_used
            },
            event_context,
            NOTHING,
            None
        )
    }

    pub(crate) fn trigger_damaging_move_used_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: MoveUseContext,
    ) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_damaging_move_used
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_status_move_used_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: MoveUseContext,
    ) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_status_move_used
            },
            event_context,
            NOTHING,
            None
        )
    }

    pub(crate) fn trigger_try_move_hit_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: MoveHitContext,
    ) -> Outcome {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_try_move_hit
            },
            event_context
        )
    }

    pub(crate) fn trigger_move_hit_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: MoveHitContext,
    ) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_move_hit
            },
            event_context,
            NOTHING,
            None
        )
    }

    pub(crate) fn trigger_damage_dealt_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: Nothing,
    ) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_damage_dealt
            },
            NOTHING,
            NOTHING,
            None
        )
    }

    pub(crate) fn trigger_try_activate_ability_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: AbilityUseContext,
    ) -> Outcome {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_try_activate_ability
            },
            event_context
        )
    }

    pub(crate) fn trigger_ability_activated_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: AbilityUseContext,
    ) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_ability_activated
            },
            event_context,
            NOTHING,
            None
        )
    }

    pub(crate) fn trigger_modify_accuracy_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: MoveUseContext,
    ) -> Percent {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_modify_accuracy
            },
            event_context,
            Percent(100),
            None
        )
    }

    pub(crate) fn trigger_try_raise_stat_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: Nothing,
    ) -> Outcome {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_try_raise_stat
            },
            event_context
        )
    }
    
    pub(crate) fn trigger_try_lower_stat_event(
        sim: &mut BattleSimulator,
    
        broadcaster_id: MonsterID,
        event_context: Nothing,
    ) -> Outcome {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id, 
            |event_handler_deck| {
                event_handler_deck.on_try_lower_stat
            },
            event_context
        )
    }
}

// This module is mostly to improve build times when working on the engine in a way that doesn't
// touch the event generation.
#[cfg(not(feature="event_gen"))] 
mod generated {
    use super::*;
    use event_dex::*;
    #[derive(Debug, Clone, Copy)]
    pub struct EventHandlerDeck {
        pub on_try_move: Option<EventHandler<OnTryMove>>,
        pub on_move_used: Option<EventHandler<OnMoveUsed>>,
        pub on_try_move_hit: Option<EventHandler<OnTryMoveHit>>,
        pub on_hit: Option<EventHandler<OnHit>>,
        pub on_damage_dealt: Option<EventHandler<OnDamageDealt>>,
        pub on_try_activate_ability: Option<EventHandler<OnTryActivateAbility>>,
        pub on_ability_activated: Option<EventHandler<OnAbilityActivated>>,
        pub on_modify_accuracy: Option<EventHandler<OnModifyAccuracy>>,
        pub on_try_raise_stat: Option<EventHandler<OnTryRaiseStat>>,
        pub on_try_lower_stat: Option<EventHandler<OnTryLowerStat>>,
        pub on_status_move_used: Option<EventHandler<OnStatusMoveUsed>>,
    }
    pub(super) const DEFAULT_EVENT_HANDLERS: EventHandlerDeck = EventHandlerDeck {
        on_try_move: None,
        on_move_used: None,
        on_try_move_hit: None,
        on_hit: None,
        on_damage_dealt: None,
        on_try_activate_ability: None,
        on_ability_activated: None,
        on_modify_accuracy: None,
        on_try_raise_stat: None,
        on_try_lower_stat: None,
        on_status_move_used: None,
    };
    pub mod event_dex {
        use super::*;
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnTryMove;

        impl Event for OnTryMove {
            type EventReturnType = Outcome;
            type ContextType = MoveUseContext;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_try_move
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_try_move
            }
            fn name(&self) -> &'static str {
                "OnTryMove"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnMoveUsed;

        impl Event for OnMoveUsed {
            type EventReturnType = Nothing;
            type ContextType = MoveUseContext;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_move_used
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_move_used
            }
            fn name(&self) -> &'static str {
                "OnMoveUsed"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnTryMoveHit;

        impl Event for OnTryMoveHit {
            type EventReturnType = Outcome;
            type ContextType = MoveHitContext;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_try_move_hit
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_try_move_hit
            }
            fn name(&self) -> &'static str {
                "OnTryMoveHit"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnHit;

        impl Event for OnHit {
            type EventReturnType = Nothing;
            type ContextType = Nothing;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_hit
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_hit
            }
            fn name(&self) -> &'static str {
                "OnHit"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnDamageDealt;

        impl Event for OnDamageDealt {
            type EventReturnType = Nothing;
            type ContextType = Nothing;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_damage_dealt
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_damage_dealt
            }
            fn name(&self) -> &'static str {
                "OnDamageDealt"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnTryActivateAbility;

        impl Event for OnTryActivateAbility {
            type EventReturnType = Outcome;
            type ContextType = AbilityUseContext;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_try_activate_ability
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_try_activate_ability
            }
            fn name(&self) -> &'static str {
                "OnTryActivateAbility"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnAbilityActivated;

        impl Event for OnAbilityActivated {
            type EventReturnType = Nothing;
            type ContextType = AbilityUseContext;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_ability_activated
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_ability_activated
            }
            fn name(&self) -> &'static str {
                "OnAbilityActivated"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnModifyAccuracy;

        impl Event for OnModifyAccuracy {
            type EventReturnType = Percent;
            type ContextType = MoveUseContext;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_modify_accuracy
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_modify_accuracy
            }
            fn name(&self) -> &'static str {
                "OnModifyAccuracy"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnTryRaiseStat;

        impl Event for OnTryRaiseStat {
            type EventReturnType = Outcome;
            type ContextType = Nothing;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_try_raise_stat
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_try_raise_stat
            }
            fn name(&self) -> &'static str {
                "OnTryRaiseStat"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnTryLowerStat;

        impl Event for OnTryLowerStat {
            type EventReturnType = Outcome;
            type ContextType = Nothing;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_try_lower_stat
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_try_lower_stat
            }
            fn name(&self) -> &'static str {
                "OnTryLowerStat"
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct OnStatusMoveUsed;

        impl Event for OnStatusMoveUsed {
            type EventReturnType = Nothing;
            type ContextType = MoveUseContext;
            fn corresponding_handler(&self, event_handler_deck: EventHandlerDeck) -> Option<EventHandler<Self>> {
                event_handler_deck.on_status_move_used
            }
            fn corresponding_handler_mut<'a>(&self, event_handler_deck: &'a mut EventHandlerDeck) -> &'a mut Option<EventHandler<Self>> {
                &mut event_handler_deck.on_status_move_used
            }
            fn name(&self) -> &'static str {
                "OnStatusMoveUsed"
            }
        }
    }
}
