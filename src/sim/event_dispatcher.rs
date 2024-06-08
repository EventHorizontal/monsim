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

The EventHandler usually returns a value that has to do with the specific event being called. With the Life Orb example,
it returned a new value for the attack stat to be used when attacking. What kind of value an EventHandler returns
is decided by the Event it responds to. The `on_calculate_attack_stat` Event expects a `u16` - the modified attack stat.
Note that Life Orb may choose to return the initial attack, which would correspond to having no effect. This is
desirable when an mechanic wants to affect something only if certain conditions are met.

Once all the EventHandlers are exhausted, the Event resolution ends and the return value is returned to the broadcaster.
The execution may be short-circuited if a special value, decided by the broadcaster, is obtained. Certain Events also
require the specification of a default value to return if there happens (as it often does) that there are no EventHandlers
for that particular Event at the moment.
*/

use core::fmt::Debug;

mod events;

use crate::{
    sim::{game_mechanics::MonsterID, ordering::sort_by_activation_order, Battle, EventHandlerSelector, Nothing, Outcome},
    ActivationOrder, FieldPosition,
};
use contexts::*;
pub use events::*;
use monsim_macros::mon;
use monsim_utils::{not, Percent, NOTHING};

use super::targetting::PositionRelationFlags;

#[derive(Debug, Clone)]
pub struct EventDispatcher;

impl EventDispatcher {
    pub fn dispatch_trial_event<C: EventContext + Copy, B: Broadcaster + Copy>(
        battle: &mut Battle,

        broadcaster_id: B,
        event_handler_selector: EventHandlerSelector<Outcome<Nothing>, C, B>,
        event_context: C,
    ) -> Outcome<Nothing> {
        EventDispatcher::dispatch_event(
            battle,
            broadcaster_id,
            event_handler_selector,
            event_context,
            Outcome::Success(NOTHING),
            Some(Outcome::Failure),
        )
    }

    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution "short-circuits", or returns early.
    pub fn dispatch_event<R: PartialEq + Copy, C: EventContext + Copy, B: Broadcaster + Copy>(
        battle: &mut Battle,

        broadcaster_id: B,
        event_handler_selector: EventHandlerSelector<R, C, B>,
        event_context: C,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        let mut owned_event_handlers = event_handler_selector(&mut battle.event_handler_cache);
        if owned_event_handlers.is_empty() {
            return default;
        }

        sort_by_activation_order(&mut battle.prng, &mut owned_event_handlers, |owned_event_handler| {
            owned_event_handler.activation_order
        });

        let mut relay = default;
        for OwnedEventHandler { event_handler, owner_id, .. } in owned_event_handlers.into_iter() {
            if EventDispatcher::does_event_pass_event_receivers_filtering_options(
                &battle,
                broadcaster_id,
                event_context.target(),
                *owner_id,
                event_handler.event_filtering_options,
            ) {
                relay = (event_handler.response)(battle, broadcaster_id, *owner_id, event_context, relay);
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
        battle: &Battle,
        event_broadcaster: impl Broadcaster,
        event_target_id: Option<MonsterID>,
        event_receiver_id: MonsterID,
        receiver_filtering_options: EventFilteringOptions,
    ) -> bool {
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
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EventHandlerCache {
    pub(crate) is_dirty: bool,
    pub(crate) current_owner_info: Option<(MonsterID, ActivationOrder)>,
    /// This EventHandler is triggered when a move is about to be used. This EventHandler is to return an `Outcome`
    /// indicating whether the move should succeed.
    pub on_try_move: Vec<OwnedEventHandler<Outcome<Nothing>, MoveUseContext>>,
    /// This EventHandler is triggered when a move is used successfully.
    pub on_move_used: Vec<OwnedEventHandler<Nothing, MoveUseContext>>,
    /// This EventHandler is meant only to be a base for `on_damaging_move_used` and `on_status_move_used`.
    pub on_damaging_move_used: Vec<OwnedEventHandler<Nothing, MoveUseContext>>,
    /// This EventHandler is triggered when a status move is used successfully.
    pub on_status_move_used: Vec<OwnedEventHandler<Nothing, MoveUseContext>>,
    /// This EventHandler is triggered after the accuracy to be used in move miss calculation is calculated. This
    /// EventHandler is to return a `u16` representing a possibly modified _base_ accuracy to be used by the move.
    /// If the EventHandler wishes to leave the accuracy unchanged, say if a certain condition is met, then it can
    /// pass back the original accuracy, which is relayed to this EventHandler.
    pub on_calculate_accuracy: Vec<OwnedEventHandler<u16, MoveHitContext>>,
    pub on_calculate_accuracy_stage: Vec<OwnedEventHandler<i8, MoveHitContext>>,
    pub on_calculate_evasion_stage: Vec<OwnedEventHandler<i8, MoveHitContext>>,
    pub on_calculate_crit_stage: Vec<OwnedEventHandler<u8, MoveHitContext>>,
    pub on_calculate_crit_damage_multiplier: Vec<OwnedEventHandler<Percent, MoveHitContext>>,
    /// This EventHandler is triggered when a individual move hit is about to be performed. This EventHandler is to
    /// return an `Outcome` indicating whether the hit should succeed.
    pub on_try_move_hit: Vec<OwnedEventHandler<Outcome<Nothing>, MoveHitContext>>,
    /// This EventHandler is triggered when a hit has been performed successfully.
    pub on_move_hit: Vec<OwnedEventHandler<Nothing, MoveHitContext>>,
    /// This EventHandler is triggered when a move is calculating the attack stat to be used. This EventHandler is to
    /// return a `u16` indicating a possibly modified attack stat to be used. If the EventHandler wishes to
    /// leave the attack unchanged, say if a certain condition is met, then it can pass back the original attack
    /// stat, which is relayed to this EventHandler.
    pub on_calculate_attack_stat: Vec<OwnedEventHandler<u16, MoveHitContext>>,
    pub on_calculate_attack_stage: Vec<OwnedEventHandler<i8, MoveHitContext>>,
    /// This EventHandler is triggered when a move is calculating the defense stat to be used. This EventHandler is to
    /// return a `u16` indicating a possibly modified defense stat to be used. If the EventHandler wishes to
    /// leave the defense unchanged, say if a certain condition is met, then it can pass back the original defense
    /// stat, which is relayed to this EventHandler.
    pub on_calculate_defense_stat: Vec<OwnedEventHandler<u16, MoveHitContext>>,
    pub on_calculate_defense_stage: Vec<OwnedEventHandler<i8, MoveHitContext>>,
    /// This EventHandler is triggered after a move's damage is calculated, giving the opportunity for the final damage
    /// to be modified. This EventHandler is to return a `u16` indicating a possibly modified damage value. If the
    /// EventHandler wishes to leave the damage unchanged, say if a certain condition is met, then it can pass back
    /// the original damage, which is relayed to this EventHandler.
    pub on_modify_damage: Vec<OwnedEventHandler<u16, MoveHitContext>>,
    /// This EventHandler is triggered after a move's damage has been dealt successfully.
    pub on_damage_dealt: Vec<OwnedEventHandler<Nothing, Nothing>>,
    /// This EventHandler is triggered when an ability is about to be activated. The EventHandler is to
    /// return an `Outcome` indicating whether the ability activation should succeed.
    pub on_try_activate_ability: Vec<OwnedEventHandler<Outcome<Nothing>, AbilityActivationContext>>,
    /// This EventHandler is triggered after an ability successfully activates.
    pub on_ability_activated: Vec<OwnedEventHandler<Nothing, AbilityActivationContext>>,
    /// This EventHandler is triggered when a stat is about to be changed. This EventHandler is to return an `Outcome`
    /// representing whether the stat change should succeed.
    pub on_try_stat_change: Vec<OwnedEventHandler<Outcome<Nothing>, StatChangeContext>>,
    /// This EventHandler is triggered when a stat is changed, allowing for the stat change to be modified.
    pub on_modify_stat_change: Vec<OwnedEventHandler<i8, StatChangeContext>>,
    /// This EventHandler is triggered after a stat is changed.
    pub on_stat_changed: Vec<OwnedEventHandler<Nothing, StatChangeContext>>,
    /// This EventHandler is triggered when a volatile status is about to be inflicted on a Monster. This EventHandler
    /// is to return and `Outcome` representing whether the infliction of the volatile status should succeed.
    pub on_try_inflict_volatile_status: Vec<OwnedEventHandler<Outcome<Nothing>, Nothing>>,
    /// This EventHandler is triggered after a volatile status has been successfully inflicted.
    pub on_volatile_status_inflicted: Vec<OwnedEventHandler<Nothing, Nothing>>,
    /// This EventHandler is triggered when a persistent status is about to be inflicted on a Monster. This EventHandler
    /// is to return and `Outcome` representing whether the infliction of the persistent status should succeed.
    pub on_try_inflict_persistent_status: Vec<OwnedEventHandler<Outcome<Nothing>, Nothing>>,
    /// This EventHandler is triggered after a presistent status has been successfully inflicted.
    pub on_persistent_status_inflicted: Vec<OwnedEventHandler<Nothing, Nothing>>,
    /// This EventHandler is triggered when a held item is about to be used. This EventHandler
    /// is to return and `Outcome` representing whether the use of the held item should succeed.
    pub on_try_use_held_item: Vec<OwnedEventHandler<Outcome<Nothing>, ItemUseContext>>,
    /// This EventHandler is triggered when a held item is used successfully.
    pub on_held_item_used: Vec<OwnedEventHandler<Nothing, ItemUseContext>>,
    /// This EventHandler is triggered at the end of each turn. This is a _temporal event_, such that it has no broadcaster.
    pub on_turn_end: Vec<OwnedEventHandler<Nothing, Nothing, Nothing>>,
}

impl EventHandlerCache {}

/// This tells asscociated EventHandlers whether to fire or not
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

/// `fn(battle: &mut BattleState, broadcaster_id: B, receiver_id: MonsterID, context: C, relay: R) -> event_outcome: R`
pub type EventResponse<R, C, B> = fn(&mut Battle, B, MonsterID, C, R) -> R;

/// Stores an `Effect` that gets simulated in response to an `Event` being triggered.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EventHandler<R: Copy, C: Copy, B: Broadcaster + Clone + Copy = MonsterID> {
    #[cfg(feature = "debug")]
    pub source_code_location: &'static str,
    pub response: EventResponse<R, C, B>,
    pub event_filtering_options: EventFilteringOptions,
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

pub trait EventContext {
    fn target(&self) -> Option<MonsterID> {
        None
    }
}

impl EventContext for Nothing {}

impl<R: Copy, C: Copy, B: Broadcaster + Copy> Debug for EventHandler<R, C, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "debug")]
        let out = {
            f.debug_struct("EventHandler")
                .field("source_code_location", &self.source_code_location)
                .finish()
        };
        #[cfg(not(feature = "debug"))]
        let out = { write!(f, "EventHandler debug information only available with feature flag \"debug\" turned on.") };
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedEventHandler<R: Copy, C: Copy, B: Broadcaster + Copy = MonsterID> {
    pub event_handler: EventHandler<R, C, B>,
    pub owner_id: MonsterID,
    pub activation_order: ActivationOrder,
}

pub mod contexts {
    use super::EventContext;
    use crate::{
        sim::{MonsterID, MoveID},
        AbilityID, ItemID, ModifiableStat,
    };
    use monsim_utils::MaxSizedVec;

    /// `move_user_id`: MonsterID of the Monster using the move.
    ///
    /// `move_used_id`: MoveID of the Move being used.
    ///
    /// `target_ids`: MonsterIDs of the Monsters the move is being used on.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveUseContext {
        pub move_user_id: MonsterID,
        pub move_used_id: MoveID,
        pub target_ids: MaxSizedVec<MonsterID, 6>,
    }

    impl EventContext for MoveUseContext {}

    impl MoveUseContext {
        pub fn new(move_used_id: MoveID, target_ids: MaxSizedVec<MonsterID, 6>) -> Self {
            Self {
                move_user_id: move_used_id.owner_id,
                move_used_id,
                target_ids,
            }
        }
    }

    /// `move_user_id`: MonsterID of the Monster hitting.
    ///
    /// `move_used_id`: MoveID of the Move being used.
    ///
    /// `target_id`: MonsterID of the Monster being hit.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveHitContext {
        pub move_user_id: MonsterID,
        pub move_used_id: MoveID,
        pub target_id: MonsterID,
        pub number_of_hits: u8,
        pub number_of_targets: u8,
    }

    impl MoveHitContext {
        pub fn new(move_used_id: MoveID, target_id: MonsterID, number_of_hits: u8, number_of_targets: u8) -> Self {
            Self {
                move_user_id: move_used_id.owner_id,
                move_used_id,
                target_id,
                number_of_hits,
                number_of_targets,
            }
        }
    }

    impl EventContext for MoveHitContext {
        fn target(&self) -> Option<MonsterID> {
            Some(self.target_id)
        }
    }

    /// `ability_owner_id`: MonsterID of the Monster whose ability is being used.
    ///
    /// `ability_used_id`: AbilityID of the Ability being used.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AbilityActivationContext {
        pub ability_owner_id: MonsterID,
        pub ability_used_id: AbilityID,
    }

    impl AbilityActivationContext {
        pub fn from_owner(ability_owner: MonsterID) -> Self {
            Self {
                ability_used_id: AbilityID { owner_id: ability_owner },
                ability_owner_id: ability_owner,
            }
        }
    }

    impl EventContext for AbilityActivationContext {}

    /// `active_monster_id`: MonsterID of the Monster to be switched out.
    ///
    /// `benched_monster_id`: MonsterID of the Monster to be switched in.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SwitchContext {
        pub active_monster_id: MonsterID,
        pub benched_monster_id: MonsterID,
    }

    impl SwitchContext {
        pub fn new(active_monster_id: MonsterID, benched_monster_id: MonsterID) -> Self {
            Self {
                active_monster_id,
                benched_monster_id,
            }
        }
    }

    impl EventContext for SwitchContext {}

    /// `active_monster_id`: MonsterID of the Monster to be switched out.
    ///
    /// `benched_monster_id`: MonsterID of the Monster to be switched in.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ItemUseContext {
        pub item_id: ItemID,
        pub item_holder_id: MonsterID,
    }

    impl ItemUseContext {
        pub fn from_holder(item_holder_id: MonsterID) -> Self {
            let item_id = ItemID::from_holder(item_holder_id);
            Self { item_id, item_holder_id }
        }
    }

    impl EventContext for ItemUseContext {}

    /// `active_monster_id`: MonsterID of the Monster to be switched out.
    ///
    /// `benched_monster_id`: MonsterID of the Monster to be switched in.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatChangeContext {
        pub affected_monster_id: MonsterID,
        pub stat: ModifiableStat,
        pub number_of_stages: i8,
    }

    impl EventContext for StatChangeContext {}
}
