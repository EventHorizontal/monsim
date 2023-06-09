#[cfg(all(test, feature = "debug"))]
mod main {
    use battle_builder_macro::build_battle;

    use crate::sim::*;
    use crate::sim::{
        ability_dex::FlashFire,
        context::Battle,
        monster_dex::{Drifblim, Mudkip, Torchic, Treecko},
        move_dex::{Bubble, Ember, Growl, Scratch, Tackle},
        Monster, Ability, Move
    };

    #[test]
    fn test_build_battle_macro() {
        extern crate self as monsim;
        let test_battle = build_battle!(
            {
                Allies: BattlerTeam {
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
                Opponents: BattlerTeam {
                    Drifblim: Monster = "Cheerio" {
                        Tackle: Move,
                        Growl: Move,
                        FlashFire: Ability,
                    },
                }
            }
        );
        assert_eq!(test_battle, {
            Battle::new(
                AllyBattlerTeam(BattlerTeam::new(vec![
                    (Battler::new(
                        BattlerUID {
                            team_id: TeamID::Allies,
                            battler_number: monster::BattlerNumber::from(0usize),
                        },
                        true,
                        monster::Monster::new(
                            monster_dex::Torchic,
                            "Ruby",
                        ),
                        move_::MoveSet::new(vec![
                            (move_::Move::new(move_dex::Scratch)),
                            (move_::Move::new(move_dex::Ember)),
                        ]),
                        ability::Ability::new(ability_dex::FlashFire),
                    )),
                    (Battler::new(
                        BattlerUID {
                            team_id: TeamID::Allies,
                            battler_number: monster::BattlerNumber::from(1usize),
                        },
                        false,
                        monster::Monster::new(
                            monster_dex::Mudkip,
                            "Sapphire",
                        ),
                        move_::MoveSet::new(vec![
                            (move_::Move::new(move_dex::Scratch)),
                            (move_::Move::new(move_dex::Ember)),
                        ]),
                        ability::Ability::new(ability_dex::FlashFire),
                    )),
                    (Battler::new(
                        BattlerUID {
                            team_id: TeamID::Allies,
                            battler_number: monster::BattlerNumber::from(2usize),
                        },
                        false,
                        monster::Monster::new(
                            monster_dex::Treecko,
                            "Emerald",
                        ),
                        move_::MoveSet::new(vec![
                            (move_::Move::new(move_dex::Bubble)),
                            (move_::Move::new(move_dex::Scratch)),
                        ]),
                        ability::Ability::new(ability_dex::FlashFire),
                    )),
                ])),
                OpponentBattlerTeam(BattlerTeam::new(vec![
                    (Battler::new(
                        BattlerUID {
                            team_id: TeamID::Opponents,
                            battler_number: monster::BattlerNumber::from(0usize),
                        },
                        true,
                        monster::Monster::new(
                            monster_dex::Drifblim,
                            "Cheerio",
                        ),
                        move_::MoveSet::new(vec![
                            (move_::Move::new(move_dex::Tackle)),
                            (move_::Move::new(move_dex::Growl)),
                        ]),
                        ability::Ability::new(ability_dex::FlashFire),
                    )),
                ])),
            )
        });
    }
}

#[cfg(all(test, feature = "debug"))]
mod bcontext {
    use battle_builder_macro::build_battle;
    
    #[test]
    fn test_display_battle_context() {
        extern crate self as monsim;
        use crate::sim::*;
        use crate::sim::{
            ability_dex::FlashFire,
            monster_dex::{Drifloon, Mudkip, Torchic, Treecko},
            move_dex::{Bubble, Ember, Scratch, Tackle},
        };
        let test_bcontext = build_battle!(
            {
                Allies: BattlerTeam {
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
                Opponents: BattlerTeam {
                    Drifloon: Monster = "Cheerio" {
                        Scratch: Move,
                        Ember: Move,
                        FlashFire: Ability,
                    },
                }
            }
        );
        println!("{}", test_bcontext);
        assert_eq!(format!["{}", test_bcontext], String::from(
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
"))
    }
}

#[cfg(all(test, feature = "debug"))]
mod event {

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_specific_event_responder() {
        use crate::sim::game_mechanics::ability_dex::FlashFire;
        let specific_event_responder = FlashFire.event_responder.on_try_move.unwrap();
        println!("{:?}", specific_event_responder);
    }

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_responder() {
        use crate::sim::ability_dex::FlashFire;
        println!("{:#?}", FlashFire.event_responder);
    }
}

#[cfg(all(test, feature = "debug"))]
mod prng {
    use std::time;

    use crate::sim::prng::*;

    #[test]
    fn test_prng_percentage_chance() {
        let mut lcrng = Prng::new(seed_from_time_now());
        let mut dist = [0u64; 100];
        for _ in 0..=10_000_000 {
            let n = lcrng.generate_u16_in_range(0..=99) as usize;
            dist[n] += 1;
        }
        let avg_deviation = dist
            .iter()
            .map(|it| ((*it as f32 / 100_000.0) - 1.0).abs())
            .reduce(|it, acc| it + acc)
            .expect("We should always get some average value.")
            / 100.0;
        let avg_deviation = f32::floor(avg_deviation * 100_000.0) / 100_000.0;
        println!(
            "LCRNG has {:?}% average deviation (threshold is at 0.005%)",
            avg_deviation
        );
        assert!(avg_deviation < 5.0e-3);
    }

    #[test]
    fn test_prng_idempotence() {
        let seed = seed_from_time_now();
        let mut lcrng_1 = Prng::new(seed);
        let mut lcrng_2 = Prng::new(seed);
        for i in 0..10_000 {
            let generated_number_1 = lcrng_1.generate_u16_in_range(0..=u16::MAX - 1);
            let generated_number_2 = lcrng_2.generate_u16_in_range(0..=u16::MAX - 1);
            assert_eq!(generated_number_1, generated_number_2, "iteration {}", i);
        }
    }

    #[test]
    fn test_prng_chance() {
        let mut lcrng = Prng::new(
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        let mut success = 0.0;
        for _ in 0..=10_000_000 {
            if lcrng.chance(33, 100) {
                success += 1.0;
            }
        }
        let avg_probability_deviation = (((success / 10_000_000.0) - 0.3333333333) as f64).abs();
        let avg_probability_deviation =
            f64::floor(avg_probability_deviation * 100_000.0) / 100_000.0;
        println!(
            "Average probability of LCRNG is off by {}% (threshold is at 0.005%)",
            avg_probability_deviation
        );
        assert!(avg_probability_deviation < 5.0e-3);
    }
}
