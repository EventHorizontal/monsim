use super::*;
pub use generated::*;
use crate::{sim::Effect, ActivationOrder};

/// Stores an `Effect` that gets simulated in response to an `Event` being triggered.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EventHandler<E: Event> {
    pub event: E,
    pub effect: Effect<E::EventReturnType, E::ContextType>,
    #[cfg(feature = "debug")]
    pub source_code_location: &'static str,
}

impl<'a, E: Event + Debug> Debug for EventHandler<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "debug")]
        let out = {
            f.debug_struct("EventHandler")
                .field("event", &self.event)
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
pub struct OwnedEventHandler<E: Event> {
    pub event_handler: EventHandler<E>,
    pub owner: MonsterUID,
    pub activation_order: ActivationOrder,
    pub filtering_options: EventFilteringOptions,
}

pub trait Event: Clone + Copy + PartialEq + Eq {
    type EventReturnType: Sized + Clone + Copy + PartialEq + Eq;
    type ContextType: Sized + Clone + Copy + PartialEq + Eq;

    fn corresponding_handler(
        &self,
        event_handler_deck: EventHandlerDeck,
    ) -> Option<EventHandler<Self>>;

    fn name(&self) -> &'static str;
}

impl EventHandlerDeck {
    pub const fn empty() -> Self {
        DEFAULT_EVENT_HANDLERS
    }
}

pub mod contexts {
    use crate::{sim::{MonsterUID, MoveUID}, AbilityUID};

    /// Holds the information of who used the move (`move_user`), which move was used (`move_used`)
    /// and who the move was used on (`target`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveUseContext {
        pub move_user: MonsterUID,
        pub move_used: MoveUID,
        pub target: MonsterUID,
    }

    impl MoveUseContext {
        pub fn new(move_uid: MoveUID, target_uid: MonsterUID) -> Self {
            Self {
                move_user: move_uid.owner_uid,
                move_used: move_uid,
                target: target_uid,
            }
        }
    }

    /// Holds the information of which monster's ability was used (`ability_used`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AbilityUseContext {
        pub ability_owner: MonsterUID,
        pub ability_used: AbilityUID,
    }

    impl AbilityUseContext {
        pub fn new(ability_owner: MonsterUID) -> Self {
            Self {
                ability_used: AbilityUID { owner: ability_owner },
                ability_owner,
            }
        }
    }

    /// Holds the information of which monster is trying to switch out (`active_monster`) 
    /// and which monster is trying to switch in (`benched_monster`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SwitchContext {
        pub active_monster: MonsterUID,
        pub benched_monster: MonsterUID,
    }

    impl SwitchContext {
        pub fn new(active_monster: MonsterUID, benched_monster: MonsterUID) -> Self {
            Self {
                active_monster,
                benched_monster,
            }
        }
    }
}

// Generated.
#[cfg(feature="macros")]
mod generated {
    use super::*;
    use monsim_macros::generate_events;
    use event_dex::*;
    
    generate_events!{
        event OnTryMove(MoveUseContext) => Outcome,
        event OnMoveUsed(MoveUseContext) => Nothing,
        event OnDamageDealt(Nothing) => Nothing,
        event OnTryActivateAbility(AbilityUseContext) => Outcome,
        event OnAbilityActivated(AbilityUseContext) => Nothing,
        event OnModifyAccuracy(MoveUseContext) => Percent,
        event OnTryRaiseStat(Nothing) => Outcome,
        event OnTryLowerStat(Nothing) => Outcome,
        event OnStatusMoveUsed(MoveUseContext) => Nothing,
    }
}

#[cfg(not(feature="macros"))] 
mod generated {
    use super::*;
    use event_dex::*;
    #[derive(Debug, Clone, Copy)]
    pub struct EventHandlerDeck {
        pub on_try_move: Option<EventHandler<OnTryMove>>,
        pub on_move_used: Option<EventHandler<OnMoveUsed>>,
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
            fn name(&self) -> &'static str {
                "OnMoveUsed"
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
            fn name(&self) -> &'static str {
                "OnStatusMoveUsed"
            }
        }
    }
}
