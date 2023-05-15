use core::fmt::Debug;

use crate::prng::Lcrng;

use super::{game_mechanics::BattlerUID, global_constants::void, Battle, BattleContext};

pub type EventHandler<R> = fn(&mut BattleContext, &mut Lcrng, BattlerUID, R) -> EventReturn<R>;
pub type ExplicitlyAnnotatedEventHandler<'a, R> =
    fn(&'a mut BattleContext, &'a mut Lcrng, BattlerUID, R) -> EventReturn<R>;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerSetInfo {
    pub event_handler_set: EventHandlerSet,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters,
}
pub type EventHandlerSetInfoList = Vec<EventHandlerSetInfo>;

#[derive(Clone, Copy)]
pub struct EventHandlerInfo<R: Clone + Copy> {
    pub event_handler: EventHandler<R>,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters,
}
pub type EventHandlerInfoList<R> = Vec<EventHandlerInfo<R>>;
pub type EventReturn<R> = R;

#[derive(Clone, Copy)]
pub struct EventHandlerSet {
    pub on_try_move: Option<EventHandler<bool>>,
    pub on_damage_dealt: Option<EventHandler<void>>,
    pub on_try_activate_ability: Option<EventHandler<bool>>,
    pub on_ability_activated: Option<EventHandler<void>>,
    pub on_modify_accuracy: Option<EventHandler<u16>>,
}

impl Debug for EventHandlerSet {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandlerSet")
            .field(
                "on_try_move",
                &(self.on_try_move as Option<ExplicitlyAnnotatedEventHandler<'a, bool>>)
                    as &dyn Debug,
            )
            .field(
                "on_damage_dealt",
                &(self.on_damage_dealt as Option<ExplicitlyAnnotatedEventHandler<'a, void>>)
                    as &dyn Debug,
            )
            .field(
                "on_try_activate_ability",
                &(self.on_try_activate_ability as Option<ExplicitlyAnnotatedEventHandler<'a, bool>>)
                    as &dyn Debug,
            )
            .field(
                "on_ability_activated",
                &(self.on_ability_activated as Option<ExplicitlyAnnotatedEventHandler<'a, void>>)
                    as &dyn Debug,
            )
            .field(
                "on_modify_accuracy",
                &(self.on_modify_accuracy as Option<ExplicitlyAnnotatedEventHandler<'a, u16>>)
                    as &dyn Debug,
            )
            .finish()
    }
}

pub const DEFAULT_HANDLERS: EventHandlerSet = EventHandlerSet {
    on_try_move: None,
    on_damage_dealt: None,
    on_try_activate_ability: None,
    on_ability_activated: None,
    on_modify_accuracy: None,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResolver;

impl EventResolver {
    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn broadcast_event<R: PartialEq + Copy>(
        context: &mut BattleContext,
        prng: &mut Lcrng,
        caller_uid: BattlerUID,
        event: &dyn InBattleEvent<EventReturnType = R>,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        let event_handler_set_plus_info = context.event_handler_sets_plus_info();
        let mut unwrapped_event_handler_plus_info = event_handler_set_plus_info
            .iter()
            .filter_map(|event_handler_set_info| {
                if let Some(handler) =
                    event.corresponding_handler(&event_handler_set_info.event_handler_set)
                {
                    Some(EventHandlerInfo {
                        event_handler: handler,
                        owner_uid: event_handler_set_info.owner_uid,
                        activation_order: event_handler_set_info.activation_order,
                        filters: EventHandlerFilters::default(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Battle::priority_sort::<EventHandlerInfo<R>>(
            prng,
            &mut unwrapped_event_handler_plus_info,
            &mut |it| it.activation_order,
        );

        if unwrapped_event_handler_plus_info.is_empty() {
            return default;
        }

        let mut relay = default;
        for EventHandlerInfo {
            event_handler,
            owner_uid,
            activation_order: _,
            filters,
        } in unwrapped_event_handler_plus_info.into_iter()
        {
            if context.filter_event_handlers(caller_uid, owner_uid, filters) {
                relay = event_handler(context, prng, owner_uid, relay);
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
        context: &mut BattleContext,
        prng: &mut Lcrng,
        caller_uid: BattlerUID,
        event: &dyn InBattleEvent<EventReturnType = bool>,
    ) -> bool {
        Self::broadcast_event(context, prng, caller_uid, event, true, Some(false))
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
    type EventReturnType: Sized;

    fn corresponding_handler(
        &self,
        event_handler_set: &EventHandlerSet,
    ) -> Option<EventHandler<Self::EventReturnType>>;
}

pub mod event_dex {
    use event_derive_macro::InBattleEvent;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(bool)]
    #[callback(on_try_move)]
    pub struct OnTryMove;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(())]
    #[callback(on_ability_activated)]
    pub struct OnAbilityActivated;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, InBattleEvent)]
    #[return_type(())]
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
}
