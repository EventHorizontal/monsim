use super::*;

impl EventHandlerDeck {
    pub const fn const_default() -> Self {
        DEFAULT_DECK
    }
}

#[cfg(feature="macros")]
use monsim_macros::generate_events;
#[cfg(feature="macros")]
generate_events!{
    event OnTryMove(MoveUsed) => Outcome,
    event OnDamageDealt(Nothing) => Nothing,
    event OnTryActivateAbility(AbilityUsed) => Outcome,
    event OnAbilityActivated(AbilityUsed) => Nothing,
    event OnModifyAccuracy(MoveUsed) => Percent,
    event OnTryRaiseStat(Nothing) => Outcome,
    event OnTryLowerStat(Nothing) => Outcome,
    event OnStatusMoveUsed(MoveUsed) => Nothing,
}
#[cfg(not(feature="macros"))]
#[derive(Debug, Clone, Copy)]
pub struct EventHandlerDeck {
    pub on_try_move: Option<EventHandler<Outcome, MoveUsed>>,
    pub on_damage_dealt: Option<EventHandler<Nothing, Nothing>>,
    pub on_try_activate_ability: Option<EventHandler<Outcome, AbilityUseContext>>,
    pub on_ability_activated: Option<EventHandler<Nothing, AbilityUseContext>>,
    pub on_modify_accuracy: Option<EventHandler<Percent, MoveUsed>>,
    pub on_try_raise_stat: Option<EventHandler<Outcome, Nothing>>,
    pub on_try_lower_stat: Option<EventHandler<Outcome, Nothing>>,
    pub on_status_move_used: Option<EventHandler<Nothing, MoveUsed>>,
}
pub const DEFAULT_DECK: EventHandlerDeck = EventHandlerDeck {
    on_try_move: None,
    on_damage_dealt: None,
    on_try_activate_ability: None,
    on_ability_activated: None,
    on_modify_accuracy: None,
    on_try_raise_stat: None,
    on_try_lower_stat: None,
    on_status_move_used: None,
};
#[derive(Debug, Clone)]
pub struct EventHandlerStorage {
    pub on_try_move: Vec<OwnedEventHandler<Outcome, MoveUsed>>,
    pub on_damage_dealt: Vec<OwnedEventHandler<Nothing, Nothing>>,
    pub on_try_activate_ability: Vec<OwnedEventHandler<Outcome, AbilityUseContext>>,
    pub on_ability_activated: Vec<OwnedEventHandler<Nothing, AbilityUseContext>>,
    pub on_modify_accuracy: Vec<OwnedEventHandler<Percent, MoveUsed>>,
    pub on_try_raise_stat: Vec<OwnedEventHandler<Outcome, Nothing>>,
    pub on_try_lower_stat: Vec<OwnedEventHandler<Outcome, Nothing>>,
    pub on_status_move_used: Vec<OwnedEventHandler<Nothing, MoveUsed>>,
}
impl EventHandlerStorage {
    pub const fn new() -> Self {
        Self {
            on_try_move: vec![],
            on_damage_dealt: vec![],
            on_try_activate_ability: vec![],
            on_ability_activated: vec![],
            on_modify_accuracy: vec![],
            on_try_raise_stat: vec![],
            on_try_lower_stat: vec![],
            on_status_move_used: vec![],
        }
    }
}
pub mod event_dex {
    use super::*;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnTryMove;

    impl Event for OnTryMove {
        type EventReturnType = Outcome;
        type ContextType = MoveUsed;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
            event_handler_deck.on_try_move
        }
        fn name(&self) -> &'static str {
            "OnTryMove"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnDamageDealt;

    impl Event for OnDamageDealt {
        type EventReturnType = Nothing;
        type ContextType = Nothing;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
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
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
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
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
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
        type ContextType = MoveUsed;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
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
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
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
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
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
        type ContextType = MoveUsed;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>> {
            event_handler_deck.on_status_move_used
        }
        fn name(&self) -> &'static str {
            "OnStatusMoveUsed"
        }
    }
}

pub trait Event: Clone + Copy {
    type EventReturnType: Sized + Clone + Copy;
    type ContextType: Sized + Clone + Copy;

    fn corresponding_handler(
        &self,
        event_handler_deck: &EventHandlerDeck,
    ) -> Option<EventHandler<Self::EventReturnType, Self::ContextType>>;

    fn name(&self) -> &'static str;
}