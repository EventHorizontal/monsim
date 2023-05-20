use core::fmt::Debug;

use crate::prng::Prng;

use super::{game_mechanics::BattlerUID, Battle, BattleContext};

#[allow(non_camel_case_types)]
type void = ();

#[cfg(not(feature = "debug"))]
#[derive(Clone, Copy)]
pub struct EventHandler<R: Clone + Copy> {
    pub callback: fn(&mut BattleContext, &mut Prng, BattlerUID, R) -> EventReturn<R>,
}

#[cfg(not(feature = "debug"))]
impl<'a, R: Clone + Copy> Debug for EventHandler<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandler")
            .field(
                "callback",
                &&(self.callback as ExplicitlyAnnotatedEventHandler<'a, R>),
            )
            .finish()
    }
}

#[cfg(feature = "debug")]
#[derive(Clone, Copy)]
pub struct EventHandler<R: Clone + Copy> {
    pub callback: fn(&mut BattleContext, &mut Prng, BattlerUID, R) -> EventReturn<R>,
    #[cfg(feature = "debug")]
    pub dbg_location: &'static str,
}

#[cfg(feature = "debug")]
impl<'a, R: Clone + Copy> Debug for EventHandler<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandler")
            .field(
                "callback",
                &&(self.callback as ExplicitlyAnnotatedEventHandler<'a, R>),
            )
            .field("location", &self.dbg_location)
            .finish()
    }
}

pub type ExplicitlyAnnotatedEventHandler<'a, R> =
    fn(&'a mut BattleContext, &'a mut Prng, BattlerUID, R) -> EventReturn<R>;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerSetInstance {
    pub event_handler_set: EventHandlerSet,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters,
}
pub type EventHandlerSetInstanceList = Vec<EventHandlerSetInstance>;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerInstance<R: Clone + Copy> {
    pub event_name: &'static str,
    pub event_handler: EventHandler<R>,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters,
}

// impl<R: Debug + Clone + Copy> Debug for EventHandlerInstance<R> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("EventHandlerInstance")
//             .field("event_name", &self.event_name)
//             .field("event_handler", &std::any::type_name::<EventHandler<R>>())
//             .field("owner_uid", &self.owner_uid)
//             .field("activation_order", &self.activation_order)
//             .field("filters", &self.filters)
//             .finish()
//     }
// }

pub type EventHandlerInstanceList<R> = Vec<EventHandlerInstance<R>>;
pub type EventReturn<R> = R;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerSet {
    pub on_try_move: Option<EventHandler<bool>>,
    pub on_damage_dealt: Option<EventHandler<void>>,
    pub on_try_activate_ability: Option<EventHandler<bool>>,
    pub on_ability_activated: Option<EventHandler<void>>,
    pub on_modify_accuracy: Option<EventHandler<u16>>,
    pub on_try_raise_stat: Option<EventHandler<bool>>,
    pub on_try_lower_stat: Option<EventHandler<bool>>,
    pub on_status_move_used: Option<EventHandler<void>>,
}

// impl Debug for EventHandlerSet {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("EventHandlerSet")
//             .field("on_try_move", {
//                 &self
//                     .on_try_move
//                     .map(|_it| std::any::type_name::<EventHandler<bool>>())
//             })
//             .field("on_damage_dealt", {
//                 &self
//                     .on_damage_dealt
//                     .map(|_it| std::any::type_name::<EventHandler<void>>())
//             })
//             .field("on_try_activate_ability", {
//                 &self
//                     .on_try_activate_ability
//                     .map(|_it| std::any::type_name::<EventHandler<bool>>())
//             })
//             .field("on_ability_activated", {
//                 &self
//                     .on_ability_activated
//                     .map(|_it| std::any::type_name::<EventHandler<void>>())
//             })
//             .field("on_modify_accuracy", {
//                 &self
//                     .on_modify_accuracy
//                     .map(|_it| std::any::type_name::<EventHandler<u16>>())
//             })
//             .field("on_try_raise_stat", {
//                 &self
//                     .on_modify_accuracy
//                     .map(|_it| std::any::type_name::<EventHandler<bool>>())
//             })
//             .field("on_try_lower_stat", {
//                 &self
//                     .on_modify_accuracy
//                     .map(|_it| std::any::type_name::<EventHandler<bool>>())
//             })
//             .finish()
//     }
// }

pub const DEFAULT_HANDLERS: EventHandlerSet = EventHandlerSet {
    on_try_move: None,
    on_damage_dealt: None,
    on_try_activate_ability: None,
    on_ability_activated: None,
    on_modify_accuracy: None,
    on_try_raise_stat: None,
    on_try_lower_stat: None,
    on_status_move_used: None,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResolver;

impl EventResolver {
    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn broadcast_event<R: PartialEq + Copy>(
        ctx: &mut BattleContext,
        prng: &mut Prng,
        caller_uid: BattlerUID,
        event: &dyn InBattleEvent<EventReturnType = R>,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        let event_handlers_set_instances = ctx.event_handler_set_instances();
        let mut event_handler_instances = event_handlers_set_instances
            .iter()
            .filter_map(|event_handler_set_instance| {
                event
                    .corresponding_handler(&event_handler_set_instance.event_handler_set)
                    .map(|event_handler| EventHandlerInstance {
                        event_name: event.name(),
                        event_handler,
                        owner_uid: event_handler_set_instance.owner_uid,
                        activation_order: event_handler_set_instance.activation_order,
                        filters: EventHandlerFilters::default(),
                    })
            })
            .collect::<Vec<_>>();

        Battle::priority_sort::<EventHandlerInstance<R>>(
            prng,
            &mut event_handler_instances,
            &mut |it| it.activation_order,
        );

        if event_handler_instances.is_empty() {
            return default;
        }

        let mut relay = default;
        for EventHandlerInstance {
            event_name: _,
            event_handler,
            owner_uid,
            activation_order: _,
            filters,
        } in event_handler_instances.into_iter()
        {
            if ctx.filter_event_handlers(caller_uid, owner_uid, filters) {
                relay = (event_handler.callback)(ctx, prng, owner_uid, relay);
                // Return early if the relay becomes the short-circuiting value.
                if let Some(value) = short_circuit {
                    if relay == value {
                        return relay;
                    }
                };
            }
        }
        relay
    }

    pub fn broadcast_try_event(
        ctx: &mut BattleContext,
        prng: &mut Prng,
        caller_uid: BattlerUID,
        event: &dyn InBattleEvent<EventReturnType = bool>,
    ) -> bool {
        Self::broadcast_event(ctx, prng, caller_uid, event, true, Some(false))
    }
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

pub trait InBattleEvent {
    type EventReturnType: Sized + Clone + Copy;

    fn corresponding_handler(
        &self,
        event_handler_set: &EventHandlerSet,
    ) -> Option<EventHandler<Self::EventReturnType>>;

    fn name(&self) -> &'static str;
}

pub mod event_dex {
    use event_derive_macro::InBattleEvent;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(bool)]
    #[callback(on_try_move)]
    pub struct OnTryMove;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(void)]
    #[callback(on_ability_activated)]
    pub struct OnAbilityActivated;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(void)]
    #[callback(on_damage_dealt)]
    pub struct OnDamageDealt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(bool)]
    #[callback(on_try_activate_ability)]
    pub struct OnTryActivateAbility;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(u16)]
    #[callback(on_modify_accuracy)]
    pub struct OnModifyAccuracy;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(bool)]
    #[callback(on_try_raise_stat)]
    pub struct OnTryRaiseStat;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(bool)]
    #[callback(on_try_lower_stat)]
    pub struct OnTryLowerStat;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(void)]
    #[callback(on_status_move_used)]
    pub struct OnStatusMoveUsed;
}
