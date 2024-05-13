use core::fmt::Debug;

pub mod events;
#[cfg(all(test, feature = "debug"))]
mod tests ;

use crate::{sim::{game_mechanics::MonsterID, ordering::sort_by_activation_order, BattleState, Nothing, Outcome, Percent}, BattleSimulator};
use contexts::*;
pub use events::*;
use monsim_utils::not;

use super::targetting::TargetFlags;

#[derive(Debug, Clone)]
pub struct EventDispatcher;

impl EventDispatcher {

    pub fn dispatch_trial_event<C: Copy>(
        sim: &mut BattleSimulator,

        broadcaster_id: MonsterID,
        event_handler_selector: fn(EventHandlerDeck) -> Vec<Option<EventHandler<Outcome, C>>>,
        event_context: C,
    ) -> Outcome {
        EventDispatcher::dispatch_event(
            sim, 
            broadcaster_id,
            event_handler_selector, 
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

        broadcaster_id: MonsterID,
        event_handler_selector: fn(EventHandlerDeck) -> Vec<Option<EventHandler<R, C>>>,
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
        for OwnedEventHandler { event_handler, owner_id, filtering_options, .. } in owned_event_handlers.into_iter() {
            if EventDispatcher::does_event_pass_event_receivers_filtering_options(&sim.battle, broadcaster_id, owner_id, filtering_options) {
                // INFO: Removed relaying the outcome of the previous handler from the event resolution. It will be
                // reintroduced if it ever turns out to be useful. Otherwise remove this comment. 
                relay = (event_handler.effect)(sim, owner_id, event_context);
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
        event_broadcaster_id: MonsterID,
        event_receiver_id: MonsterID,
        receiver_filtering_options: EventFilteringOptions,
    ) -> bool {

        let mut passes_filter;
        
        let EventFilteringOptions { allowed_broadcaster_relation_flags, requires_being_active } = receiver_filtering_options;

        // First check - does the event receiver require themselves to be active? If so check if they are actually active.
        passes_filter = if requires_being_active {
            battle.monster(event_receiver_id).is_active()
        } else {
            true
        };

        // Skip the rest of the calculation if it doesn't pass.
        if not!(passes_filter) { return false };

        // Second check - are the broadcaster's relation flags a subset of the allowed relation flags? that is, is the broadcaster
        // within the allowed relations to the event receiver?
        let mut broadcaster_relation_flags = TargetFlags::empty();
        let event_broadcaster_field_position = battle.monster(event_broadcaster_id)
            .board_position
            .field_position()
            .expect("We assume broadcasters must be on the field.");
        let event_receiver_field_position = battle.monster(event_receiver_id)
            .board_position
            .field_position();
        // The event receiver might be benched.
        if let Some(event_receiver_field_position) = event_receiver_field_position {
            if event_broadcaster_field_position.is_adjacent_to(event_receiver_field_position) {
                broadcaster_relation_flags |= TargetFlags::ADJACENT
            } else {
                broadcaster_relation_flags |= TargetFlags::NONADJACENT
            }
            // FEATURE: BENCHED adjacency flag?
        }
        if battle.are_opponents(event_broadcaster_id, event_receiver_id) {
            broadcaster_relation_flags |= TargetFlags::OPPONENTS
        } else if battle.are_allies(event_broadcaster_id, event_receiver_id) {
            broadcaster_relation_flags |= TargetFlags::ALLIES
        } else {
            broadcaster_relation_flags |= TargetFlags::SELF
        }

        passes_filter = allowed_broadcaster_relation_flags.contains(broadcaster_relation_flags);

        passes_filter
    }  
}

/// This tells asscociated EventHandlers whether to fire or not 
/// in response to a certain kind of Event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFilteringOptions {
    /// This field dictates which Monsters' event broadcasts for the EventHandler to
    /// respond to.
    pub allowed_broadcaster_relation_flags: TargetFlags,
    /// If `true` the EventHandler only responds to the Event if its owner is active.
    /// 
    /// If `false`, the EventHandler ignores the whether the owner is active or not. 
    /// (This could useful for abilities like Regenerator).
    pub requires_being_active: bool,
}

impl EventFilteringOptions {
    pub const fn default() -> EventFilteringOptions {
        EventFilteringOptions {
            allowed_broadcaster_relation_flags: TargetFlags::ADJACENT.union(TargetFlags::NONADJACENT).union(TargetFlags::OPPONENTS),
            requires_being_active: true,
        }
    }
}


