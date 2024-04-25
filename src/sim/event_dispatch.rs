use core::fmt::Debug;

pub mod events;
#[cfg(all(test, feature = "debug"))]
mod tests ;

use crate::{sim::{game_mechanics::MonsterUID, ordering::sort_by_activation_order, BattleState, Nothing, Outcome, Percent}, BattleSimulator};
use contexts::*;
pub use events::*;

#[derive(Debug, Clone)]
pub struct EventDispatcher;

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

// TODO: Move to ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationOrder {
    pub priority: i8,
    pub speed: u16,
    pub order: u16,
}

impl EventDispatcher {

    pub fn dispatch_trial_event<C: Copy>(
        sim: &mut BattleSimulator,

        event: impl Event<EventReturnType = Outcome, ContextType = C>,
        broadcaster_uid: MonsterUID,
        event_context: C,
    ) -> Outcome {
        EventDispatcher::dispatch_event(
            sim, 
            event, 
            broadcaster_uid, 
            event_context, 
            Outcome::Success, 
            Some(Outcome::Failure)
        )
    }

    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn dispatch_event<R: PartialEq + Copy, C: Copy>(
        sim: &mut BattleSimulator,

        event: impl Event<EventReturnType = R, ContextType = C>,
        broadcaster_uid: MonsterUID,
        calling_context: C,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        
        let mut owned_event_handlers = sim.battle.event_handlers_for(event);

        if owned_event_handlers.is_empty() {
            return default;
        }
 
        sort_by_activation_order(&mut sim.battle.prng, &mut owned_event_handlers, |owned_event_handler| {
            owned_event_handler.activation_order
        });

        let mut relay = default;
        for OwnedEventHandler { event_handler, owner, filtering_options, .. } in owned_event_handlers.into_iter() {
            if Self::filter_event_handlers(&sim.battle, broadcaster_uid, owner, filtering_options) {
                // TODO: / INFO: Removed relaying the outcome of the previous handler from the event resolution. It will be
                // reintroduced if it ever turns out to be useful. Otherwise remove this comment. 
                relay = (event_handler.effect)(sim, calling_context);
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
        battle: &BattleState,
        broadcaster: MonsterUID,
        owner: MonsterUID,
        filter_options: EventFilteringOptions,
    ) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if broadcaster == owner {
                bitmask |= TargetFlags::SELF.bits()
            } // 0x01
            if battle.are_allies(owner, broadcaster) {
                bitmask |= TargetFlags::ALLIES.bits()
            } // 0x02
            if battle.are_opponents(owner, broadcaster) {
                bitmask |= TargetFlags::OPPONENTS.bits()
            } //0x04
              // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = filter_options.event_source.bits() == bitmask;
        let is_active_passed = battle.is_active_monster(owner);

        event_source_filter_passed && is_active_passed
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


