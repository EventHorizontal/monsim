#[cfg(test)]
mod main {
    use battle_context_macro::battle_context;

    use crate::sim::{battle_context::BattleContext, monster_dex::{Treecko, Torchic, Mudkip, Drifblim}, ability_dex::FlashFire, move_dex::{Scratch, Ember, Bubble, Tackle, Growl}};

    #[test]
    fn test_bcontext_macro() {
        extern crate self as monsim;
        let test_bcontext = battle_context!(
            {
                AllyTeam {
                    mon Treecko "Ruby" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "Sapphire" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Mudkip "Emerald" {
                        mov Bubble,
                        mov Scratch,
                        abl FlashFire,
                    },
                },
                OpponentTeam {
                    mon Drifblim "Cheerio" {
                        mov Tackle,
                        mov Growl,
                        abl FlashFire,
                    },
                }
            }
        );
        assert_eq!(
            test_bcontext,
            {
                BattleContext::new(
                    monsim::sim::AllyBattlerTeam(monsim::sim::BattlerTeam::new(
                        vec![
                                (monsim::sim::Battler::new(
                                    monsim::sim::BattlerUID {
                                        team_id: monsim::sim::TeamID::Ally,
                                        battler_number: monsim::sim::monster::BattlerNumber::from(0usize),
                                    },
                                    true,
                                    monsim::sim::monster::Monster::new(
                                        monsim::sim::monster_dex::Torchic,
                                        "Ruby",
                                    ),
                                    monsim::sim::move_::MoveSet::new(
                                        vec![
                                                (monsim::sim::move_::Move::new(
                                                    monsim::sim::move_dex::Scratch,
                                                )),
                                                (monsim::sim::move_::Move::new(
                                                    monsim::sim::move_dex::Ember,
                                                )),
                                            ],
                                    ),
                                    monsim::sim::ability::Ability::new(monsim::sim::ability_dex::FlashFire),
                                )),
                                (monsim::sim::Battler::new(
                                    monsim::sim::BattlerUID {
                                        team_id: monsim::sim::TeamID::Ally,
                                        battler_number: monsim::sim::monster::BattlerNumber::from(1usize),
                                    },
                                    false,
                                    monsim::sim::monster::Monster::new(
                                        monsim::sim::monster_dex::Torchic,
                                        "Sapphire",
                                    ),
                                    monsim::sim::move_::MoveSet::new(
                                        vec![
                                                (monsim::sim::move_::Move::new(
                                                    monsim::sim::move_dex::Scratch,
                                                )),
                                                (monsim::sim::move_::Move::new(
                                                    monsim::sim::move_dex::Ember,
                                                )),
                                            ],
                                    ),
                                    monsim::sim::ability::Ability::new(monsim::sim::ability_dex::FlashFire),
                                )),
                                (monsim::sim::Battler::new(
                                    monsim::sim::BattlerUID {
                                        team_id: monsim::sim::TeamID::Ally,
                                        battler_number: monsim::sim::monster::BattlerNumber::from(2usize),
                                    },
                                    false,
                                    monsim::sim::monster::Monster::new(
                                        monsim::sim::monster_dex::Torchic,
                                        "Emerald",
                                    ),
                                    monsim::sim::move_::MoveSet::new(
                                        vec![
                                                (monsim::sim::move_::Move::new(
                                                    monsim::sim::move_dex::Scratch,
                                                )),
                                                (monsim::sim::move_::Move::new(
                                                    monsim::sim::move_dex::Ember,
                                                )),
                                            ],
                                    ),
                                    monsim::sim::ability::Ability::new(monsim::sim::ability_dex::FlashFire),
                                )),
                            ],
                    )),
                    monsim::sim::OpponentBattlerTeam(monsim::sim::BattlerTeam::new(
                        vec![(monsim::sim::Battler::new(
                                monsim::sim::BattlerUID {
                                    team_id: monsim::sim::TeamID::Opponent,
                                    battler_number: monsim::sim::monster::BattlerNumber::from(0usize),
                                },
                                true,
                                monsim::sim::monster::Monster::new(
                                    monsim::sim::monster_dex::Torchic,
                                    "Cheerio",
                                ),
                                monsim::sim::move_::MoveSet::new(
                                    vec![
                                            (monsim::sim::move_::Move::new(monsim::sim::move_dex::Scratch)),
                                            (monsim::sim::move_::Move::new(monsim::sim::move_dex::Ember)),
                                        ],
                                ),
                                monsim::sim::ability::Ability::new(monsim::sim::ability_dex::FlashFire),
                            ))],
                    )),
                )
            }
        );
    }
}

#[cfg(test)]
mod bcontext {

    use battle_context_macro::battle_context;

    use crate::sim::{BattlerNumber, BattlerUID, EventHandlerFilters, TeamID, };

    #[test]
    fn test_event_filtering_for_event_sources() {
        extern crate self as monsim;
        use crate::sim::{battle_context::BattleContext, monster_dex::{Treecko, Torchic, Mudkip}, ability_dex::FlashFire, move_dex::{Scratch, Ember, Bubble, Tackle}};
        let test_battle_context = battle_context!(
            {
                AllyTeam {
                    mon Torchic "Ruby" {
                        mov Ember,
                        mov Scratch,
                        abl FlashFire,
                    },
                    mon Mudkip "Sapphire" {
                        mov Tackle,
                        mov Bubble,
                        abl FlashFire,
                    },
                },
                OpponentTeam {
                    mon Treecko "Emerald" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                }
            }
        );

        let passed_filter = test_battle_context.filter_event_handlers(
            BattlerUID {
                team_id: TeamID::Ally,
                battler_number: BattlerNumber::_1,
            },
            BattlerUID {
                team_id: TeamID::Opponent,
                battler_number: BattlerNumber::_1,
            },
            EventHandlerFilters::default(),
        );
        assert!(passed_filter);
    }

    #[test]
    fn test_display_battle_context() {
        extern crate self as monsim;
        use crate::sim::{battle_context::BattleContext, monster_dex::{Treecko, Torchic, Mudkip, Drifloon}, ability_dex::FlashFire, move_dex::{Scratch, Ember, Bubble, Tackle}};
        let test_bcontext = battle_context!(
            {
                AllyTeam {
                    mon Torchic "Ruby" {
                        mov Ember,
                        mov Scratch,
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
                    mon Drifloon "Cheerio" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                }
            }
        );
        println!("{}", test_bcontext);
        assert_eq!(format!["{}", test_bcontext], String::from("Ally Team\n\t├── Ruby the Torchic (Ally_1) [HP: 152/152]\n\t│\t│\n\t│\t├── type Fire/None \n\t│\t├── abl Flash Fire\n\t│\t├── mov Ember\n\t│\t└── mov Scratch\n\t│\t\n\t├── Sapphire the Mudkip (Ally_2) [HP: 157/157]\n\t│\t│\n\t│\t├── type Water/None \n\t│\t├── abl Flash Fire\n\t│\t├── mov Tackle\n\t│\t└── mov Bubble\n\t│\t\n\t└── Emerald the Treecko (Ally_3) [HP: 147/147]\n\t\t│\n\t\t├── type Grass/None \n\t\t├── abl Flash Fire\n\t\t├── mov Scratch\n\t\t└── mov Ember\n\t\t\nOpponent Team\n\t└── Cheerio the Drifloon (Opponent_1) [HP: 197/197]\n\t\t│\n\t\t├── type Ghost/Flying \n\t\t├── abl Flash Fire\n\t\t├── mov Scratch\n\t\t└── mov Ember\n\t\t\n"))
    }
}

#[cfg(test)]
mod event {

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_handler() {
        use crate::sim::game_mechanics::ability_dex::FlashFire;
        let event_handler = FlashFire.event_handlers.on_try_move.unwrap();
        println!("{:?}", event_handler);
    }

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_handler_set() {
        use crate::sim::ability_dex::FlashFire;
        println!("{:#?}", FlashFire.event_handlers);
    }
    
}

#[cfg(test)]
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
