use core::fmt::Debug;

use crate::sim::{game_mechanics::BattlerUID, prng::Prng, Battle};
use event_setup_macro::event_setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResolver;

#[allow(non_camel_case_types)]
type void = ();

#[cfg(not(feature = "debug"))]
#[derive(Clone, Copy)]
pub struct SpecificEventResponder<R: Clone + Copy> {
    pub callback: fn(&mut Battle, &mut Prng, BattlerUID, R) -> R,
}

#[cfg(not(feature = "debug"))]
impl<'a, R: Clone + Copy> Debug for SpecificEventResponder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventResponder")
            .field(
                "callback",
                &&(self.callback as SpecificEventResponderWithLifeTime<'a, R>),
            )
            .finish()
    }
}

#[cfg(feature = "debug")]
#[derive(Clone, Copy)]
pub struct SpecificEventResponder<R: Clone + Copy> {
    pub callback: fn(&mut Battle, &mut Prng, BattlerUID, R) -> R,
    #[cfg(feature = "debug")]
    pub dbg_location: &'static str,
}

pub type SpecificEventResponderWithLifeTime<'a, R> =
    fn(&'a mut Battle, &'a mut Prng, BattlerUID, R) -> R;

#[derive(Debug, Clone, Copy)]
pub struct SpecificEventResponderInstance<R: Clone + Copy> {
    pub event_name: &'static str,
    pub specific_event_responder: SpecificEventResponder<R>,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventResponderFilters,
}

pub type SpecificEventResponderInstanceList<R> = Vec<SpecificEventResponderInstance<R>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResponderFilters {
    pub whose_event: TargetFlags,
    pub on_battlefield: bool,
}

bitflags::bitflags! {
    pub struct TargetFlags: u8 {
        const SELF = 0b0001;
        const ALLIES = 0b0010;
        const OPPONENTS = 0b0100;
        const ENVIRONMENT = 0b1000;
    }
}

event_setup![
    pub struct EventResponder {
        match event {
            on_try_move => bool,
            on_damage_dealt => void,
            on_try_activate_ability => bool,
            on_ability_activated => void,
            on_modify_accuracy => u16,
            on_try_raise_stat => bool,
            on_try_lower_stat => bool,
            on_status_move_used => void,
        }
    }
    pub const DEFAULT_RESPONSE = None;
    pub trait InBattleEvent;
];

#[derive(Debug, Clone, Copy)]
pub struct EventResponderInstance {
    pub event_responder: EventResponder,
    pub owner_uid: BattlerUID,
    pub activation_order: ActivationOrder,
    pub filters: EventResponderFilters,
}

pub type EventResponderInstanceList = Vec<EventResponderInstance>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationOrder {
    pub priority: u16,
    pub speed: u16,
    pub order: u16,
}

impl EventResolver {
    pub fn broadcast_trial_event(
        battle: &mut Battle,
        prng: &mut Prng,
        caller_uid: BattlerUID,
        event: &dyn InBattleEvent<EventReturnType = bool>,
    ) -> bool {
        Self::broadcast_event(battle, prng, caller_uid, event, true, Some(false))
    }

    /// `default` tells the resolver what value it should return if there are no event responders, or the event responders fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a responder in the chain, the resolution short-circuits and returns early.
    pub fn broadcast_event<R: PartialEq + Copy>(
        battle: &mut Battle,
        prng: &mut Prng,
        caller_uid: BattlerUID,
        event: &dyn InBattleEvent<EventReturnType = R>,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        let event_responder_instances: EventResponderInstanceList =
            battle.event_responder_instances();
        let mut specific_event_responder_instances: SpecificEventResponderInstanceList<R> =
            extract_specific_event_responder_instances(event_responder_instances, event);

        if specific_event_responder_instances.is_empty() {
            return default;
        }

        crate::sim::ordering::sort_by_activation_order::<SpecificEventResponderInstance<R>>(
            prng,
            &mut specific_event_responder_instances,
            &mut |it| it.activation_order,
        );

        let mut relay = default;
        for SpecificEventResponderInstance {
            specific_event_responder,
            owner_uid,
            filters,
            ..
        } in specific_event_responder_instances.into_iter()
        {
            if Self::filter_event_responders(battle, caller_uid, owner_uid, filters) {
                relay = (specific_event_responder.callback)(battle, prng, owner_uid, relay);
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

    fn filter_event_responders(
        battle: &Battle,
        event_caller_uid: BattlerUID,
        owner_uid: BattlerUID,
        event_responder_filters: EventResponderFilters,
    ) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if event_caller_uid == owner_uid {
                bitmask |= TargetFlags::SELF.bits()
            } // 0x01
            if battle.are_allies(owner_uid, event_caller_uid) {
                bitmask |= TargetFlags::ALLIES.bits()
            } // 0x02
            if battle.are_opponents(owner_uid, event_caller_uid) {
                bitmask |= TargetFlags::OPPONENTS.bits()
            } //0x04
              // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = event_responder_filters.whose_event.bits() == bitmask;
        let on_battlefield_passed = battle.is_battler_on_field(owner_uid);

        event_source_filter_passed && on_battlefield_passed
    }
}

fn extract_specific_event_responder_instances<R: Copy>(
    event_responder_instances: Vec<EventResponderInstance>,
    event: &(dyn InBattleEvent<EventReturnType = R>),
) -> Vec<SpecificEventResponderInstance<R>> {
    event_responder_instances
        .iter()
        .filter_map(|event_responder_instance| {
            event
                .corresponding_responder(&event_responder_instance.event_responder)
                .map(|specific_event_responder| SpecificEventResponderInstance {
                    event_name: event.name(),
                    specific_event_responder,
                    owner_uid: event_responder_instance.owner_uid,
                    activation_order: event_responder_instance.activation_order,
                    filters: EventResponderFilters::default(),
                })
        })
        .collect::<Vec<_>>()
}

#[cfg(feature = "debug")]
impl<'a, R: Clone + Copy> Debug for SpecificEventResponder<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpecificEventResponder")
            .field(
                "callback",
                &&(self.callback as SpecificEventResponderWithLifeTime<'a, R>),
            )
            .field("location", &self.dbg_location)
            .finish()
    }
}

impl EventResponderFilters {
    pub const fn default() -> EventResponderFilters {
        EventResponderFilters {
            whose_event: TargetFlags::OPPONENTS,
            on_battlefield: true,
        }
    }
}

#[cfg(all(test, feature = "debug"))]
mod tests {
    use super::*;
    use crate::sim::build_battle;

    #[test]
    fn test_if_priority_sorting_is_deterministic() {
        extern crate self as monsim;
        use crate::sim::*;
        use crate::sim::{
            ability_dex::FlashFire,
            monster_dex::{Drifblim, Mudkip, Torchic, Treecko},
            move_dex::{Bubble, Ember, Scratch, Tackle},
        };
        let mut result = [Vec::new(), Vec::new()];
        for i in 0..=1 {
            let test_bcontext = build_battle!(
                {
                    Allies: BattlerTeam {
                        Torchic: Monster = "Ruby" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Mudkip: Monster = "Sapphire" {
                            Tackle: Move,
                            Bubble: Move,
                            FlashFire: Ability,
                        },
                        Treecko: Monster = "Emerald" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                    },
                    Opponents: BattlerTeam {
                        Drifblim: Monster {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                    }
                }
            );

            let mut prng = Prng::new(crate::sim::prng::seed_from_time_now());

            let event_responder_instances = test_bcontext.event_responder_instances();
            use crate::sim::event_dex::OnTryMove;
            let mut specific_event_responder_instances = event_responder_instances
                .iter()
                .filter_map(|event_responder_instance| {
                    if let Some(responder) =
                        OnTryMove.corresponding_responder(&event_responder_instance.event_responder)
                    {
                        Some(SpecificEventResponderInstance {
                            event_name: OnTryMove.name(),
                            specific_event_responder: responder,
                            owner_uid: event_responder_instance.owner_uid,
                            activation_order: event_responder_instance.activation_order,
                            filters: EventResponderFilters::default(),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            crate::sim::ordering::sort_by_activation_order::<SpecificEventResponderInstance<bool>>(
                &mut prng,
                &mut specific_event_responder_instances,
                &mut |it| it.activation_order,
            );

            result[i] = specific_event_responder_instances
                .into_iter()
                .map(|specific_event_responder_instance| {
                    test_bcontext
                        .monster(specific_event_responder_instance.owner_uid)
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
        #[cfg(feature = "debug")]
        extern crate self as monsim;
        use crate::sim::*;
        use crate::sim::{
            ability_dex::FlashFire,
            monster_dex::{Drifblim, Mudkip, Torchic},
            move_dex::{Ember, Scratch},
        };
        let mut result = [Vec::new(), Vec::new()];
        for i in 0..=1 {
            let test_bcontext = build_battle!(
                {
                    Allies: BattlerTeam {
                        Torchic: Monster = "A" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "B" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "C" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "D" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "E" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Mudkip: Monster = "F" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        }
                    },
                    Opponents: BattlerTeam {
                        Drifblim: Monster = "G" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "H" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "I" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "J" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "K" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                        Torchic: Monster = "L" {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                    }
                }
            );
            let mut prng = Prng::new(i as u64);

            let event_responder_instances = test_bcontext.event_responder_instances();
            use crate::sim::{event_dex::OnTryMove, InBattleEvent};
            let mut specific_event_responder_instances = event_responder_instances
                .iter()
                .filter_map(|event_responder_instance| {
                    if let Some(responder) =
                        OnTryMove.corresponding_responder(&event_responder_instance.event_responder)
                    {
                        Some(SpecificEventResponderInstance {
                            event_name: OnTryMove.name(),
                            specific_event_responder: responder,
                            owner_uid: event_responder_instance.owner_uid,
                            activation_order: event_responder_instance.activation_order,
                            filters: EventResponderFilters::default(),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            crate::sim::ordering::sort_by_activation_order::<SpecificEventResponderInstance<bool>>(
                &mut prng,
                &mut specific_event_responder_instances,
                &mut |it| it.activation_order,
            );

            result[i] = specific_event_responder_instances
                .into_iter()
                .map(|specific_event_responder_instance| {
                    test_bcontext
                        .monster(specific_event_responder_instance.owner_uid)
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

    #[test]
    #[cfg(feature = "debug")]
    fn test_event_filtering_for_event_sources() {
        extern crate self as monsim;
        use crate::sim::*;
        use crate::sim::{
            ability_dex::FlashFire,
            monster_dex::{Mudkip, Torchic, Treecko},
            move_dex::{Bubble, Ember, Scratch, Tackle},
            BattlerNumber, TeamID,
        };
        let test_battle_context = build_battle!(
            {
                Allies: BattlerTeam {
                    Torchic: Monster = "Ruby" {
                        Ember: Move,
                        Scratch: Move,
                        FlashFire: Ability,
                    },
                    Mudkip: Monster = "Sapphire" {
                        Tackle: Move,
                        Bubble: Move,
                        FlashFire: Ability,
                    },
                },
                Opponents: BattlerTeam {
                    Treecko: Monster = "Emerald" {
                        Scratch: Move,
                        Ember: Move,
                        FlashFire: Ability,
                    },
                }
            }
        );

        let passed_filter = EventResolver::filter_event_responders(
            &test_battle_context,
            BattlerUID {
                team_id: TeamID::Allies,
                battler_number: BattlerNumber::_1,
            },
            BattlerUID {
                team_id: TeamID::Opponents,
                battler_number: BattlerNumber::_1,
            },
            EventResponderFilters::default(),
        );
        assert!(passed_filter);
    }

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_specific_event_responder_instance() {
        use crate::sim::ability_dex::FlashFire;
        let specific_event_responder_instance = SpecificEventResponderInstance {
            event_name: event_dex::OnTryMove.name(),
            specific_event_responder: FlashFire.event_responder.on_try_move.unwrap(),
            owner_uid: BattlerUID {
                team_id: crate::sim::TeamID::Allies,
                battler_number: crate::sim::BattlerNumber::_1,
            },
            activation_order: crate::sim::ActivationOrder {
                priority: 1,
                speed: 99,
                order: 0,
            },
            filters: crate::sim::EventResponderFilters::default(),
        };
        println!("{:#?}", specific_event_responder_instance);
    }
}
