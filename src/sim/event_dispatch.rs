use core::fmt::Debug;

pub mod events;
#[cfg(all(test, feature = "debug"))]
mod tests ;

use crate::{sim::{game_mechanics::MonsterID, ordering::sort_by_activation_order, BattleState, Nothing, Outcome, Percent}, BattleSimulator};
use contexts::*;
pub use events::*;
use monsim_utils::{not, NOTHING};

use super::targetting::TargetFlags;

#[derive(Debug, Clone)]
pub struct EventDispatcher;

impl EventDispatcher {

    pub fn dispatch_trial_event<C: Copy, B: Broadcaster + Copy>(
        sim: &mut BattleSimulator,

        broadcaster_id: B,
        event_handler_selector: fn(EventHandlerDeck) -> Vec<Option<EventHandler<Outcome<Nothing>, C, B>>>,
        event_context: C,
    ) -> Outcome<Nothing> {
        EventDispatcher::dispatch_event(
            sim, 
            broadcaster_id,
            event_handler_selector, 
            event_context, 
            Outcome::Success(NOTHING), 
            Some(Outcome::Failure)
        )
    }

    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn dispatch_event<R: PartialEq + Copy, C: Copy, B: Broadcaster + Copy>(
        sim: &mut BattleSimulator,

        broadcaster_id: B,
        event_handler_selector: fn(EventHandlerDeck) -> Vec<Option<EventHandler<R, C, B>>>,
        event_context: C,
        default: R,
        short_circuit: Option<R>,
    ) -> R {

        let mut owned_event_handlers = sim.battle.owned_event_handlers(event_handler_selector);

        if owned_event_handlers.is_empty() {
            return default;
        }
 
        sort_by_activation_order(&mut sim.battle.prng, &mut owned_event_handlers, |owned_event_handler| {
            owned_event_handler.activation_order
        });

        let mut relay = default;
        for OwnedEventHandler { event_handler, owner_id, .. } in owned_event_handlers.into_iter() {
            if EventDispatcher::does_event_pass_event_receivers_filtering_options(&sim.battle, broadcaster_id, owner_id, event_handler.event_filtering_options) {
                
                relay = (event_handler.response)(sim, broadcaster_id, owner_id, event_context, relay);
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

    fn does_event_pass_event_receivers_filtering_options(
        battle: &BattleState,
        event_broadcaster_id: impl Broadcaster,
        event_receiver_id: MonsterID,
        receiver_filtering_options: EventFilteringOptions,
    ) -> bool {

        let mut passes_filter;
        
        let EventFilteringOptions { 
            only_if_broadcaster_is: allowed_broadcaster_relation_flags, 
            only_if_target_is: allowed_target_relation_flags,
            only_if_receiver_is_active: requires_being_active 
        } = receiver_filtering_options;

        // First check - does the event receiver require themselves to be active? If so check if they are actually active.
        passes_filter = if requires_being_active {
            battle.monster(event_receiver_id).is_active()
        } else {
            true
        };

        // Skip the rest of the calculation if it doesn't pass.
        if not!(passes_filter) { return false };

        if let Some(event_broadcaster_id) = event_broadcaster_id.is_sourced() {
            // Second check - are the broadcaster's relation flags a subset of the allowed relation flags? that is, is the broadcaster
            // within the allowed relations to the event receiver?
            let mut broadcaster_relation_flags = TargetFlags::empty();
            let event_broadcaster_field_position = battle.monster(event_broadcaster_id)
                .board_position
                .field_position()
                .expect("We assume broadcasters must be on the field.");
            // This is an optional value because it may be that the event receiver is benched.
            let event_receiver_field_position = battle.monster(event_receiver_id)
                .board_position
                .field_position();
            
            if battle.are_opponents(event_broadcaster_id, event_receiver_id) {
                broadcaster_relation_flags |= TargetFlags::OPPONENTS;
            } else if battle.are_allies(event_broadcaster_id, event_receiver_id) {
                broadcaster_relation_flags |= TargetFlags::ALLIES;
            } else {
                broadcaster_relation_flags |= TargetFlags::SELF;
            }
            // Adjacency doesn't apply to self
            if not!(broadcaster_relation_flags.contains(TargetFlags::SELF)) {
                if let Some(event_receiver_field_position) = event_receiver_field_position {
                    if event_broadcaster_field_position.is_adjacent_to(event_receiver_field_position) {
                        broadcaster_relation_flags |= TargetFlags::ADJACENT;
                    } else {
                        broadcaster_relation_flags |= TargetFlags::NONADJACENT;
                    }
                }
            }
            passes_filter = allowed_broadcaster_relation_flags.contains(broadcaster_relation_flags);
        }

        passes_filter
    }  
}

/// This tells asscociated EventHandlers whether to fire or not 
/// in response to a certain kind of Event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFilteringOptions {
    /// The EventHandler will only activate if the broadcaster is within the allowed
    /// relations to the owner of the EventHandler, e.g. if `only_if_broadcaster_is
    /// = TargetFlags::OPPONENTS` then the EventHandler only procs if its owner (the 
    /// receiver) is an opponent of the broadcaster of the event.
    pub only_if_broadcaster_is: TargetFlags,
    /// Filters the EventHandler based on the relationship between the target and the
    /// receiver. Does nothing if context of the event has no specific target.
    pub only_if_target_is: TargetFlags,
    /// If `true` the EventHandler only responds to the Event if its owner is active.
    /// 
    /// If `false`, the EventHandler ignores the whether the owner is active or not. 
    /// (This could useful for abilities like Regenerator).
    pub only_if_receiver_is_active: bool,
}

impl EventFilteringOptions {
    pub const fn default() -> EventFilteringOptions {
        EventFilteringOptions {
            only_if_broadcaster_is: TargetFlags::ADJACENT.union(TargetFlags::NONADJACENT).union(TargetFlags::OPPONENTS),
            only_if_target_is: TargetFlags::SELF,
            only_if_receiver_is_active: true,
        }
    }
}


