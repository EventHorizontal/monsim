use core::fmt::Debug;

use crate::sim::{game_mechanics::MonsterUID, ordering::sort_by_activation_order, Battle, Nothing, Outcome, Percent};
use contexts::*;
use event_setup_macro::event_setup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventDispatcher;

type EventCallback<R, C> = fn(&mut Battle, C, R) -> R;
#[cfg(feature = "debug")]
type EventCallbackWithLifetime<'a, R, C> = fn(&'a mut Battle, C, R) -> R;

/// `R`: indicates return type
///
/// `C`: indicates context specifier type
#[derive(Clone, Copy)]
pub struct EventHandler<R: Copy, C: Copy> {
    pub callback: EventCallback<R, C>,
    #[cfg(feature = "debug")]
    pub debugging_information: &'static str,
}


#[derive(Debug, Clone, Copy)]
pub struct OwnedEventHandler<R: Copy, C: Copy> {
    pub event_name: &'static str,
    pub event_handler: EventHandler<R, C>,
    pub owner_uid: MonsterUID,
    pub activation_order: ActivationOrder,
    pub filtering_options: EventFilteringOptions,
}

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

pub mod contexts {
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
    /// A "deck" is meant to be a collection with 0-1 of each "card".
    pub struct EventHandlerDeck {
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
    const DEFAULT_DECK = None;
    pub trait InBattleEvent;
];

#[derive(Debug, Clone, Copy)]
pub struct OwnedEventHandlerDeck {
    pub event_handler_deck: EventHandlerDeck,
    pub owner_uid: MonsterUID,
    pub activation_order: ActivationOrder,
    pub filtering_options: EventFilteringOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationOrder {
    pub priority: u16,
    pub speed: u16,
    pub order: u16,
}

impl EventDispatcher {

    pub fn dispatch_trial_event<C: Copy>(
        battle: &mut Battle,
        broadcaster_uid: MonsterUID,
        calling_context: C,
        event: impl InBattleEvent<EventReturnType = Outcome, ContextType = C>,
    ) -> Outcome {
        Self::dispatch_event(battle, broadcaster_uid, calling_context, event, Outcome::Success, Some(Outcome::Failure))
    }

    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn dispatch_event<R: PartialEq + Copy, C: Copy>(
        battle: &mut Battle,
        broadcaster_uid: MonsterUID,
        calling_context: C,
        event: impl InBattleEvent<EventReturnType = R, ContextType = C>,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        let mut event_handler_instances = Self::handlers_for_event(battle.event_handler_deck_instances(), event);

        if event_handler_instances.is_empty() {
            return default;
        }
 
        sort_by_activation_order::<OwnedEventHandler<R, C>>(&mut battle.prng, &mut event_handler_instances, &mut |it| {
            it.activation_order
        });

        let mut relay = default;
        for OwnedEventHandler {
            event_handler,
            owner_uid,
            filtering_options: filter_options,
            ..
        } in event_handler_instances.into_iter()
        {
            if Self::filter_event_handlers(battle, broadcaster_uid, owner_uid, filter_options) {
                relay = (event_handler.callback)(battle, calling_context, relay);
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
        battle: &Battle,
        broadcaster_uid: MonsterUID,
        owner_uid: MonsterUID,
        filter_options: EventFilteringOptions,
    ) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if broadcaster_uid == owner_uid {
                bitmask |= TargetFlags::SELF.bits()
            } // 0x01
            if battle.are_allies(owner_uid, broadcaster_uid) {
                bitmask |= TargetFlags::ALLIES.bits()
            } // 0x02
            if battle.are_opponents(owner_uid, broadcaster_uid) {
                bitmask |= TargetFlags::OPPONENTS.bits()
            } //0x04
              // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = filter_options.event_source.bits() == bitmask;
        let is_active_passed = battle.is_active_monster(owner_uid);

        event_source_filter_passed && is_active_passed
    }

    fn handlers_for_event<R: Copy, C: Copy>(
        event_handler_deck_instances: Vec<OwnedEventHandlerDeck>,
        event: impl InBattleEvent<EventReturnType = R, ContextType = C>,
    ) -> Vec<OwnedEventHandler<R, C>> {
        event_handler_deck_instances
            .iter()
            .filter_map(|it| it.handler_for_event(event))
            .collect::<Vec<_>>()
    }
}

impl<'a, R: Copy, C: Copy> Debug for EventHandler<R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "debug")]
        let out = {
            f.debug_struct("EventHandler")
                .field("callback", &&(self.callback as EventCallbackWithLifetime<'a, R, C>))
                .field("location", &self.debugging_information)
                .finish()
        };
        #[cfg(not(feature = "debug"))]
        let out = {
            write!(f, "EventHandler debug information only available with feature flag \"debug\" turned on.")
        };
        out
    }
}

impl EventHandlerDeck {
    pub const fn default() -> Self {
        DEFAULT_DECK
    }
}

impl OwnedEventHandlerDeck {
    fn handler_for_event<R: Copy, C: Copy>(
        &self,
        event: impl InBattleEvent<EventReturnType = R, ContextType = C>,
    ) -> Option<OwnedEventHandler<R, C>> {
        let event_handler = event.corresponding_handler(&self.event_handler_deck);
        event_handler.map(|event_handler| OwnedEventHandler {
            // INFO: Trait methods are non-const so we can only add the `event_name` during runtime.
            event_name: event.name(),
            event_handler,
            owner_uid: self.owner_uid,
            activation_order: self.activation_order,
            // TODO: Think about wether we want filtering options per handler, per deck or per mechanic
            filtering_options: self.filtering_options,
        })
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

#[cfg(all(test, feature = "debug"))]
mod tests {
    use super::*;
    use crate::sim::build_battle;

    #[test]
    fn test_if_priority_sorting_is_deterministic() {
        extern crate self as monsim;
        use crate::sim::*;
        use crate::sim::{
            test_ability_dex::FlashFire,
            test_monster_dex::{Drifblim, Mudkip, Torchic, Treecko},
            test_move_dex::{Bubble, Ember, Scratch, Tackle},
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

            let event_handler_deck_instances = test_battle.event_handler_deck_instances();
            use crate::sim::event_dex::OnTryMove;
            let mut event_handler_instances = EventDispatcher::handlers_for_event(event_handler_deck_instances, OnTryMove);

            crate::sim::ordering::sort_by_activation_order(&mut prng, &mut event_handler_instances, &mut |it| it.activation_order);

            result[i] = event_handler_instances
                .into_iter()
                .map(|event_handler_instance| test_battle.monster(event_handler_instance.owner_uid).name())
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
            test_ability_dex::FlashFire,
            test_monster_dex::{Drifblim, Mudkip, Torchic},
            test_move_dex::{Ember, Scratch},
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

            let event_handler_deck_instances = test_battle.event_handler_deck_instances();
            use crate::sim::event_dex::OnTryMove;

            let mut event_handler_instances = EventDispatcher::handlers_for_event(event_handler_deck_instances, OnTryMove);

            crate::sim::ordering::sort_by_activation_order(&mut prng, &mut event_handler_instances, &mut |it| it.activation_order);

            result[i] = event_handler_instances
                .into_iter()
                .map(|event_handler_instance| test_battle.monster(event_handler_instance.owner_uid).name())
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
            test_ability_dex::FlashFire,
            test_monster_dex::{Mudkip, Torchic, Treecko},
            test_move_dex::{Bubble, Ember, Scratch, Tackle},
            MonsterNumber, TeamUID,
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

        let passed_filter = EventDispatcher::filter_event_handlers(
            &test_battle,
            MonsterUID {
                team_uid: TeamUID::Allies,
                monster_number: MonsterNumber::_1,
            },
            MonsterUID {
                team_uid: TeamUID::Opponents,
                monster_number: MonsterNumber::_1,
            },
            EventFilteringOptions::default(),
        );
        assert!(passed_filter);
    }

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_handler_instance() {
        use crate::sim::test_ability_dex::FlashFire;
        let event_handler_instance = OwnedEventHandler {
            event_name: event_dex::OnTryMove.name(),
            event_handler: FlashFire.event_handler_deck.on_try_move.unwrap(),
            owner_uid: MonsterUID {
                team_uid: crate::sim::TeamUID::Allies,
                monster_number: crate::sim::MonsterNumber::_1,
            },
            activation_order: crate::sim::ActivationOrder {
                priority: 1,
                speed: 99,
                order: 0,
            },
            filtering_options: crate::sim::EventFilteringOptions::default(),
        };
        println!("{:#?}", event_handler_instance);
    }
}
