/*!
### Event System
Events are an integral part of the `monsim` engine, they enable the engine to model any reactive game mechanics,
such as abilities or items. An example would be the item *Life Orb*, which reacts to the `on_calculate_attack_stat`
event, by raising the attack by 50%. It also reacts to the `on_move_used` event, by draining 10% of the user's max
HP.

Each Event has a *broadcaster* and zero or more *receivers*. The broadcaster is responsible for emitting or triggering
the Event, and then each receiver returns an `EventHandler` that contains a callback function of the appropriate
type and some extra information about how and when to activate it, most prominently `EventFilteringOptions`. This is
then wrapped into an `OwnedEventHandler` that contains additional information about the owner of the EventHandler (i.e
the Monster whose EventHandler it is). The `EventDispatcher` is responsible for collecting, filtering and calling all
the callbacks of the appropriate EventHandlers.

An Event is broadcasted during the turn-loop for two major reasons:
1. To test if there are mechanics that forbid the next action, or alter it. These events are associated with
functions of the form `on_try_<something>`. A reactive EventHandler may choose to disable this. Think moves like
`Embargo` which prevents item use.
2. To inform the entities in the battle that something specific happened. These events are associated with
functions of the form `on_<something>_happened`. A reactive EventHandler may choose to do something every time
that specific thing happens, or only if further conditions are satisfied. `Passho Berry` reacts to the Event
`on_move_used` when used by an opponent, but only if the move is water-type and super-effective, which it then checks
manually.

The EventHandler returns a value, which tells the broadcaster how to modify the logic being evaluated. With the Life Orb
example, it returned a new value for the attack stat to be used when attacking. What kind of value an EventHandler returns
is decided by the Event it responds to. The `on_calculate_attack_stat` Event expects a `u16` - the modified attack stat.
Note that Life Orb may choose to return the original attack stat, which would correspond to having no effect. This is
desirable when an mechanic wants to affect the simulation only if certain conditions are met, it then returns the original
value when the condition is not met.

The Event Dispatcher folds the return values of all the EventHandlers it collected from the Battle, and then the return
value is returned to the broadcaster. The execution may be short-circuited if a special value, decided by the broadcaster,
is obtained. Certain Events also require the specification of a default value to return if there happens (as it often does)
that there are no EventHandlers for that particular Event at the moment. For "trial" events, which encapsulate checking if
some action will be successful, have always have a default value of `Outcome::Success<()>` which means they succeed by default,
as would be expected.
*/

pub mod contexts;
pub mod events;

use core::fmt::Debug;

use monsim_macros::mon;
use monsim_utils::{not, Nothing, Outcome, Percent, NOTHING};

use crate::{ActivationOrder, Battle, FieldPosition, MonsterID, PositionRelationFlags};
pub use contexts::*;

use super::ordering::sort_by_activation_order;

pub struct EventDispatcher;

impl EventDispatcher {
    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution "short-circuits", or returns early.
    pub fn dispatch_event<R: PartialEq + Copy + 'static, C: EventContext + Copy + 'static, B: Broadcaster + Copy + 'static>(
        battle: &mut Battle,

        event: impl Event<R, C, B>,
        broadcaster_id: B,
        event_context: C,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        #[cfg(feature = "debug")]
        let event_dispatch_start_time = std::time::SystemTime::now();

        let mut owned_event_handlers = battle.owned_event_handlers(event);

        if owned_event_handlers.is_empty() {
            return default;
        }

        sort_by_activation_order(&mut battle.prng, &mut owned_event_handlers, |owned_event_handler| {
            owned_event_handler.activation_order()
        });

        let mut relay = default;
        for owned_event_handler in owned_event_handlers.into_iter() {
            if EventDispatcher::does_event_pass_event_receivers_filtering_options(
                &battle,
                broadcaster_id,
                event_context.target(),
                owned_event_handler.owner_id(),
                owned_event_handler.event_filtering_options(),
            ) {
                relay = owned_event_handler.respond(battle, broadcaster_id, event_context, relay);
                // Return early if the relay becomes the short-circuiting value.
                if let Some(value) = short_circuit {
                    if relay == value {
                        return relay;
                    }
                };
            }
        }
        #[cfg(feature = "debug")]
        {
            let elapsed_time = event_dispatch_start_time.elapsed().expect("This should work every time.");
            println!("Event dispatch cycle took {:?}", elapsed_time);
        }

        relay
    }

    fn does_event_pass_event_receivers_filtering_options(
        battle: &Battle,
        event_broadcaster: impl Broadcaster,
        event_target_id: Option<MonsterID>,
        event_receiver_id: Option<MonsterID>,
        receiver_filtering_options: EventFilteringOptions,
    ) -> bool {
        let Some(event_receiver_id) = event_receiver_id else {
            // The event_receiver is the Environment, that auto-passes checks.
            return true;
        };

        let mut passes_filter;

        let EventFilteringOptions {
            only_if_broadcaster_is: allowed_broadcaster_position_relation_flags,
            only_if_target_is: allowed_target_position_relation_flags,
            only_if_receiver_is_active: requires_being_active,
        } = receiver_filtering_options;

        // First check - does the event receiver require themselves to be active? If so check if they are actually active.
        passes_filter = if requires_being_active { mon![event_receiver_id].is_active() } else { true };

        // Skip the rest of the calculation if it doesn't pass.
        if not!(passes_filter) {
            return false;
        };

        let event_receiver_field_position = mon![event_receiver_id]
            .board_position
            .field_position()
            .expect("For now we disallow the receiver to be benched. This is will probably be reverted in the future.");

        if let Some(event_broadcaster_id) = event_broadcaster.source() {
            // Second check - are the broadcaster's relation flags a subset of the allowed relation flags? that is, is the broadcaster within the allowed relations to the event receiver?
            let event_broadcaster_field_position = mon![event_broadcaster_id]
                .board_position
                .field_position()
                .expect("We assume broadcasters must be on the field.");

            passes_filter = FieldPosition::is_position_relation_allowed_by_flags(
                event_receiver_field_position,
                event_broadcaster_field_position,
                allowed_broadcaster_position_relation_flags,
            );
        }

        if not!(passes_filter) {
            return false;
        };

        // The event target is the contextual target for the action associated with this event. For example,
        // this could be the target of the current move.
        if let Some(event_target_id) = event_target_id {
            let event_target_field_position = mon![event_target_id].board_position.field_position();

            // The event target may have fainted by the time an EventHandler procs.
            if let Some(event_target_field_position) = event_target_field_position {
                passes_filter = FieldPosition::is_position_relation_allowed_by_flags(
                    event_receiver_field_position,
                    event_target_field_position,
                    allowed_target_position_relation_flags,
                );
            }
        }

        passes_filter
    }

    pub fn dispatch_trial_event<C: EventContext + Copy + 'static, B: Broadcaster + Copy + 'static>(
        battle: &mut Battle,

        event: impl Event<Outcome, C, B>,
        broadcaster_id: B,
        event_context: C,
    ) -> Outcome {
        EventDispatcher::dispatch_event(battle, event, broadcaster_id, event_context, Outcome::Success(NOTHING), Some(Outcome::Failure))
    }

    pub fn dispatch_notify_event<C: EventContext + Copy + 'static, B: Broadcaster + Copy + 'static>(
        battle: &mut Battle,

        event: impl Event<Nothing, C, B>,
        broadcaster_id: B,
        event_context: C,
    ) {
        EventDispatcher::dispatch_event(battle, event, broadcaster_id, event_context, NOTHING, None)
    }
}

pub trait Event<R: Copy + Sized, C: EventContext + Copy + Sized, B: Broadcaster + Copy = MonsterID> {
    fn get_event_handler_with_receiver(&self, event_listener: &'static dyn EventListener) -> Option<EventHandler<R, C, MonsterID, B>>;
    fn get_event_handler_without_receiver(&self, event_listener: &'static dyn EventListener<Nothing>) -> Option<EventHandler<R, C, Nothing, B>>;
}

pub trait Broadcaster {
    fn source(&self) -> Option<MonsterID> {
        None
    }
}

impl Broadcaster for MonsterID {
    fn source(&self) -> Option<MonsterID> {
        Some(*self)
    }
}

impl Broadcaster for Nothing {}

pub trait EventListener<V = MonsterID> {
    fn on_try_move_handler(&self) -> Option<EventHandler<Outcome, MoveUseContext, V>> {
        None
    }

    fn on_move_used_handler(&self) -> Option<EventHandler<Nothing, MoveUseContext, V>> {
        None
    }

    fn on_damaging_move_used_handler(&self) -> Option<EventHandler<Nothing, MoveUseContext, V>> {
        None
    }

    fn on_status_move_used_handler(&self) -> Option<EventHandler<Nothing, MoveUseContext, V>> {
        None
    }

    fn on_calculate_accuracy_handler(&self) -> Option<EventHandler<u16, MoveHitContext, V>> {
        None
    }

    fn on_calculate_accuracy_stage_handler(&self) -> Option<EventHandler<i8, MoveHitContext, V>> {
        None
    }

    fn on_calculate_evasion_stage_handler(&self) -> Option<EventHandler<i8, MoveHitContext, V>> {
        None
    }

    fn on_calculate_crit_stage_handler(&self) -> Option<EventHandler<u8, MoveHitContext, V>> {
        None
    }

    fn on_calculate_crit_damage_multiplier_handler(&self) -> Option<EventHandler<Percent, MoveHitContext, V>> {
        None
    }

    fn on_try_move_hit_handler(&self) -> Option<EventHandler<Outcome<Nothing>, MoveHitContext, V>> {
        None
    }

    fn on_move_hit_handler(&self) -> Option<EventHandler<Nothing, MoveHitContext, V>> {
        None
    }

    fn on_calculate_attack_stat_handler(&self) -> Option<EventHandler<u16, MoveHitContext, V>> {
        None
    }

    fn on_calculate_attack_stage_handler(&self) -> Option<EventHandler<i8, MoveHitContext, V>> {
        None
    }

    fn on_calculate_defense_stat_handler(&self) -> Option<EventHandler<u16, MoveHitContext, V>> {
        None
    }

    fn on_calculate_defense_stage_handler(&self) -> Option<EventHandler<i8, MoveHitContext, V>> {
        None
    }

    fn on_modify_damage_handler(&self) -> Option<EventHandler<u16, MoveHitContext, V>> {
        None
    }

    fn on_damage_dealt_handler(&self) -> Option<EventHandler<Nothing, Nothing, V>> {
        None
    }

    fn on_try_activate_ability_handler(&self) -> Option<EventHandler<Outcome<Nothing>, AbilityActivationContext, V>> {
        None
    }

    fn on_ability_activated_handler(&self) -> Option<EventHandler<Nothing, AbilityActivationContext, V>> {
        None
    }

    fn on_try_stat_change_handler(&self) -> Option<EventHandler<Outcome<Nothing>, StatChangeContext, V>> {
        None
    }

    fn on_modify_stat_change_handler(&self) -> Option<EventHandler<i8, StatChangeContext, V>> {
        None
    }

    fn on_stat_changed_handler(&self) -> Option<EventHandler<Nothing, StatChangeContext, V>> {
        None
    }

    fn on_try_inflict_volatile_status_handler(&self) -> Option<EventHandler<Outcome<Nothing>, Nothing, V>> {
        None
    }

    fn on_volatile_status_inflicted_handler(&self) -> Option<EventHandler<Nothing, Nothing, V>> {
        None
    }

    fn on_try_inflict_persistent_status_handler(&self) -> Option<EventHandler<Outcome<Nothing>, Nothing, V>> {
        None
    }

    fn on_persistent_status_inflicted_handler(&self) -> Option<EventHandler<Nothing, Nothing, V>> {
        None
    }

    fn on_try_use_held_item_handler(&self) -> Option<EventHandler<Outcome<Nothing>, ItemUseContext, V>> {
        None
    }

    fn on_held_item_used_handler(&self) -> Option<EventHandler<Nothing, ItemUseContext, V>> {
        None
    }

    fn on_turn_end_handler(&self) -> Option<EventHandler<Nothing, Nothing, V, Nothing>> {
        None
    }
}

impl<T> Debug for dyn EventListener<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Event Listener>")
    }
}

pub struct NullEventListener;

impl EventListener for NullEventListener {}

impl EventListener<Nothing> for NullEventListener {}

/// `fn(battle: &mut BattleState, broadcaster_id: B, receiver_id: ActorID, context: C, relay: R) -> event_outcome: R`
pub type EventResponse<R, C, V, B> = fn(&mut Battle, B, V, C, R) -> R;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventHandler<R, C, V, B = MonsterID> {
    pub response: EventResponse<R, C, V, B>,
    pub event_filtering_options: EventFilteringOptions,
}

pub trait OwnedEventHandler<R, C, B> {
    fn respond(&self, battle: &mut Battle, broadcaster_id: B, context: C, default: R) -> R;

    fn activation_order(&self) -> ActivationOrder;

    fn owner_id(&self) -> Option<MonsterID>;

    fn event_filtering_options(&self) -> EventFilteringOptions;
}

impl<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy> Clone for Box<dyn OwnedEventHandler<R, C, B>> {
    fn clone(&self) -> Self {
        self.to_owned()
    }
}

impl<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy> OwnedEventHandler<R, C, B> for OwnedEventHandlerWithReceiver<R, C, B> {
    fn respond(&self, battle: &mut Battle, broadcaster_id: B, context: C, default: R) -> R {
        (self.event_handler.response)(battle, broadcaster_id, self.owner_id, context, default)
    }

    fn activation_order(&self) -> ActivationOrder {
        self.activation_order
    }

    fn owner_id(&self) -> Option<MonsterID> {
        Some(self.owner_id)
    }

    fn event_filtering_options(&self) -> EventFilteringOptions {
        self.event_handler.event_filtering_options
    }
}

impl<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy> OwnedEventHandler<R, C, B> for OwnedEventHandlerWithoutReceiver<R, C, B> {
    fn respond(&self, battle: &mut Battle, broadcaster_id: B, context: C, default: R) -> R {
        (self.event_handler.response)(battle, broadcaster_id, NOTHING, context, default)
    }

    fn activation_order(&self) -> ActivationOrder {
        self.activation_order
    }

    fn owner_id(&self) -> Option<MonsterID> {
        None
    }

    fn event_filtering_options(&self) -> EventFilteringOptions {
        self.event_handler.event_filtering_options
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedEventHandlerWithReceiver<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy> {
    pub event_handler: EventHandler<R, C, MonsterID, B>,
    pub owner_id: MonsterID,
    pub activation_order: ActivationOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedEventHandlerWithoutReceiver<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy> {
    pub event_handler: EventHandler<R, C, Nothing, B>,
    pub activation_order: ActivationOrder,
}

// This tells asscociated EventHandlers whether to fire or not
/// in response to a certain kind of Event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFilteringOptions {
    /// Filters the EventHandler's response based on the relationship between the
    /// broadcaster and the receiver. Does nothing if there is no broadcaster.
    pub only_if_broadcaster_is: PositionRelationFlags,
    /// Filters the EventHandler based on the relationship between the target and the
    /// receiver. Does nothing if the event context has no clear target.
    pub only_if_target_is: PositionRelationFlags,
    /// If `true` the EventHandler only responds to the Event if its receiver is active.
    ///
    /// If `false`, the EventHandler ignores the whether the receiver is active or not
    /// (This could useful for abilities like Regenerator).
    pub only_if_receiver_is_active: bool,
}

impl EventFilteringOptions {
    pub const fn default() -> EventFilteringOptions {
        EventFilteringOptions {
            only_if_broadcaster_is: PositionRelationFlags::ADJACENT
                .union(PositionRelationFlags::NONADJACENT)
                .union(PositionRelationFlags::OPPONENTS),
            only_if_target_is: PositionRelationFlags::SELF,
            only_if_receiver_is_active: true,
        }
    }
}
