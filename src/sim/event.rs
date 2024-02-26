use core::fmt::Debug;

use crate::sim::{game_mechanics::MonsterUID, Battle, Nothing, Outcome, Percent};
use broadcast_contexts::*;
use event_setup_macro::event_setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResolver;

/// `R`: indicates return type
///
/// `C`: indicates context specifier type
#[cfg(not(feature = "debug"))]
#[derive(Clone, Copy)]
pub struct EventResponder<R: Copy, C: Copy> {
    pub callback: fn(&mut Battle, C, R) -> R,
}

/// `R`: indicates return type
///
/// `C`: indicates context specifier type
#[cfg(feature = "debug")]
#[derive(Clone, Copy)]
pub struct EventResponder<R: Copy, C: Copy> {
    pub callback: fn(&mut Battle, C, R) -> R,
    pub dbg_location: &'static str,
}

pub type EventResponderWithLifeTime<'a, R, C> = fn(&'a mut Battle, C, R) -> R;

#[derive(Debug, Clone, Copy)]
pub struct EventResponderInstance<R: Copy, C: Copy> {
    pub event_name: &'static str,
    pub event_responder: EventResponder<R, C>,
    pub owner_uid: MonsterUID,
    pub activation_order: ActivationOrder,
    pub filters: EventFilterOptions,
}

pub type EventResponderInstanceList<R, C> = Vec<EventResponderInstance<R, C>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFilterOptions {
    pub event_source: TargetFlags,
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

pub mod broadcast_contexts {
    use crate::sim::{MonsterUID, MoveUID};

    #[derive(Debug, Clone, Copy)]
    pub struct MoveUsed {
        pub attacker_uid: MonsterUID,
        pub move_uid: MoveUID,
        pub target_uid: MonsterUID,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct AbilityUsed {
        pub ability_holder_uid: MonsterUID,
    }

    impl MoveUsed {
        pub fn new(move_uid: MoveUID, target_uid: MonsterUID) -> Self {
            Self {
                attacker_uid: move_uid.owner_uid,
                move_uid,
                target_uid,
            }
        }
    }

    impl AbilityUsed {
        pub fn new(ability_user_uid: MonsterUID) -> Self {
            Self {
                ability_holder_uid: ability_user_uid,
            }
        }
    }
}

event_setup![
    pub struct CompositeEventResponder {
        match event {
            /// Return value: `Outcome::Success` means the move succeeded.
            #[context(MoveUsed)]
            on_try_move => Outcome,

            #[context(MoveUsed)]
            on_damage_dealt => Nothing,

            /// Return value: `Outcome::Success` means ability activation succeeded.
            #[context(AbilityUsed)]
            on_try_activate_ability => Outcome,

            #[context(AbilityUsed)]
            on_ability_activated => Nothing,

            /// Return value: `Percent` value indicates percentage multiplier for
            /// accuracy modification.
            #[context(MoveUsed)]
            on_modify_accuracy => Percent,

            /// Return value: `Outcome::Success` means stat was successfully raised.
            #[context(None)]
            on_try_raise_stat => Outcome,

            /// Return value: `Outcome::Success` means stat was successfully lowered.
            #[context(None)]
            on_try_lower_stat => Outcome,

            #[context(MoveUsed)]
            on_status_move_used => Nothing,
        }
    }
    pub const DEFAULT_RESPONSE = None;
    pub trait InBattleEvent;
];

#[derive(Debug, Clone, Copy)]
pub struct CompositeEventResponderInstance {
    pub composite_event_responder: CompositeEventResponder,
    pub owner_uid: MonsterUID,
    pub activation_order: ActivationOrder,
    pub filters: EventFilterOptions,
}

pub type CompositeEventResponderInstanceList = Vec<CompositeEventResponderInstance>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationOrder {
    pub priority: u16,
    pub speed: u16,
    pub order: u16,
}

impl EventResolver {
    pub fn broadcast_trial_event<C: Copy>(
        battle: &mut Battle,
        broadcaster_uid: MonsterUID,
        calling_context: C,
        event: &dyn InBattleEvent<EventReturnType = Outcome, ContextType = C>,
    ) -> Outcome {
        Self::broadcast_event(battle, broadcaster_uid, calling_context, event, Outcome::Success, Some(Outcome::Failure))
    }

    /// `default` tells the resolver what value it should return if there are no event responders, or the event responders fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a responder in the chain, the resolution short-circuits and returns early.
    pub fn broadcast_event<R: PartialEq + Copy, C: Copy>(
        battle: &mut Battle,
        broadcaster_uid: MonsterUID,
        calling_context: C,
        event: &dyn InBattleEvent<EventReturnType = R, ContextType = C>,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        let composite_event_responder_instances: CompositeEventResponderInstanceList = battle.composite_event_responder_instances();
        let mut event_responder_instances: EventResponderInstanceList<R, C> =
            EventResolver::get_responders_to_event(composite_event_responder_instances, event);

        if event_responder_instances.is_empty() {
            return default;
        }

        crate::sim::ordering::sort_by_activation_order::<EventResponderInstance<R, C>>(&mut battle.prng, &mut event_responder_instances, &mut |it| {
            it.activation_order
        });

        let mut relay = default;
        for EventResponderInstance {
            event_responder,
            owner_uid,
            filters,
            ..
        } in event_responder_instances.into_iter()
        {
            if Self::filter_composite_event_responders(battle, broadcaster_uid, owner_uid, filters) {
                relay = (event_responder.callback)(battle, calling_context, relay);
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

    fn filter_composite_event_responders(
        battle: &Battle,
        event_caller_uid: MonsterUID,
        owner_uid: MonsterUID,
        composite_event_responder_filters: EventFilterOptions,
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
        let event_source_filter_passed = composite_event_responder_filters.event_source.bits() == bitmask;
        let is_active_passed = battle.is_active_monster(owner_uid);

        event_source_filter_passed && is_active_passed
    }

    fn get_responders_to_event<R: Copy, C: Copy>(
        composite_event_responder_instances: Vec<CompositeEventResponderInstance>,
        event: &(dyn InBattleEvent<EventReturnType = R, ContextType = C>),
    ) -> Vec<EventResponderInstance<R, C>> {
        composite_event_responder_instances
            .iter()
            .filter_map(|it| it.get_responder_to_event(event))
            .collect::<Vec<_>>()
    }
}

#[cfg(not(feature = "debug"))]
impl<'a, R: Copy, C: Copy> Debug for EventResponder<R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventResponder")
            .field("callback", &&(self.callback as EventResponderWithLifeTime<'a, R, C>))
            .finish()
    }
}

#[cfg(feature = "debug")]
impl<'a, R: Copy, C: Copy> Debug for EventResponder<R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventResponder")
            .field("callback", &&(self.callback as EventResponderWithLifeTime<'a, R, C>))
            .field("location", &self.dbg_location)
            .finish()
    }
}

impl CompositeEventResponderInstance {
    fn get_responder_to_event<R: Copy, C: Copy>(
        &self,
        event: &dyn InBattleEvent<EventReturnType = R, ContextType = C>,
    ) -> Option<EventResponderInstance<R, C>> {
        let event_responder = event.corresponding_responder(&self.composite_event_responder);
        event_responder.map(|event_responder| EventResponderInstance {
            event_name: event.name(),
            event_responder,
            owner_uid: self.owner_uid,
            activation_order: self.activation_order,
            filters: self.filters,
        })
    }
}

impl EventFilterOptions {
    pub const fn default() -> EventFilterOptions {
        EventFilterOptions {
            event_source: TargetFlags::OPPONENTS,
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
            let test_battle = build_battle!(
                {
                    Allies: MonsterTeam {
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
                    Opponents: MonsterTeam {
                        Drifblim: Monster {
                            Scratch: Move,
                            Ember: Move,
                            FlashFire: Ability,
                        },
                    }
                }
            );

            let mut prng = Prng::new(crate::sim::prng::seed_from_time_now());

            let composite_event_responder_instances = test_battle.composite_event_responder_instances();
            use crate::sim::event_dex::OnTryMove;
            let mut event_responder_instances = EventResolver::get_responders_to_event(composite_event_responder_instances, &OnTryMove);

            crate::sim::ordering::sort_by_activation_order(&mut prng, &mut event_responder_instances, &mut |it| it.activation_order);

            result[i] = event_responder_instances
                .into_iter()
                .map(|event_responder_instance| test_battle.monster(event_responder_instance.owner_uid).name())
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
            let test_battle = build_battle!(
                {
                    Allies: MonsterTeam {
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
                    Opponents: MonsterTeam {
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

            let composite_event_responder_instances = test_battle.composite_event_responder_instances();
            use crate::sim::event_dex::OnTryMove;

            let mut event_responder_instances = EventResolver::get_responders_to_event(composite_event_responder_instances, &OnTryMove);

            crate::sim::ordering::sort_by_activation_order(&mut prng, &mut event_responder_instances, &mut |it| it.activation_order);

            result[i] = event_responder_instances
                .into_iter()
                .map(|event_responder_instance| test_battle.monster(event_responder_instance.owner_uid).name())
                .collect::<Vec<_>>();
        }

        // Check that the two runs are not equal, there is an infinitesimal chance they won't be, but the probability is negligible.
        assert_ne!(result[0], result[1]);
        // Check that Drifblim is indeed the in the front.
        assert_eq!(result[0][0], "G");
        // Check that the Torchics are all in the middle.
        for name in ["A", "B", "C", "D", "E", "H", "I", "J", "K", "L"].iter() {
            assert!(result[0].contains(&name.to_string()));
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
            MonsterNumber, TeamID,
        };
        let test_battle = build_battle!(
            {
                Allies: MonsterTeam {
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
                Opponents: MonsterTeam {
                    Treecko: Monster = "Emerald" {
                        Scratch: Move,
                        Ember: Move,
                        FlashFire: Ability,
                    },
                }
            }
        );

        let passed_filter = EventResolver::filter_composite_event_responders(
            &test_battle,
            MonsterUID {
                team_id: TeamID::Allies,
                monster_number: MonsterNumber::_1,
            },
            MonsterUID {
                team_id: TeamID::Opponents,
                monster_number: MonsterNumber::_1,
            },
            EventFilterOptions::default(),
        );
        assert!(passed_filter);
    }

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_responder_instance() {
        use crate::sim::ability_dex::FlashFire;
        let event_responder_instance = EventResponderInstance {
            event_name: event_dex::OnTryMove.name(),
            event_responder: FlashFire.composite_event_responder.on_try_move.unwrap(),
            owner_uid: MonsterUID {
                team_id: crate::sim::TeamID::Allies,
                monster_number: crate::sim::MonsterNumber::_1,
            },
            activation_order: crate::sim::ActivationOrder {
                priority: 1,
                speed: 99,
                order: 0,
            },
            filters: crate::sim::EventFilterOptions::default(),
        };
        println!("{:#?}", event_responder_instance);
    }
}
