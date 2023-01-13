use core::fmt::Debug;

use super::{BattleContext, game_mechanics::BattlerUID};

pub type EventHandler<R> = fn(&mut BattleContext, BattlerUID, R) -> EventReturn<R>;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerSetInfo {
    pub event_handler_set: EventHandlerSet,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters
}
pub type EventHandlerSetInfoList = Vec<EventHandlerSetInfo>;

#[derive(Clone, Copy)]
pub struct EventHandlerInfo<R: Clone+Copy> {
    pub event_handler: EventHandler<R>,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters
}
pub type EventHandlerInfoList<R> = Vec<EventHandlerInfo<R>>;

pub type Void = ();
pub type EventReturn<R> = R;

#[derive(Clone, Copy)]
pub struct EventHandlerSet {
    pub on_try_move: Option<EventHandler<bool>>,
    pub on_damage_dealt: Option<EventHandler<Void>>,
    pub on_try_activate_ability: Option<EventHandler<bool>>,
    pub on_ability_activated: Option<EventHandler<Void>>,
    pub on_modify_accuracy: Option<EventHandler<u16>>,
}

#[test]
fn test_print_event_handler_set() {
    use crate::battle_sim::ability_dex::FlashFire;
    println!("{:?}", FlashFire.event_handlers);
}

impl Debug for EventHandlerSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandlerSet")
            .field("on_try_move", if self.on_try_move.is_some() { &Some("EventHandler") } else { &None::<()> } )
            .field("on_damage_dealt", if self.on_damage_dealt.is_some() { &Some("EventHandler") } else { &None::<()> } )
            .field("on_try_activate_ability", if self.on_try_activate_ability.is_some() { &Some("EventHandler") } else { &None::<()> } )
            .field("on_ability_activated", if self.on_ability_activated.is_some() { &Some("EventHandler") } else { &None::<()> } )
            .field("on_modify_accuracy", if self.on_modify_accuracy.is_some() { &Some("EventHandler") } else { &None::<()> } )
            .finish()
    }
}

pub const DEFAULT_HANDLERS: EventHandlerSet = EventHandlerSet { 
    on_try_move: None, 
    on_damage_dealt: None,
    on_try_activate_ability: None, 
    on_ability_activated: None, 
    on_modify_accuracy: None 
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResolver;

impl EventResolver {
    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    /// 
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn broadcast_event<R: PartialEq + Copy>(context: &mut BattleContext, caller_uid: BattlerUID, event: &dyn InBattleEvent<EventReturnType=R>, default: R, short_circuit: Option<R>) -> R {
        let handler_retriever = event.handler_retriever();
        
        let event_handler_set_plus_info = context.event_handler_sets_plus_info();
        let mut unwrapped_event_handler_plus_info = event_handler_set_plus_info
            .iter()
            .filter_map(|event_handler_set_info| {
                    if let Some (handler) = handler_retriever(&event_handler_set_info.event_handler_set) {
                        Some(EventHandlerInfo {
                            event_handler: handler,
                            owner_uid: event_handler_set_info.owner_uid,
                            activation_order: event_handler_set_info.activation_order,
                            filters: EventHandlerFilters::default(),
                        })
                    } else{
                        None
                    }
                }
            )
            .collect::<Vec<_>>();

        context.priority_sort::<R>(&mut unwrapped_event_handler_plus_info);
                    
        if unwrapped_event_handler_plus_info.is_empty() {
            return default;
        }

        let mut relay = default;    
        for EventHandlerInfo {
            event_handler,
            owner_uid,
            activation_order: _,
            filters,
        } in unwrapped_event_handler_plus_info.into_iter() {
            if context.filter_event_handlers(caller_uid, owner_uid, filters) {
                relay = event_handler(context, owner_uid, relay);
                // Return early if the relay becomes the short-circuiting value.
                if let Some(value) = short_circuit {
                    if relay == value { return relay; }    
                };
            }
        }
        relay
    }
    
    pub fn broadcast_try_event(context: & mut BattleContext, caller_uid: BattlerUID, event: &dyn InBattleEvent<EventReturnType = bool>) -> bool {
        Self::broadcast_event(context, caller_uid, event, true, Some(false))
    }
}

macro_rules! field {
    ($x:ident) => {
        |it| { it.$x }
    };
}

pub trait InBattleEvent {
    type EventReturnType: Sized;
    
    fn handler_retriever(&self) -> fn(&EventHandlerSet) -> Option<EventHandler<Self::EventReturnType>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationOrder {
    pub priority: u16,
    pub speed: u16,
    pub order: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventHandlerFilters {
    pub whose_event: TargetFlags,
    pub on_battlefield: bool,
}

impl EventHandlerFilters {
    pub(crate) const fn default() -> EventHandlerFilters {
        EventHandlerFilters {
            whose_event: TargetFlags::OPPONENTS,
            on_battlefield: true,
        }
    }
}

use bitflags::bitflags;
bitflags! {
    pub struct TargetFlags: u8 {
        const SELF = 0b0001;
        const ALLIES = 0b0010;
        const OPPONENTS = 0b0100;
        const ENVIRONMENT = 0b1000;
    }
}

pub mod event_dex {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnTryMove;
    
    impl InBattleEvent for OnTryMove {
        type EventReturnType = bool;
    
        fn handler_retriever(&self) -> fn(&EventHandlerSet) -> Option<EventHandler<Self::EventReturnType>> {
            field!(on_try_move)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnAbilityActivated;
    
    impl InBattleEvent for OnAbilityActivated {
        type EventReturnType = ();
    
        fn handler_retriever(&self) -> fn(&EventHandlerSet) -> Option<EventHandler<Self::EventReturnType>> {
            field!(on_ability_activated)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnDamageDealt;
    
    impl InBattleEvent for OnDamageDealt {
        type EventReturnType = ();
    
        fn handler_retriever(&self) -> fn(&EventHandlerSet) -> Option<EventHandler<Self::EventReturnType>> {
            field!(on_damage_dealt)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnTryActivateAbility;
    
    impl InBattleEvent for OnTryActivateAbility {
        type EventReturnType = bool;
    
        fn handler_retriever(&self) -> fn(&EventHandlerSet) -> Option<EventHandler<Self::EventReturnType>> {
            field!(on_try_activate_ability)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnModifyAccuracy;
    
    impl InBattleEvent for OnModifyAccuracy {
        type EventReturnType = u16;
    
        fn handler_retriever(&self) -> fn(&EventHandlerSet) -> Option<EventHandler<Self::EventReturnType>> {
            field!(on_modify_accuracy)
        }
    }
}
