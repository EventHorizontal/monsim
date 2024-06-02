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

use super::*;
use crate::sim::ActivationOrder;
pub use event_dex::*;

pub type EventResponse<R, C, B> =  fn(/* simulator */ &mut BattleSimulator, /* broadcaster_id */ B, /* receiver_id */ MonsterID, /* context */ C, /* relay */ R) -> R;

/// Stores an `Effect` that gets simulated in response to an `Event` being triggered.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EventHandler<R: Copy, C: Copy, B: Broadcaster + Clone + Copy> {
    #[cfg(feature = "debug")]
    pub source_code_location: &'static str,
    pub response: EventResponse<R, C, B>,
    pub event_filtering_options: EventFilteringOptions,
}

pub trait Broadcaster {
    fn is_sourced(&self) -> Option<MonsterID>;
}

impl Broadcaster for MonsterID {
    fn is_sourced(&self) -> Option<MonsterID> {
        Some(*self)
    }
}

impl Broadcaster for Nothing {
    fn is_sourced(&self) -> Option<MonsterID> {
        None
    }
}

impl<R: Copy, C: Copy, B: Broadcaster + Copy> Debug for EventHandler<R, C, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "debug")]
        let out = {
            f.debug_struct("EventHandler")
                .field("source_code_location", &self.source_code_location)
                .finish()
        };
        #[cfg(not(feature = "debug"))]
        let out = {
            write!(f, "EventHandler debug information only available with feature flag \"debug\" turned on.")
        };
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedEventHandler<R: Copy, C: Copy, B: Broadcaster + Copy> {
    pub event_handler: EventHandler<R, C, B>,
    pub owner_id: MonsterID,
    pub activation_order: ActivationOrder,
}

impl EventHandlerDeck {
    pub const fn empty() -> Self {
        DEFAULT_EVENT_HANDLERS
    }
}

pub mod contexts {
    use monsim_utils::MaxSizedVec;

    use crate::{sim::{MonsterID, MoveID}, AbilityID, ItemID};

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
    }

    impl MoveHitContext {
        pub fn new(move_used_id: MoveID, target_id: MonsterID) -> Self {
            Self {
                move_user_id: move_used_id.owner_id,
                move_used_id,
                target_id,
            }
        }
    }

    /// `ability_owner_id`: MonsterID of the Monster whose ability is being used.
    /// 
    /// `ability_used_id`: AbilityID of the Ability being used.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AbilityUseContext {
        pub ability_owner_id: MonsterID,
        pub ability_used_id: AbilityID,
    }

    impl AbilityUseContext {
        pub fn new(ability_owner: MonsterID) -> Self {
            Self {
                ability_used_id: AbilityID { owner_id: ability_owner },
                ability_owner_id: ability_owner,
            }
        }
    }

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
            Self {
                item_id,
                item_holder_id,
            }
        }
    }
}

mod event_dex {
    use super::*;
    use monsim_utils::NOTHING;

    #[derive(Debug, Clone, Copy)]
    pub struct EventHandlerDeck {
        pub on_try_move: Option<EventHandler<Outcome<Nothing>, MoveUseContext, MonsterID>>,
        pub on_move_used: Option<EventHandler<Nothing, MoveUseContext, MonsterID>>,
        pub on_damaging_move_used: Option<EventHandler<Nothing, MoveUseContext, MonsterID>>,
        pub on_status_move_used: Option<EventHandler<Nothing, MoveUseContext, MonsterID>>,
        pub on_try_move_hit: Option<EventHandler<Outcome<Nothing>, MoveHitContext, MonsterID>>,
        pub on_move_hit: Option<EventHandler<Nothing, MoveHitContext, MonsterID>>,
        pub on_calculate_attack_stat: Option<EventHandler<u16, MoveHitContext, MonsterID>>,
        pub on_calculate_defense_stat: Option<EventHandler<u16, MoveHitContext, MonsterID>>,
        pub on_modify_damage: Option<EventHandler<u16, MoveHitContext, MonsterID>>,
        pub on_damage_dealt: Option<EventHandler<Nothing, Nothing, MonsterID>>,
        pub on_try_activate_ability: Option<EventHandler<Outcome<Nothing>, AbilityUseContext, MonsterID>>,
        pub on_ability_activated: Option<EventHandler<Nothing, AbilityUseContext, MonsterID>>,
        pub on_modify_accuracy: Option<EventHandler<Percent, MoveUseContext, MonsterID>>,
        pub on_try_raise_stat: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
        pub on_try_lower_stat: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
        pub on_try_add_volatile_status: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
        pub on_try_add_permanent_status: Option<EventHandler<Outcome<Nothing>, Nothing, MonsterID>>,
        pub on_try_use_held_item: Option<EventHandler<Outcome<Nothing>, ItemUseContext, MonsterID>>,
        pub on_held_item_used: Option<EventHandler<Nothing, ItemUseContext, MonsterID>>,
        pub on_turn_end: Option<EventHandler<Nothing, Nothing, Nothing>>,
    }
    
    pub(super) const DEFAULT_EVENT_HANDLERS: EventHandlerDeck = EventHandlerDeck {
        on_try_move: None,
        on_move_used: None,
        on_damaging_move_used: None,
        on_status_move_used: None,
        on_try_move_hit: None,
        on_move_hit: None,
        on_calculate_attack_stat: None,
        on_calculate_defense_stat: None,
        on_modify_damage: None,
        on_damage_dealt: None,
        on_try_activate_ability: None,
        on_ability_activated: None,
        on_modify_accuracy: None,
        on_try_raise_stat: None,
        on_try_lower_stat: None,
        on_try_add_volatile_status: None,
        on_try_add_permanent_status: None,
        on_try_use_held_item: None,
        on_held_item_used: None,
        on_turn_end: None,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EventID {
        OnTryMove,
        OnMoveUsed,
        OnDamagingMoveUsed,
        OnStatusMoveUsed,
        OnTryMoveHit,
        OnMoveHit,
        OnCalculateAttackStat,
        OnCalculateDefenseStat,
        OnModifyDamage,
        OnDamageDealt,
        OnTryActivateAbility,
        OnAbilityActivated,
        OnModifyAccuracy,
        OnTryRaiseStat,
        OnTryLowerStat,
        OnTryAddVolatileStatus,
        OnTryAddPermanentStatus,
        OnTryUseHeldItem,
        OnHeldItemUsed,
        OnTurnEnd,
    }

    pub(crate) fn trigger_on_try_move_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_move)]
            },
            event_context,
        )
    }

    pub(crate) fn trigger_on_move_used_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_move_used)]
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_on_damaging_move_used_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_damaging_move_used), (event_handler_deck.on_move_used)]
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_on_status_move_used_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_status_move_used), (event_handler_deck.on_move_used)]
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_on_try_move_hit_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_move_hit)]
            },
            event_context,
        )
    }

    pub(crate) fn trigger_on_move_hit_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveHitContext) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_move_hit)]
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_on_calculate_attack_stat_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_calculate_attack_stat)]
            },
            event_context,
            default,
            None,
        )
    }

    pub(crate) fn trigger_on_calculate_defense_stat_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveHitContext, default: u16) -> u16 {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_calculate_defense_stat)]
            },
            event_context,
            default,
            None,
        )
    }

    pub(crate) fn trigger_on_modify_damage_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveHitContext, current_damage: u16) -> u16 {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_modify_damage)]
            },
            event_context,
            current_damage,
            None,
        )
    }

    pub(crate) fn trigger_on_damage_dealt_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: Nothing) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_damage_dealt)]
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_on_try_activate_ability_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: AbilityUseContext) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_activate_ability)]
            },
            event_context,
        )
    }

    pub(crate) fn trigger_on_ability_activated_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: AbilityUseContext) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_ability_activated)]
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_on_modify_accuracy_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: MoveUseContext) -> Percent {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_modify_accuracy)]
            },
            event_context,
            Percent(100),
            None,
        )
    }

    pub(crate) fn trigger_on_try_raise_stat_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_raise_stat)]
            },
            event_context,
        )
    }

    pub(crate) fn trigger_on_try_lower_stat_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_lower_stat)]
            },
            event_context,
        )
    }

    pub(crate) fn trigger_on_try_add_volatile_status_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_add_volatile_status)]
            },
            event_context,
        )
    }

    pub(crate) fn trigger_on_try_add_permanent_status_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: Nothing) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_add_permanent_status)]
            },
            event_context,
        )
    }
    
    pub(crate) fn trigger_on_try_use_held_item_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Outcome<Nothing> {
        EventDispatcher::dispatch_trial_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_try_use_held_item)]
            },
            event_context,
        )
    }

    pub(crate) fn trigger_on_held_item_used_event(sim: &mut BattleSimulator, broadcaster_id: MonsterID, event_context: ItemUseContext) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_held_item_used)]
            },
            event_context,
            NOTHING,
            None,
        )
    }

    pub(crate) fn trigger_on_turn_end_event(sim: &mut BattleSimulator, broadcaster_id: Nothing, event_context: Nothing) -> Nothing {
        EventDispatcher::dispatch_event(
            sim,
            broadcaster_id,
            |event_handler_deck| {
                vec![(event_handler_deck.on_turn_end)]
            },
            event_context,
            NOTHING,
            None,
        )
    }
}
