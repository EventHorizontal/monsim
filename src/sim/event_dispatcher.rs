pub mod contexts;
pub mod events;

use core::fmt::Debug;

use monsim_macros::mon;
use monsim_utils::{not, Nothing, Outcome, Percent, NOTHING};

use crate::{
    status::{PersistentStatusID, VolatileStatusID},
    AbilityID, ActivationOrder, Battle, FieldPosition, ItemID, MechanicKind, MonsterID, MoveID, PositionRelationFlags, Stat, TeamID, TrapID,
};
pub use contexts::*;

use super::{ordering::sort_by_activation_order, WeatherID};

pub struct EventDispatcher;

impl EventDispatcher {
    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution "short-circuits", or returns early.
    pub fn dispatch_event<Ctx: EventContext + 'static, Rtn: EventReturnable + 'static, Bdc: Broadcaster + 'static>(
        battle: &mut Battle,

        event: impl Event<Ctx, Rtn, Bdc>,
        broadcaster: Bdc,
        event_context: Ctx,
        default: Rtn,
        short_circuit: Option<Rtn>,
    ) -> Rtn {
        #[cfg(feature = "debug")]
        println!["(Dispatching {})", event.name()];

        let mut event_handlers = EventDispatcher::collect_event_handlers_for(battle, event);

        if event_handlers.is_empty() {
            return default;
        }

        sort_by_activation_order(&mut battle.prng, &mut event_handlers, |owned_event_handler| {
            owned_event_handler.activation_order()
        });

        let mut relay = default;
        for event_handler in event_handlers.into_iter() {
            #[cfg(feature = "debug")]
            println!["\t└── (EventHandler for {mechanic} considered)", mechanic = event_handler.mechanic_name(battle)];
            if EventDispatcher::does_event_handler_pass_filters(
                battle,
                broadcaster.as_id(),
                event_context.target(),
                event_handler.owner_id(),
                event_handler.mechanic_kind(),
                event_handler.event_filtering_options(),
            ) {
                relay = event_handler.respond(battle, broadcaster, event_context, relay);
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

    pub fn collect_event_handlers_for<Ctx: EventContext + 'static, Rtn: EventReturnable + 'static, Bdc: Broadcaster + 'static>(
        battle: &Battle,
        event: impl Event<Ctx, Rtn, Bdc>,
    ) -> Vec<Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>> {
        let mut output_event_handlers = Vec::new();
        // Collect all the event handlers from each team
        for team in battle.teams().iter() {
            for monster in team.monsters() {
                let owner_id = monster.id;
                // of the Monster itself
                if let Some(owned_event_handler) = event.get_event_handler_with_receiver(monster.species().event_listener()).map(|event_handler| {
                    Box::new(EventHandlerWithOwner {
                        event_handler,
                        receiver_id: owner_id,
                        activation_order: ActivationOrder {
                            priority: 0,
                            speed: monster.stat(Stat::Speed),
                            order: 0,
                        },
                        mechanic_id: monster.id,
                        mechanic_kind: MechanicKind::Monster,
                    }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>
                }) {
                    output_event_handlers.push(owned_event_handler);
                }

                // from the Monster's ability
                if let Some(owned_event_handler) = event.get_event_handler_with_receiver(monster.ability.event_listener()).map(|event_handler| {
                    Box::new(EventHandlerWithOwner {
                        event_handler,
                        receiver_id: owner_id,
                        mechanic_id: monster.ability().id,
                        activation_order: ActivationOrder {
                            priority: 0,
                            speed: monster.stat(Stat::Speed),
                            order: monster.ability.order(),
                        },
                        mechanic_kind: MechanicKind::Ability,
                    }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>
                }) {
                    output_event_handlers.push(owned_event_handler);
                }

                // INFO: Moves don't have EventHandlers any more. This may be reverted in the future.

                // from the Monster's volatile statuses
                monster.volatile_statuses.into_iter().for_each(|volatile_status| {
                    if let Some(owned_event_handler) = event.get_event_handler_with_receiver(volatile_status.event_listener()).map(|event_handler| {
                        Box::new(EventHandlerWithOwner {
                            event_handler,
                            receiver_id: owner_id,
                            mechanic_id: volatile_status.id,
                            activation_order: ActivationOrder {
                                priority: 0,
                                speed: monster.stat(Stat::Speed),
                                order: 0,
                            },
                            mechanic_kind: MechanicKind::VolatileStatus,
                        }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>
                    }) {
                        output_event_handlers.push(owned_event_handler)
                    }
                });

                // from the Monster's persistent status
                if let Some(persistent_status) = monster.persistent_status {
                    if let Some(event_handler) = event.get_event_handler_with_receiver(persistent_status.event_handlers()) {
                        let owned_event_handler = Box::new(EventHandlerWithOwner {
                            event_handler,
                            receiver_id: owner_id,
                            activation_order: ActivationOrder {
                                priority: 0,
                                speed: monster.stat(Stat::Speed),
                                order: 0,
                            },
                            mechanic_id: persistent_status.id,
                            mechanic_kind: MechanicKind::PersistentStatus,
                        }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>;
                        output_event_handlers.push(owned_event_handler);
                    }
                }

                // from the Monster's held item
                if let Some(held_item) = monster.held_item {
                    if let Some(event_handler) = event.get_event_handler_with_receiver(held_item.event_listener()) {
                        let owned_event_handler = Box::new(EventHandlerWithOwner {
                            event_handler,
                            receiver_id: owner_id,
                            activation_order: ActivationOrder {
                                priority: 0,
                                speed: monster.stat(Stat::Speed),
                                order: 0,
                            },
                            mechanic_id: held_item.id,
                            mechanic_kind: MechanicKind::Item,
                        }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>;
                        output_event_handlers.push(owned_event_handler);
                    }
                }
            }
        }
        // From the weather
        if let Some(weather) = battle.environment().weather() {
            if let Some(event_handler) = event.get_event_handler_without_receiver(weather.event_listener()) {
                let owned_event_handler = Box::new(EventHandlerWithOwner {
                    event_handler,
                    receiver_id: NOTHING,
                    mechanic_id: WeatherID,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: 0,
                        order: 0,
                    },
                    mechanic_kind: MechanicKind::Weather,
                }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>;
                output_event_handlers.push(owned_event_handler);
            }
        }

        // From the terrain
        if let Some(terrain) = battle.environment().terrain() {
            if let Some(event_handler) = event.get_event_handler_without_receiver(terrain.event_listener()) {
                let owned_event_handler = Box::new(EventHandlerWithOwner {
                    event_handler,
                    receiver_id: NOTHING,
                    mechanic_id: NOTHING,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: 0,
                        order: 0,
                    },
                    mechanic_kind: MechanicKind::Terrain,
                }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>;
                output_event_handlers.push(owned_event_handler);
            }
        }

        // From the entry hazards
        for trap in battle.environment().traps().iter().flatten() {
            if let Some(event_handler) = event.get_event_handler_without_receiver(trap.event_listener()) {
                let owned_event_handler = Box::new(EventHandlerWithOwner {
                    event_handler,
                    receiver_id: NOTHING,
                    mechanic_id: trap.id,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: 0,
                        order: 0,
                    },
                    mechanic_kind: MechanicKind::Trap { team_id: trap.id.team_id },
                }) as Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>>;
                output_event_handlers.push(owned_event_handler);
            }
        }

        output_event_handlers
    }

    fn does_event_handler_pass_filters(
        battle: &Battle,

        optional_broadcaster_id: Option<MonsterID>,
        optional_target_id: Option<MonsterID>,
        optional_receiver_id: Option<MonsterID>,

        event_listener_mechanic_kind: MechanicKind,
        receiver_filtering_options: EventFilteringOptions,
    ) -> bool {
        // TODO: Some of the branches probably need some refinement.
        match event_listener_mechanic_kind {
            MechanicKind::Trap { team_id } => {
                if let Some(event_broadcaster_id) = optional_broadcaster_id {
                    TeamID::are_same(team_id, mon![event_broadcaster_id].id.team_id)
                } else {
                    false
                }
            }
            MechanicKind::Terrain | MechanicKind::Weather => {
                if let Some(event_broadcaster_id) = optional_broadcaster_id {
                    if not![mon![event_broadcaster_id].is_grounded()] {
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            _ => {
                let Some(event_receiver_id) = optional_receiver_id else {
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

                if let Some(event_broadcaster_id) = optional_broadcaster_id {
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
                if let Some(event_target_id) = optional_target_id {
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
    }

    pub fn dispatch_trial_event<Ctx: EventContext + 'static, Bdc: Broadcaster + 'static>(
        battle: &mut Battle,

        event: impl Event<Ctx, Outcome, Bdc>,
        broadcaster_id: Bdc,
        event_context: Ctx,
    ) -> Outcome {
        EventDispatcher::dispatch_event(battle, event, broadcaster_id, event_context, Outcome::Success(NOTHING), Some(Outcome::Failure))
    }

    pub fn dispatch_notify_event<Ctx: EventContext + 'static, Bdc: Broadcaster + 'static>(
        battle: &mut Battle,

        event: impl Event<Ctx, Nothing, Bdc>,
        broadcaster_id: Bdc,
        event_context: Ctx,
    ) {
        EventDispatcher::dispatch_event(battle, event, broadcaster_id, event_context, NOTHING, None)
    }
}

// Event -------------------------------------------------- //

pub trait Event<Ctx: EventContext, Rtn: EventReturnable, Bdc: Broadcaster = MonsterID> {
    #[cfg(feature = "debug")]
    fn name(&self) -> &'static str;

    fn get_event_handler_with_receiver<Mch: MechanicID>(
        &self,
        event_listener: &'static dyn EventListener<Mch>,
    ) -> Option<EventHandler<Ctx, Rtn, Mch, MonsterID, Bdc>>;

    fn get_event_handler_without_receiver<Mch: MechanicID>(
        &self,
        event_listener: &'static dyn EventListener<Mch, Nothing>,
    ) -> Option<EventHandler<Ctx, Rtn, Mch, Nothing, Bdc>>;
}

// EventListener ------------------------------------------ //

pub trait EventListener<Mch: MechanicID, Rcv: Receiver = MonsterID> {
    fn on_try_move_handler(&self) -> Option<EventHandler<MoveUseContext, Outcome, Mch, Rcv>> {
        None
    }

    fn on_move_used_handler(&self) -> Option<EventHandler<MoveUseContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_damaging_move_used_handler(&self) -> Option<EventHandler<MoveUseContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_status_move_used_handler(&self) -> Option<EventHandler<MoveUseContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_calculate_accuracy_handler(&self) -> Option<EventHandler<MoveHitContext, u16, Mch, Rcv>> {
        None
    }

    fn on_calculate_accuracy_stage_handler(&self) -> Option<EventHandler<MoveHitContext, i8, Mch, Rcv>> {
        None
    }

    fn on_calculate_evasion_stage_handler(&self) -> Option<EventHandler<MoveHitContext, i8, Mch, Rcv>> {
        None
    }

    fn on_calculate_crit_stage_handler(&self) -> Option<EventHandler<MoveHitContext, u8, Mch, Rcv>> {
        None
    }

    fn on_calculate_crit_damage_multiplier_handler(&self) -> Option<EventHandler<MoveHitContext, Percent, Mch, Rcv>> {
        None
    }

    fn on_try_move_hit_handler(&self) -> Option<EventHandler<MoveHitContext, Outcome, Mch, Rcv>> {
        None
    }

    fn on_move_hit_handler(&self) -> Option<EventHandler<MoveHitContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_calculate_attack_stat_handler(&self) -> Option<EventHandler<MoveHitContext, u16, Mch, Rcv>> {
        None
    }

    fn on_calculate_attack_stage_handler(&self) -> Option<EventHandler<MoveHitContext, i8, Mch, Rcv>> {
        None
    }

    fn on_calculate_defense_stat_handler(&self) -> Option<EventHandler<MoveHitContext, u16, Mch, Rcv>> {
        None
    }

    fn on_calculate_defense_stage_handler(&self) -> Option<EventHandler<MoveHitContext, i8, Mch, Rcv>> {
        None
    }

    fn on_modify_damage_handler(&self) -> Option<EventHandler<MoveHitContext, u16, Mch, Rcv>> {
        None
    }

    /// This event is triggered when the broadcaster receives damage.
    fn on_damage_received_handler(&self) -> Option<EventHandler<Nothing, Nothing, Mch, Rcv>> {
        None
    }

    fn on_health_recovered_handler(&self) -> Option<EventHandler<Nothing, Nothing, Mch, Rcv>> {
        None
    }

    fn on_try_activate_ability_handler(&self) -> Option<EventHandler<AbilityActivationContext, Outcome, Mch, Rcv>> {
        None
    }

    fn on_ability_activated_handler(&self) -> Option<EventHandler<AbilityActivationContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_try_stat_change_handler(&self) -> Option<EventHandler<StatChangeContext, Outcome, Mch, Rcv>> {
        None
    }

    fn on_modify_stat_change_handler(&self) -> Option<EventHandler<StatChangeContext, i8, Mch, Rcv>> {
        None
    }

    fn on_stat_changed_handler(&self) -> Option<EventHandler<StatChangeContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_try_inflict_volatile_status_handler(&self) -> Option<EventHandler<InflictVolatileStatusContext, Outcome, Mch, Rcv>> {
        None
    }

    fn on_volatile_status_inflicted_handler(&self) -> Option<EventHandler<InflictVolatileStatusContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_try_inflict_persistent_status_handler(&self) -> Option<EventHandler<InflictPersistentStatusContext, Outcome, Mch, Rcv>> {
        None
    }

    fn on_persistent_status_inflicted_handler(&self) -> Option<EventHandler<InflictPersistentStatusContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_try_use_held_item_handler(&self) -> Option<EventHandler<ItemUseContext, Outcome, Mch, Rcv>> {
        None
    }

    fn on_held_item_used_handler(&self) -> Option<EventHandler<ItemUseContext, Nothing, Mch, Rcv>> {
        None
    }

    fn on_monster_enter_battle_handler(&self) -> Option<EventHandler<Nothing, Nothing, Mch, Rcv>> {
        None
    }

    fn on_monster_exit_battle_handler(&self) -> Option<EventHandler<Nothing, Nothing, Mch, Rcv>> {
        None
    }

    fn on_turn_end_handler(&self) -> Option<EventHandler<Nothing, Nothing, Mch, Rcv, Nothing>> {
        None
    }
}

impl<Mch, Rcv> Debug for dyn EventListener<Mch, Rcv> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Event Listener>")
    }
}

pub struct NullEventListener;

impl<Mch: MechanicID> EventListener<Mch> for NullEventListener {}

impl<Mch: MechanicID> EventListener<Mch, Nothing> for NullEventListener {}

// EventHandlers --------------------------------------------------- //

/// `Bdc` is the ID of the broadcaster, that is, the monster that triggered the event. `Bdc` is `Nothing`/`()` if the event
/// was triggered by a non-monster, e.g. the weather.
///
/// `Rcv` is the ID of the receiver, that is, the monster that is reacting to the event. `Rcv` is
/// `Nothing`/`()` if the event is being received by a non-monster, e.g. the weather.
///
/// `Mch` is the ID of the mechanic on the receiver that is reacting to the event. This could be, for
/// example, the `AbilityID` of the ability (the mechanic) on the monster (the receiver).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventHandler<Ctx: EventContext, Rtn: EventReturnable, Mch: MechanicID, Rcv: Receiver, Bdc: Broadcaster = MonsterID> {
    pub response: EventResponse<Ctx, Rtn, Mch, Rcv, Bdc>,
    pub event_filtering_options: EventFilteringOptions,
}

/// `fn(battle: &mut BattleState, broadcaster_id: Bdc, receiver_id: Rcv, mechanic_id: Mch, context: Ctx, relay: Rtn) -> event_outcome: Rtn`
pub type EventResponse<Ctx, Rtn, Mch, Rcv, Bdc> = fn(&mut Battle, Bdc, Rcv, Mch, Ctx, Rtn) -> Rtn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventHandlerWithOwner<Ctx: EventContext, Rtn: EventReturnable, Mch: MechanicID, Rcv: Receiver, Bdc: Broadcaster> {
    pub event_handler: EventHandler<Ctx, Rtn, Mch, Rcv, Bdc>,
    pub receiver_id: Rcv,
    pub mechanic_id: Mch,
    pub mechanic_kind: MechanicKind,
    pub activation_order: ActivationOrder,
}

use dyn_clone::DynClone;

pub trait EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>: DynClone {
    fn respond(&self, battle: &mut Battle, broadcaster_id: Bdc, context: Ctx, default: Rtn) -> Rtn;

    fn activation_order(&self) -> ActivationOrder;

    fn owner_id(&self) -> Option<MonsterID>;

    fn event_filtering_options(&self) -> EventFilteringOptions;

    fn mechanic_kind(&self) -> MechanicKind;

    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &mut Battle) -> &'static str;
}

impl<Ctx: EventContext, Rtn: EventReturnable, Mch: MechanicID, Rcv: Receiver, Bdc: Broadcaster> EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>
    for EventHandlerWithOwner<Ctx, Rtn, Mch, Rcv, Bdc>
{
    fn respond(&self, battle: &mut Battle, broadcaster_id: Bdc, context: Ctx, default: Rtn) -> Rtn {
        #[cfg(feature = "debug")]
        println!["\t└── (EventHandler for {mechanic} activated)", mechanic = self.mechanic_name(battle)];
        (self.event_handler.response)(battle, broadcaster_id, self.receiver_id, self.mechanic_id, context, default)
    }

    fn activation_order(&self) -> ActivationOrder {
        self.activation_order
    }

    fn owner_id(&self) -> Option<MonsterID> {
        self.receiver_id.id()
    }

    fn event_filtering_options(&self) -> EventFilteringOptions {
        self.event_handler.event_filtering_options
    }

    fn mechanic_kind(&self) -> MechanicKind {
        self.mechanic_kind
    }

    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &mut Battle) -> &'static str {
        self.mechanic_id.mechanic_name(battle)
    }
}

impl<Rtn: EventReturnable, Ctx: EventContext, Bdc: Broadcaster> Clone for Box<dyn EventHandlerWithOwnerEmbedded<Ctx, Rtn, Bdc>> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

// EventFilteringOptions -------------------------------------------------- //

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

// Constraint Traits ------------------------------------------------------------ //

// Broadcasters ---------------

pub trait Broadcaster: Copy {
    fn as_id(&self) -> Option<MonsterID> {
        None
    }
}

impl Broadcaster for MonsterID {
    fn as_id(&self) -> Option<MonsterID> {
        Some(*self)
    }
}

impl Broadcaster for Nothing {}

pub trait EventContext: Copy {
    fn target(&self) -> Option<MonsterID> {
        None
    }
}

impl EventContext for Nothing {}

// Returnables -----------------

pub trait EventReturnable: PartialEq + Copy {}

impl<T: PartialEq + Copy> EventReturnable for T {}

pub trait Receiver: Copy {
    fn id(&self) -> Option<MonsterID> {
        None
    }
}

// Receivers -----------------

impl Receiver for Nothing {}

impl Receiver for MonsterID {
    fn id(&self) -> Option<MonsterID> {
        Some(*self)
    }
}

// Mechanics ------------------

pub trait MechanicID: Copy {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str;
}

impl MechanicID for Nothing {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, _: &Battle) -> &'static str {
        "None"
    }
}

impl MechanicID for AbilityID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str {
        battle.ability(*self).name()
    }
}

impl MechanicID for ItemID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str {
        battle.item(*self).map_or("None", |i| i.name())
    }
}

impl MechanicID for MonsterID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str {
        battle.monster(*self).name()
    }
}

impl MechanicID for MoveID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str {
        battle.move_(*self).name()
    }
}

impl MechanicID for PersistentStatusID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str {
        battle.persistent_status(*self).map_or("None", |ps| ps.name())
    }
}

impl MechanicID for TrapID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str {
        battle.trap(*self).map_or("None", |t| t.name())
    }
}

impl MechanicID for VolatileStatusID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &Battle) -> &'static str {
        battle.volatile_status(*self).map_or("None", |s| s.name())
    }
}
