use crate::{prng::Prng, test_monster_dex::Zombler};

#[test]
fn test_if_priority_sorting_is_deterministic() {
    extern crate self as monsim;
    use crate::sim::*;
    use crate::sim::{
        test_ability_dex::FlashFire,
        test_monster_dex::{Merkey, Squirecoal, Dandyleo},
        test_move_dex::{Bubble, Ember, Scratch, Tackle},
    };
    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let test_battle = BattleState::spawn()
            .add_ally_team(
                MonsterTeam::spawn()
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("Ruby")
                    )
                    .add_monster(
                        Merkey.spawn(
                            (Tackle.spawn(), Some(Bubble.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("Sapphire")
                    )
                    .add_monster(
                        Dandyleo.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("Emerald")
                    )
            )
            .add_opponent_team(
                MonsterTeam::spawn()
                    .add_monster(
                            Zombler.spawn(
                                (Scratch.spawn(), Some(Ember.spawn()), None, None),
                                FlashFire.spawn()
                            )
                    )
            )
            .build();

        let mut prng = Prng::from_current_time();
        let sim = BattleSimulator::init(test_battle);
        use crate::sim::event_dex::OnTryMove;
        let mut owned_event_handlers = sim.battle.event_handlers_for(OnTryMove);
        crate::sim::ordering::sort_by_activation_order(&mut prng, &mut owned_event_handlers, |it| it.activation_order);

        result[i] = owned_event_handlers
            .into_iter()
            .map(|event_handler| sim.battle.monster(event_handler.owner).name())
            .collect::<Vec<_>>();
    }

    assert_eq!(result[0], result[1]);
    assert_eq!(result[0][0], "Zombler");
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
        test_monster_dex::{Merkey, Squirecoal},
        test_move_dex::{Ember, Scratch},
    };
    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let test_battle = BattleState::spawn()
            .add_ally_team(
                MonsterTeam::spawn()
                    .add_monster(
                        Zombler.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("A")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("B")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("C")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("D")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("E")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("F")
                    )
            )
            .add_opponent_team(
                MonsterTeam::spawn()
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("G")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("H")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("I")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("J")
                    )
                    .add_monster(
                        Squirecoal.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("K")
                    )
                    .add_monster(
                        Merkey.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("L")
                    )
        )
        .build();
        let mut prng = Prng::new(i as u64);
        let sim = BattleSimulator::init(test_battle);

        use crate::sim::event_dex::OnTryMove;

        let mut owned_event_handlers = sim.battle.event_handlers_for(OnTryMove);

        crate::sim::ordering::sort_by_activation_order(&mut prng, &mut owned_event_handlers, |it| it.activation_order);

        result[i] = owned_event_handlers
            .into_iter()
            .map(|owned_event_handler| sim.battle.monster(owned_event_handler.owner).name())
            .collect::<Vec<_>>();
    }

    // Check that the two runs are not equal, there is an infinitesimal chance they will be by coincidence, but the probability is negligible.
    assert_ne!(result[0], result[1]);
    // Check that Zombler is indeed the in the front.
    assert_eq!(result[0][0], "A");
    // Check that the Squirecoals are all in the middle.
    for name in ["B", "C", "D", "E", "F", "G", "H", "I", "J", "K"].iter() {
        assert!(result[0].contains(&name.to_string()));
    }
    //Check that the Merkey is last.
    assert_eq!(result[0][11], "L");
}

#[test]
#[cfg(feature = "debug")]
fn test_event_filtering_for_event_sources() {
    extern crate self as monsim;
    use crate::sim::*;
    use crate::sim::{
        test_ability_dex::FlashFire,
        test_monster_dex::{Merkey, Squirecoal, Dandyleo},
        test_move_dex::{Bubble, Ember, Scratch, Tackle},
        MonsterNumber, TeamUID,
    };
    let test_battle = BattleState::spawn()
            .add_ally_team(
                MonsterTeam::spawn()
                    .add_monster(
                        Squirecoal.spawn(
                            (Ember.spawn(), Some(Scratch.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("Ruby")
                    )
                    .add_monster(
                        Merkey.spawn(
                            (Tackle.spawn(), Some(Bubble.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("Sapphire")
                    )
                )
                .add_opponent_team(
                    MonsterTeam::spawn()
                    .add_monster(
                        Dandyleo.spawn(
                            (Scratch.spawn(), Some(Ember.spawn()), None, None),
                            FlashFire.spawn()
                        )
                        .with_nickname("Emerald")
                    )
            )
            .build();
    
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
fn test_print_owned_event_handler() {
    use crate::sim::{test_ability_dex::FlashFire, event_dispatch::OwnedEventHandler, MonsterUID};
    let owned_event_handler = OwnedEventHandler {
        event_handler: FlashFire.event_handlers().on_try_move.unwrap(),
        activation_order: crate::ActivationOrder { priority: 0, speed: 11, order: FlashFire.order() },
        owner: MonsterUID {
            team_uid: crate::sim::TeamUID::Allies,
            monster_number: crate::sim::MonsterNumber::_1,
        },
        filtering_options: crate::sim::EventFilteringOptions::default(),
    };
    println!("{:#?}", owned_event_handler);
}
