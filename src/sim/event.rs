use core::fmt::Debug;

use crate::sim::prng::Prng;

use super::{game_mechanics::BattlerUID, BattleContext};

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
                &&(self.callback as EventHandlerWithLifeTime<'a, R>),
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
                &&(self.callback as EventHandlerWithLifeTime<'a, R>),
            )
            .field("location", &self.dbg_location)
            .finish()
    }
}

pub type EventHandlerWithLifeTime<'a, R> =
    fn(&'a mut BattleContext, &'a mut Prng, BattlerUID, R) -> EventReturn<R>;

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerSetInstance {
    pub event_handler_set: EventHandlerSet,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters,
}

pub type EventHandlerSetInstanceList = Vec<EventHandlerSetInstance>;

#[test]
#[cfg(feature = "debug")]
fn test_print_event_handler_instance() {
    use crate::sim::ability_dex::FlashFire;
    let event_handler_instance = EventHandlerInstance {
        event_name: event_dex::OnTryMove.name(),
        event_handler: FlashFire.event_handlers.on_try_move.unwrap(),
        owner_uid: BattlerUID {
            team_id: crate::sim::TeamID::Ally,
            battler_number: crate::sim::BattlerNumber::_1,
        },
        activation_order: crate::sim::ActivationOrder {
            priority: 1,
            speed: 99,
            order: 0,
        },
        filters: crate::sim::EventHandlerFilters::default(),
    };
    println!("{:#?}", event_handler_instance);
}

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerInstance<R: Clone + Copy> {
    pub event_name: &'static str,
    pub event_handler: EventHandler<R>,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventHandlerFilters,
}

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
        let mut event_handler_instances: EventHandlerInstanceList<R> = event_handlers_set_instances
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

        crate::sim::ordering::sort_by_activation_order::<EventHandlerInstance<R>>(
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

#[test]
#[cfg(feature = "debug")]
fn test_if_priority_sorting_is_deterministic() {
    extern crate self as monsim;
    use crate::sim::{
        ability_dex::FlashFire,
        battle_context,
        monster_dex::{Drifblim, Mudkip, Torchic, Treecko},
        move_dex::{Bubble, Ember, Scratch, Tackle},
    };
    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let test_bcontext = battle_context!(
            {
                AllyTeam {
                    mon Torchic "Ruby" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Mudkip "Sapphire" {
                        mov Tackle,
                        mov Bubble,
                        abl FlashFire,
                    },
                    mon Treecko "Emerald" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                },
                OpponentTeam {
                    mon Drifblim {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                }
            }
        );

        let mut prng = Prng::new(crate::sim::prng::seed_from_time_now());

        let event_handler_set_instances = test_bcontext.event_handler_set_instances();
        use crate::sim::{event_dex::OnTryMove, InBattleEvent};
        let mut event_handler_instances = event_handler_set_instances
            .iter()
            .filter_map(|event_handler_set_instance| {
                if let Some(handler) =
                    OnTryMove.corresponding_handler(&event_handler_set_instance.event_handler_set)
                {
                    Some(EventHandlerInstance {
                        event_name: OnTryMove.name(),
                        event_handler: handler,
                        owner_uid: event_handler_set_instance.owner_uid,
                        activation_order: event_handler_set_instance.activation_order,
                        filters: EventHandlerFilters::default(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        crate::sim::ordering::sort_by_activation_order::<EventHandlerInstance<bool>>(
            &mut prng,
            &mut event_handler_instances,
            &mut |it| it.activation_order,
        );

        result[i] = event_handler_instances
            .into_iter()
            .map(|event_handler_instance| {
                test_bcontext
                    .monster(event_handler_instance.owner_uid)
                    .nickname
            })
            .collect::<Vec<_>>();
    }

    assert_eq!(result[0], result[1]);
    assert_eq!(result[0][0], "Drifblim");
    assert_eq!(result[0][1], "Emerald");
    assert_eq!(result[0][2], "Ruby");
    assert_eq!(result[0][3], "Sapphire");
}

#[test]
#[cfg(feature = "debug")]
fn test_priority_sorting_with_speed_ties() {
    extern crate self as monsim;
    use crate::sim::{
        ability_dex::FlashFire,
        battle_context,
        monster_dex::{Drifblim, Mudkip, Torchic},
        move_dex::{Bubble, Ember, Scratch, Tackle},
    };
    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let test_bcontext = battle_context!(
            {
                AllyTeam {
                    mon Torchic "A" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "B" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "C" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "D" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "E" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Mudkip "F" {
                        mov Tackle,
                        mov Bubble,
                        abl FlashFire,
                    }
                },
                OpponentTeam {
                    mon Drifblim "G" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "H" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "I" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "J" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "K" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "L" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                }
            }
        );
        let mut prng = Prng::new(i as u64);

        let event_handler_set_instances = test_bcontext.event_handler_set_instances();
        use crate::sim::{event_dex::OnTryMove, InBattleEvent};
        let mut event_handler_instances = event_handler_set_instances
            .iter()
            .filter_map(|event_handler_set_instance| {
                if let Some(handler) =
                    OnTryMove.corresponding_handler(&event_handler_set_instance.event_handler_set)
                {
                    Some(EventHandlerInstance {
                        event_name: OnTryMove.name(),
                        event_handler: handler,
                        owner_uid: event_handler_set_instance.owner_uid,
                        activation_order: event_handler_set_instance.activation_order,
                        filters: EventHandlerFilters::default(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        crate::sim::ordering::sort_by_activation_order::<EventHandlerInstance<bool>>(
            &mut prng,
            &mut event_handler_instances,
            &mut |it| it.activation_order,
        );

        result[i] = event_handler_instances
            .into_iter()
            .map(|event_handler_instance| {
                test_bcontext
                    .monster(event_handler_instance.owner_uid)
                    .nickname
            })
            .collect::<Vec<_>>();
    }

    // Check that the two runs are not equal, there is an infinitesimal chance they won't be, but the probability is negligible.
    assert_ne!(result[0], result[1]);
    // Check that Drifblim is indeed the in the front.
    assert_eq!(result[0][0], "G");
    // Check that the Torchics are all in the middle.
    for name in ["A", "B", "C", "D", "E", "H", "I", "J", "K", "L"].iter() {
        assert!(result[0].contains(name));
    }
    //Check that the Mudkip is last.
    assert_eq!(result[0][11], "F");
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
    pub const fn default() -> EventHandlerFilters {
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
