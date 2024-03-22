#[cfg(all(test, feature = "debug"))]
mod main {
    use monsim_macros::battle_state;

    use crate::sim::*;
    use crate::sim::{
        test_ability_dex::FlashFire,
        battle::BattleState,
        test_monster_dex::{Drifblim, Mudkip, Torchic, Treecko},
        test_move_dex::{Bubble, Ember, Growl, Scratch, Tackle},
        Ability, Monster, Move,
    };

    #[test]
    fn test_battle_macro() {
        extern crate self as monsim;
        let test_battle = battle_state!(
            {
                Allies: MonsterTeam {
                    Torchic: Monster = "Ruby" {
                        Scratch: Move,
                        Ember: Move,
                        FlashFire: Ability,
                    },
                    Mudkip: Monster = "Sapphire" {
                        Scratch: Move,
                        Ember: Move,
                        FlashFire: Ability,
                    },
                    Treecko: Monster = "Emerald" {
                        Bubble: Move,
                        Scratch: Move,
                        FlashFire: Ability,
                    },
                },
                Opponents: MonsterTeam {
                    Drifblim: Monster {
                        Tackle: Move,
                        Growl: Move,
                        FlashFire: Ability,
                    },
                }
            }
        );
        assert_eq!(test_battle, 
            BattleState::new(
                    MonsterTeam::new(vec![
                        (Monster::new(
                            MonsterUID {
                                team_uid: TeamUID::Allies,
                                monster_number: MonsterNumber::from(0usize),
                            },
                            &test_monster_dex::Torchic, 
                            Some("Ruby"),
                            move_::MoveSet::new(vec![(move_::Move::new(&test_move_dex::Scratch)), (move_::Move::new(&test_move_dex::Ember))]),
                            ability::Ability::new(&test_ability_dex::FlashFire),
                        )),
                        (Monster::new(
                            MonsterUID {
                                team_uid: TeamUID::Allies,
                                monster_number: MonsterNumber::from(1usize),
                            },
                            &test_monster_dex::Mudkip, 
                            Some("Sapphire"),
                            move_::MoveSet::new(vec![(move_::Move::new(&test_move_dex::Scratch)), (move_::Move::new(&test_move_dex::Ember))]),
                            ability::Ability::new(&test_ability_dex::FlashFire),
                        )),
                        (Monster::new(
                            MonsterUID {
                                team_uid: TeamUID::Allies,
                                monster_number: MonsterNumber::from(2usize),
                            },
                            &test_monster_dex::Treecko, 
                            Some("Emerald"),
                            move_::MoveSet::new(vec![(move_::Move::new(&test_move_dex::Bubble)), (move_::Move::new(&test_move_dex::Scratch))]),
                            ability::Ability::new(&test_ability_dex::FlashFire),
                        )),
                    ], TeamUID::Allies),
                    MonsterTeam::new(vec![
                        (Monster::new(
                            MonsterUID {
                                team_uid: TeamUID::Opponents,
                                monster_number: MonsterNumber::from(0usize),
                            },
                            &test_monster_dex::Drifblim, 
                            None,
                            move_::MoveSet::new(vec![(move_::Move::new(&test_move_dex::Tackle)), (move_::Move::new(&test_move_dex::Growl))]),
                            ability::Ability::new(&test_ability_dex::FlashFire),
                        )),
                    ], TeamUID::Opponents),
            )
        );
    }
}

#[cfg(all(test, feature = "debug"))]
mod battle {
    use monsim_macros::battle_state;

    #[test]
    fn test_display_battle() {
        extern crate self as monsim;
        use crate::sim::*;
        use crate::sim::{
            test_ability_dex::FlashFire,
            test_monster_dex::{Drifloon, Mudkip, Torchic, Treecko},
            test_move_dex::{Bubble, Ember, Scratch, Tackle},
        };
        let test_battle = battle_state!(
            {
                Allies: MonsterTeam {
                    Torchic: Monster = "Ruby" {
                        Ember: Move,
                        Scratch: Move,
                        FlashFire: Ability,
                    },
                    Mudkip: Monster {
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
                    Drifloon: Monster = "Cheerio" {
                        Scratch: Move,
                        Ember: Move,
                        FlashFire: Ability,
                    },
                }
            }
        );
        println!("{}", test_battle);
        assert_eq!(
            format!["{}", test_battle],
            String::from(
                "Ally Team
\t├── Ruby the Torchic (Allies_1) [HP: 152/152]
\t│\t│
\t│\t├──    type: Fire
\t│\t├── ability: Flash Fire
\t│\t├──    move: Ember
\t│\t└──    move: Scratch
\t│\t
\t├── Mudkip (Allies_2) [HP: 157/157]
\t│\t│
\t│\t├──    type: Water
\t│\t├── ability: Flash Fire
\t│\t├──    move: Tackle
\t│\t└──    move: Bubble
\t│\t
\t└── Emerald the Treecko (Allies_3) [HP: 147/147]
\t \t│
\t \t├──    type: Grass
\t \t├── ability: Flash Fire
\t \t├──    move: Scratch
\t \t└──    move: Ember
\t \t
Opponent Team
\t└── Cheerio the Drifloon (Opponents_1) [HP: 197/197]
\t \t│
\t \t├──    type: Ghost/Flying
\t \t├── ability: Flash Fire
\t \t├──    move: Scratch
\t \t└──    move: Ember
\t \t
"
            )
        )
    }
}

#[cfg(all(test, feature = "debug"))]
mod event {

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_handler() {
        use crate::sim::game_mechanics::test_ability_dex::FlashFire;
        let event_handler = FlashFire.event_handler_deck.on_try_move.unwrap();
        println!("{:?}", event_handler);
    }

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_handler_deck() {
        use crate::sim::test_ability_dex::FlashFire;
        println!("{:#?}", FlashFire.event_handler_deck);
    }
}

#[cfg(all(test, feature = "debug"))]
mod prng {
    use crate::sim::prng::*;

    #[test]
    fn test_prng_percentage_chance() {
        let mut prng = Prng::from_current_time();
        let mut dist = [0u64; 100];
        for _ in 0..=10_000_000 {
            let n = prng.generate_random_u16_in_range(0..=99) as usize;
            dist[n] += 1;
        }
        let avg_deviation = dist
            .iter()
            .map(|it| ((*it as f32 / 100_000.0) - 1.0).abs())
            .reduce(|it, acc| it + acc)
            .expect("We should always get some average value.")
            / 100.0;
        let avg_deviation = f32::floor(avg_deviation * 100_000.0) / 100_000.0;
        println!("LCRNG has {:?}% average deviation (threshold is at 0.005%)", avg_deviation);
        assert!(avg_deviation < 5.0e-3);
    }

    #[test]
    fn test_if_prng_is_deterministic_for_specific_seed() {
        let mut prng1 = Prng::from_current_time();
        let mut prng2 = prng1;
        for i in 0..10_000 {
            let generated_number_1 = prng1.generate_random_u16_in_range(0..=u16::MAX - 1);
            let generated_number_2 = prng2.generate_random_u16_in_range(0..=u16::MAX - 1);
            assert_eq!(generated_number_1, generated_number_2, "iteration {}", i);
        }
    }

    #[test]
    fn test_prng_chance() {
        let mut prng = Prng::from_current_time();

        let mut success = 0.0;
        for _ in 0..=10_000_000 {
            if prng.chance(33, 100) {
                success += 1.0;
            }
        }
        let avg_probability_deviation = (((success / 10_000_000.0) - 0.3333333333) as f64).abs();
        let avg_probability_deviation = f64::floor(avg_probability_deviation * 100_000.0) / 100_000.0;
        println!("Average probability of LCRNG is off by {}% (threshold is at 0.005%)", avg_probability_deviation);
        assert!(avg_probability_deviation < 5.0e-3);
    }
}

#[cfg(all(test, feature = "debug"))]
mod utils {
    use monsim_utils::{Ally, TeamAffl};

    #[test]
    #[should_panic]
    fn test_expect_wrong_team() {
        let item = Ally::new(10usize);
        let item = TeamAffl::ally(item);
        (item.map(|i| { i - 1 }).expect_opponent());
    }

    #[test]
    fn test_expect_right_team() {
        let item = Ally::new(10usize);
        let item = TeamAffl::ally(item);
        item.map(|i| {i + 1}).expect_ally();
    }
}