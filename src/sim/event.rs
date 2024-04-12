use core::fmt::Debug;

use crate::{message_log::MessageLog, prng::Prng, sim::{game_mechanics::MonsterUID, generate_events, ordering::sort_by_activation_order, Nothing, Outcome, Percent}, BattleEntities};

use contexts::*;
use event_dex::*;

generate_events!{
    event OnTryMove(TheMoveUsed) => Outcome,
    event OnDamageDealt(Nothing) => Nothing,
    event OnTryActivateAbility(TheAbilityActivated) => Outcome,
    event OnAbilityActivated(TheAbilityActivated) => Nothing,
    event OnModifyAccuracy(TheMoveUsed) => Percent,
    event OnTryRaiseStat(Nothing) => Outcome,
    event OnTryLowerStat(Nothing) => Outcome,
    event OnStatusMoveUsed(TheMoveUsed) => Nothing,
}

pub trait Event: Clone + Copy {
    
    type EventResult: Sized + Clone + Copy;
    type Context: Sized + Clone + Copy;

    fn corresponding_handlers<'a>(&self, event_handler_storage: &'a EventHandlerStorage) -> &'a Vec<OwnedEventHandler<Self::EventResult, Self::Context>>;
    fn corresponding_handlers_mut<'a>(&self, event_handler_storage: &'a mut EventHandlerStorage) -> &'a mut Vec<OwnedEventHandler<Self::EventResult, Self::Context>>;

    fn uid(&self) -> EventID;
}

pub trait EventResponse<R: Clone + Copy, C: Clone + Copy> {
 fn get(&self) -> &EventCallback<R, C>;
}

impl<R: Clone + Copy, C: Sized + Clone + Copy> EventResponse<R, C> for EventCallback<R, C> {
    fn get(&self) -> &EventCallback<R, C> {
        self
    }
}  

#[derive(Debug, Clone)]
pub struct EventDispatcher {
    pub(crate) event_handler_storage: EventHandlerStorage,
}

pub type BattleAPI<'a> = (&'a mut BattleEntities, &'a mut MessageLog);

type EventCallback<R, C> = fn(BattleAPI, C, R) -> R;
#[cfg(feature = "debug")]
type EventCallbackWithLifetime<'a, R, C> = fn((&'a mut BattleEntities, &'a mut MessageLog), C, R) -> R;

/// `R`: indicates return type
///
/// `C`: indicates context specifier type
#[derive(Clone, Copy)]
pub struct EventHandler<R: Copy, C: Copy> {
    pub callback: EventCallback<R, C>,
    #[cfg(feature = "debug")]
    pub debugging_information: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct OwnedEventHandler<R: Copy, C: Copy> {
    pub for_event: EventID,
    pub event_handler: EventHandler<R, C>,
    pub owner: OwnerInfo,
    pub filtering_options: EventFilteringOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFilteringOptions {
    pub event_source: TargetFlags,
    pub requires_being_active: bool,
}

bitflags::bitflags! {
    pub struct TargetFlags: u8 {
        const SELF = 0b0001;
        const ALLIES = 0b0010;
        const OPPONENTS = 0b0100;
        const ENVIRONMENT = 0b1000;
    }
}

pub mod contexts {
    use crate::{sim::{MonsterUID, MoveUID}, AbilityUID};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TheMoveUsed {
        // TODO: Make these private?
        pub move_user: MonsterUID,
        pub move_used: MoveUID,
        pub target: MonsterUID,
    }

    impl TheMoveUsed {
        pub fn new(move_used: MoveUID, target: MonsterUID) -> Self {
            Self {
                move_user: move_used.owner_uid,
                move_used,
                target,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TheAbilityActivated {
        pub activated_ability: AbilityUID,
    }

    impl TheAbilityActivated {
        pub fn new(ability_owner: MonsterUID) -> Self {
            Self {
                activated_ability: AbilityUID { owner: ability_owner },
            }
        }
    }
}

impl EventHandlerStorage {
    #[cfg(not(feature="debug"))]
    pub fn add<R: Copy, C: Copy>(&mut self, owner: OwnerInfo, event: impl Event<EventResult = R, Context = C>, callback: EventCallback<R, C>) {
        let owned_event_handler = OwnedEventHandler {
            event_name: format!["{:?}", event.uid()],
            event_handler: EventHandler {
                callback,
            },
            owner,
            filtering_options: EventFilteringOptions::default(),
        };
        self[event.uid()].push(owned_event_handler); 
    }

    #[cfg(feature="debug")]
    pub fn add<R: Copy, C: Copy>(&mut self, owner: OwnerInfo, event: impl Event<EventResult = R, Context = C>, callback: EventCallback<R, C>, debugging_information: &'static str) {
        let owned_event_handler = OwnedEventHandler {
            for_event: event.uid(),
            event_handler: EventHandler {
                callback,
                debugging_information,
            },
            owner,
            filtering_options: EventFilteringOptions::default(),
        };
        event.corresponding_handlers_mut(self).push(owned_event_handler); 
    }
}

/// Stores all the information related to the Monster that owns whatever p
#[derive(Debug, Clone, Copy)]
pub struct OwnerInfo {
    pub uid: MonsterUID,
    pub activation_order: ActivationOrder,
}

// TODO: Move to ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationOrder {
    pub priority: i8,
    pub speed: u16,
    pub order: u16,
}

impl EventDispatcher {

    /// Convenience wrapper for `dispatch_event` for specifially trial events.
    pub fn dispatch_try_event<C: Copy>(
        &mut self,
        prng: &mut Prng,
        api: (&mut BattleEntities, &mut MessageLog),
        event: impl Event<EventResult = Outcome, Context = C>,
        broadcaster: MonsterUID,
        context: C,
    ) -> Outcome {
        self.dispatch_event(prng, api, event, broadcaster, context, Outcome::Success, Some(Outcome::Failure))
    }

    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn dispatch_event<R: PartialEq + Copy, C: Copy>(
        &mut self,
        prng: &mut Prng,
        api: (&mut BattleEntities, &mut MessageLog),
        event: impl Event<EventResult = R, Context = C>,
        broadcaster: MonsterUID,
        context: C,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        // We would like to sort the owned event handlers, but then we want to drop the mutable reference
        {
            let owned_event_handlers = event.corresponding_handlers_mut(&mut self.event_handler_storage);
            
            // We also want to check if it's empty before we commit to sorting it.
            if owned_event_handlers.is_empty() {
                return default;
            }
     
            sort_by_activation_order::<OwnedEventHandler<R, C>>(prng, owned_event_handlers, &mut |it| {
                it.owner.activation_order
            });   
        }
        let owned_event_handlers = event.corresponding_handlers(&self.event_handler_storage);

        let mut relay = default;
        for OwnedEventHandler {
            event_handler,
            owner,
            filtering_options,
            ..
        } in owned_event_handlers.into_iter()
        {
            if Self::filter_event_handlers(api.0, broadcaster, owner.uid, *filtering_options) {
                relay = (event_handler.callback)(api, context, relay);
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

    fn filter_event_handlers(
        battle_entities: &BattleEntities,
        broadcaster_uid: MonsterUID,
        owner_uid: MonsterUID,
        filter_options: EventFilteringOptions,
    ) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if broadcaster_uid == owner_uid {
                bitmask |= TargetFlags::SELF.bits()
            } // 0x01
            if battle_entities.are_allies(owner_uid, broadcaster_uid) {
                bitmask |= TargetFlags::ALLIES.bits()
            } // 0x02
            if battle_entities.are_opponents(owner_uid, broadcaster_uid) {
                bitmask |= TargetFlags::OPPONENTS.bits()
            } //0x04
              // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = filter_options.event_source.bits() == bitmask;
        let is_active_passed = battle_entities.is_active_monster(owner_uid);

        event_source_filter_passed && is_active_passed
    }
}

impl<'a, R: Copy, C: Copy> Debug for EventHandler<R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "debug")]
        let out = {
            f.debug_struct("EventHandler")
                .field("callback", &&(self.callback as EventCallbackWithLifetime<'a, R, C>))
                .field("location", &self.debugging_information)
                .finish()
        };
        #[cfg(not(feature = "debug"))]
        let out = {
            write!(f, "EventHandler debug information only available with feature flag \"debug\" turned on.")
        };
        out
    }
}

impl EventFilteringOptions {
    pub const fn default() -> EventFilteringOptions {
        EventFilteringOptions {
            event_source: TargetFlags::OPPONENTS,
            requires_being_active: true,
        }
    }
}