use core::fmt::Debug;

use crate::sim::{ordering::sort_by_activation_order, Battle, Nothing, Outcome, Percent};
use contexts::*;
use event_setup_macro::event_setup;

use super::MonsterRef;

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
pub struct OwnedEventHandler<'a, R: Copy, C: Copy> {
    pub event_name: &'static str,
    pub event_handler: for<'b> fn(&'b mut Battle, C, R) -> R,
    pub owner: MonsterRef<'a>,
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
    use crate::sim::{MonsterRef, MoveRef};

    #[derive(Debug, Clone, Copy)]
    pub struct MoveUsed<'a> {
        pub attacker: MonsterRef<'a>,
        pub move_: MoveRef<'a>,
        pub target: MonsterRef<'a>,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct AbilityUsed<'a> {
        pub ability_owner: MonsterRef<'a>,
    }

    impl<'a> MoveUsed<'a> {
        pub fn new(attacker: MonsterRef<'a>, move_: MoveRef<'a>, target: MonsterRef<'a>) -> Self {
            Self {
                attacker,
                move_,
                target,
            }
        }
    }

    impl<'a> AbilityUsed<'a> {
        pub fn new(ability_owner: MonsterRef<'a>) -> Self {
            Self {
                ability_owner,
            }
        }
    }
}

// event_setup![
//     /// A "deck" is meant to be a collection with 0-1 of each "card".
//     pub struct EventHandlerDeck {
//         match event {
//             /// Return value: `Outcome::Success` means the move succeeded.
//             #[context(MoveUsed)]
//             on_try_move => Outcome,

//             #[context(MoveUsed)]
//             on_damage_dealt => Nothing,

//             /// Return value: `Outcome::Success` means ability activation succeeded.
//             #[context(AbilityUsed)]
//             on_try_activate_ability => Outcome,

//             #[context(AbilityUsed)]
//             on_ability_activated => Nothing,

//             /// Return value: `Percent` value indicates percentage multiplier for
//             /// accuracy modification.
//             #[context(MoveUsed)]
//             on_modify_accuracy => Percent,

//             /// Return value: `Outcome::Success` means stat was successfully raised.
//             #[context(Nothing)]
//             on_try_raise_stat => Outcome,

//             /// Return value: `Outcome::Success` means stat was successfully lowered.
//             #[context(Nothing)]
//             on_try_lower_stat => Outcome,

//             #[context(MoveUsed)]
//             on_status_move_used => Nothing,
//         }
//     }
//     const DEFAULT_DECK = None;
//     pub trait InBattleEvent;
// ];

#[doc = r#" A "deck" is meant to be a collection with 0-1 of each "card"."#]
#[derive(Debug, Clone, Copy)]
pub struct EventHandlerDeck {
    #[doc = r" Return value: `Outcome::Success` means the move succeeded."]
    pub on_try_move: Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Outcome) -> Outcome>,
    pub on_damage_dealt: Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Nothing) -> Nothing>,
    #[doc = r" Return value: `Outcome::Success` means ability activation succeeded."]
    pub on_try_activate_ability: Option<for<'b> fn(&'b mut Battle, AbilityUsed<'b>, Outcome) -> Outcome>,
    pub on_ability_activated: Option<for<'b> fn(&'b mut Battle, AbilityUsed<'b>, Nothing) -> Nothing>,
    #[doc = r" Return value: `Percent` value indicates percentage multiplier for"]
    #[doc = r" accuracy modification."]
    pub on_modify_accuracy: Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Percent) -> Percent>,
    #[doc = r" Return value: `Outcome::Success` means stat was successfully raised."]
    pub on_try_raise_stat: Option<for<'b> fn(&'b mut Battle, Nothing, Outcome) -> Outcome>,
    #[doc = r" Return value: `Outcome::Success` means stat was successfully lowered."]
    pub on_try_lower_stat: Option<for<'b> fn(&'b mut Battle, Nothing, Outcome) -> Outcome>,
    pub on_status_move_used: Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Nothing) -> Nothing>,
}
const DEFAULT_DECK: EventHandlerDeck = EventHandlerDeck {
    on_try_move: None,
    on_damage_dealt: None,
    on_try_activate_ability: None,
    on_ability_activated: None,
    on_modify_accuracy: None,
    on_try_raise_stat: None,
    on_try_lower_stat: None,
    on_status_move_used: None,
};
pub trait InBattleEvent: Clone + Copy {
    type EventReturnType: Sized + Clone + Copy;
    type EventContext<'a>: Sized + Clone + Copy;
    fn corresponding_handler(
        &self,
        event_handler_deck: &EventHandlerDeck,
    ) -> Option<for<'b> fn(&'b mut Battle, Self::EventContext<'b>, Self::EventReturnType) -> Self::EventReturnType>;

    fn name(&self) -> &'static str;
}
pub mod event_dex {
    use super::*;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnTryMove;

    impl InBattleEvent for OnTryMove {
        type EventReturnType = Outcome;
        type EventContext<'a> = MoveUsed<'a>;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Outcome) -> Outcome> {
            event_handler_deck.on_try_move
        }
        fn name(&self) -> &'static str {
            "OnTryMove"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnDamageDealt;

    impl InBattleEvent for OnDamageDealt {
        type EventReturnType = Nothing;
        type EventContext<'a> = MoveUsed<'a>;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Nothing) -> Nothing> {
            event_handler_deck.on_damage_dealt
        }
        fn name(&self) -> &'static str {
            "OnDamageDealt"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnTryActivateAbility;

    impl InBattleEvent for OnTryActivateAbility {
        type EventReturnType = Outcome;
        type EventContext<'a> = AbilityUsed<'a>;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, AbilityUsed<'b>, Outcome) -> Outcome> {
            event_handler_deck.on_try_activate_ability
        }
        fn name(&self) -> &'static str {
            "OnTryActivateAbility"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnAbilityActivated;

    impl InBattleEvent for OnAbilityActivated {
        type EventReturnType = Nothing;
        type EventContext<'a> = AbilityUsed<'a>;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, AbilityUsed<'b>, Nothing) -> Nothing> {
            event_handler_deck.on_ability_activated
        }
        fn name(&self) -> &'static str {
            "OnAbilityActivated"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnModifyAccuracy;

    impl InBattleEvent for OnModifyAccuracy {
        type EventReturnType = Percent;
        type EventContext<'a> = MoveUsed<'a>;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Percent) -> Percent> {
            event_handler_deck.on_modify_accuracy
        }
        fn name(&self) -> &'static str {
            "OnModifyAccuracy"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnTryRaiseStat;

    impl InBattleEvent for OnTryRaiseStat {
        type EventReturnType = Outcome;
        type EventContext<'a> = Nothing;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, Nothing, Outcome) -> Outcome> {
            event_handler_deck.on_try_raise_stat
        }
        fn name(&self) -> &'static str {
            "OnTryRaiseStat"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnTryLowerStat;

    impl InBattleEvent for OnTryLowerStat {
        type EventReturnType = Outcome;
        type EventContext<'a> = Nothing;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, Nothing, Outcome) -> Outcome> {
            event_handler_deck.on_try_lower_stat
        }
        fn name(&self) -> &'static str {
            "OnTryLowerStat"
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OnStatusMoveUsed;

    impl InBattleEvent for OnStatusMoveUsed {
        type EventReturnType = Nothing;
        type EventContext<'a> = MoveUsed<'a>;
        fn corresponding_handler(&self, event_handler_deck: &EventHandlerDeck) -> Option<for<'b> fn(&'b mut Battle, MoveUsed<'b>, Nothing) -> Nothing> {
            event_handler_deck.on_status_move_used
        }
        fn name(&self) -> &'static str {
            "OnStatusMoveUsed"
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OwnedEventHandlerDeck<'a> {
    pub event_handler_deck: EventHandlerDeck,
    pub owner: MonsterRef<'a>,
    pub activation_order: ActivationOrder,
    pub filtering_options: EventFilteringOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActivationOrder {
    pub priority: i8,
    pub speed: u16,
    pub order: u16,
}

impl EventDispatcher {

    pub fn dispatch_trial_event<'a, C: Copy>(
        battle: &mut Battle,
        broadcaster: MonsterRef,
        calling_context: C,
        event: impl InBattleEvent<EventReturnType = Outcome, EventContext<'a> = C>,
    ) -> Outcome {
        Self::dispatch_event(battle, broadcaster, calling_context, event, Outcome::Success, Some(Outcome::Failure))
    }

    /// `default` tells the resolver what value it should return if there are no event handlers, or the event handlers fall through.
    ///
    /// `short_circuit` is an optional value that, if returned by a handler in the chain, the resolution short-circuits and returns early.
    pub fn dispatch_event<'a, R: PartialEq + Copy, C: Copy>(
        battle: &mut Battle,
        event_initiator: MonsterRef,
        calling_context: C,
        event: impl InBattleEvent<EventReturnType = R, EventContext<'a> = C>,
        default: R,
        short_circuit: Option<R>,
    ) -> R {
        let mut event_handler_instances = Self::handlers_for_event(battle.event_handler_decks(), event);

        if event_handler_instances.is_empty() {
            return default;
        }
 
        sort_by_activation_order::<OwnedEventHandler<R, C>>(&mut battle.prng, &mut event_handler_instances, &mut |it| {
            it.activation_order
        });

        let mut relay = default;
        for OwnedEventHandler {
            event_handler,
            owner,
            filtering_options: filter_options,
            ..
        } in event_handler_instances.into_iter()
        {
            if Self::filter_event_handlers(battle, event_initiator, owner, filter_options) {
                relay = (event_handler)(battle, calling_context, relay);
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
        event_caller: MonsterRef,
        handler_owner: MonsterRef,
        filter_options: EventFilteringOptions,
    ) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if event_caller == handler_owner {
                bitmask |= TargetFlags::SELF.bits()
            } // 0x01
            if battle.are_allies(handler_owner, event_caller) {
                bitmask |= TargetFlags::ALLIES.bits()
            } // 0x02
            if battle.are_opponents(handler_owner, event_caller) {
                bitmask |= TargetFlags::OPPONENTS.bits()
            } //0x04
              // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = filter_options.event_source.bits() == bitmask;
        let is_active_passed = battle.is_active_monster(handler_owner);

        event_source_filter_passed && is_active_passed
    }

    fn handlers_for_event<'a, R: Copy, C: Copy>(
        event_handler_deck_instances: Vec<OwnedEventHandlerDeck>,
        event: impl InBattleEvent<EventReturnType = R, EventContext<'a> = C>,
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
    pub const fn const_default() -> Self {
        DEFAULT_DECK
    }
}

impl<'a> OwnedEventHandlerDeck<'a> {
    fn handler_for_event<R: Copy, C: Copy>(
        &self,
        event: impl InBattleEvent<EventReturnType = R, EventContext<'a> = C>,
    ) -> Option<OwnedEventHandler<'a, R, C>> {
        let event_handler = event.corresponding_handler(&self.event_handler_deck);
        event_handler.map( move |event_handler| OwnedEventHandler {
            // INFO: Trait methods are non-const so we can only add the `event_name` during runtime.
            event_name: event.name(),
            event_handler,
            owner: self.owner,
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

// #[cfg(all(test, feature = "debug"))]
// mod tests {
//     use super::*;
//     use crate::sim::build_battle;

//     #[test]
//     fn test_if_priority_sorting_is_deterministic() {
//         extern crate self as monsim;
//         use crate::sim::*;
//         use crate::sim::{
//             test_ability_dex::FlashFire,
//             test_monster_dex::{Drifblim, Mudkip, Torchic, Treecko},
//             test_move_dex::{Bubble, Ember, Scratch, Tackle},
//         };
//         let mut result = [Vec::new(), Vec::new()];
//         for i in 0..=1 {
//             let test_battle = build_battle!(
//                 {
//                     Allies: MonsterTeam {
//                         Torchic: Monster = "Ruby" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Mudkip: Monster = "Sapphire" {
//                             Tackle: Move,
//                             Bubble: Move,
//                             FlashFire: Ability,
//                         },
//                         Treecko: Monster = "Emerald" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                     },
//                     Opponents: MonsterTeam {
//                         Drifblim: Monster {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                     }
//                 }
//             );

//             let mut prng = Prng::new(crate::sim::prng::seed_from_time_now());

//             let event_handler_deck_instances = test_battle.event_handler_deck_instances();
//             use crate::sim::event_dex::OnTryMove;
//             let mut event_handler_instances = EventDispatcher::handlers_for_event(event_handler_deck_instances, OnTryMove);

//             crate::sim::ordering::sort_by_activation_order(&mut prng, &mut event_handler_instances, &mut |it| it.activation_order);

//             result[i] = event_handler_instances
//                 .into_iter()
//                 .map(|event_handler_instance| test_battle.monster(event_handler_instance.owner_uid).name())
//                 .collect::<Vec<_>>();
//         }

//         assert_eq!(result[0], result[1]);
//         assert_eq!(result[0][0], "Drifblim");
//         assert_eq!(result[0][1], "Emerald");
//         assert_eq!(result[0][2], "Ruby");
//         assert_eq!(result[0][3], "Sapphire");
//     }
// 
//     #[test]
//     #[cfg(feature = "debug")]
//     fn test_priority_sorting_with_speed_ties() {
//         #[cfg(feature = "debug")]
//         extern crate self as monsim;
//         use crate::sim::*;
//         use crate::sim::{
//             test_ability_dex::FlashFire,
//             test_monster_dex::{Drifblim, Mudkip, Torchic},
//             test_move_dex::{Ember, Scratch},
//         };
//         let mut result = [Vec::new(), Vec::new()];
//         for i in 0..=1 {
//             let test_battle = build_battle!(
//                 {
//                     Allies: MonsterTeam {
//                         Torchic: Monster = "A" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "B" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "C" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "D" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "E" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Mudkip: Monster = "F" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         }
//                     },
//                     Opponents: MonsterTeam {
//                         Drifblim: Monster = "G" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "H" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "I" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "J" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "K" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                         Torchic: Monster = "L" {
//                             Scratch: Move,
//                             Ember: Move,
//                             FlashFire: Ability,
//                         },
//                     }
//                 }
//             );
//             let mut prng = Prng::new(i as u64);

//             let event_handler_deck_instances = test_battle.event_handler_deck_instances();
//             use crate::sim::event_dex::OnTryMove;

//             let mut event_handler_instances = EventDispatcher::handlers_for_event(event_handler_deck_instances, OnTryMove);

//             crate::sim::ordering::sort_by_activation_order(&mut prng, &mut event_handler_instances, &mut |it| it.activation_order);

//             result[i] = event_handler_instances
//                 .into_iter()
//                 .map(|event_handler_instance| test_battle.monster(event_handler_instance.owner_uid).name())
//                 .collect::<Vec<_>>();
//         }

//         // Check that the two runs are not equal, there is an infinitesimal chance they won't be, but the probability is negligible.
//         assert_ne!(result[0], result[1]);
//         // Check that Drifblim is indeed the in the front.
//         assert_eq!(result[0][0], "G");
//         // Check that the Torchics are all in the middle.
//         for name in ["A", "B", "C", "D", "E", "H", "I", "J", "K", "L"].iter() {
//             assert!(result[0].contains(&name.to_string()));
//         }
//         //Check that the Mudkip is last.
//         assert_eq!(result[0][11], "F");
//     }

//     #[test]
//     #[cfg(feature = "debug")]
//     fn test_event_filtering_for_event_sources() {
//         extern crate self as monsim;
//         use crate::sim::*;
//         use crate::sim::{
//             test_ability_dex::FlashFire,
//             test_monster_dex::{Mudkip, Torchic, Treecko},
//             test_move_dex::{Bubble, Ember, Scratch, Tackle},
//             MonsterNumber, TeamUID,
//         };
//         let test_battle = build_battle!(
//             {
//                 Allies: MonsterTeam {
//                     Torchic: Monster = "Ruby" {
//                         Ember: Move,
//                         Scratch: Move,
//                         FlashFire: Ability,
//                     },
//                     Mudkip: Monster = "Sapphire" {
//                         Tackle: Move,
//                         Bubble: Move,
//                         FlashFire: Ability,
//                     },
//                 },
//                 Opponents: MonsterTeam {
//                     Treecko: Monster = "Emerald" {
//                         Scratch: Move,
//                         Ember: Move,
//                         FlashFire: Ability,
//                     },
//                 }
//             }
//         );

//         let passed_filter = EventDispatcher::filter_event_handlers(
//             &test_battle,
//             MonsterUID {
//                 team_uid: TeamUID::Allies,
//                 monster_number: MonsterNumber::_1,
//             },
//             MonsterUID {
//                 team_uid: TeamUID::Opponents,
//                 monster_number: MonsterNumber::_1,
//             },
//             EventFilteringOptions::default(),
//         );
//         assert!(passed_filter);
//     }

//     #[test]
//     #[cfg(feature = "debug")]
//     fn test_print_event_handler_instance() {
//         use crate::sim::test_ability_dex::FlashFire;
//         let event_handler_instance = OwnedEventHandler {
//             event_name: event_dex::OnTryMove.name(),
//             event_handler: FlashFire.event_handler_deck.on_try_move.unwrap(),
//             owner_uid: MonsterUID {
//                 team_uid: crate::sim::TeamUID::Allies,
//                 monster_number: crate::sim::MonsterNumber::_1,
//             },
//             activation_order: crate::sim::ActivationOrder {
//                 priority: 1,
//                 speed: 99,
//                 order: 0,
//             },
//             filtering_options: crate::sim::EventFilteringOptions::default(),
//         };
//         println!("{:#?}", event_handler_instance);
//     }
// }
