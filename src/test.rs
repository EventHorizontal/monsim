
#[cfg(all(test, feature = "debug"))]
mod battle {

    #[test]
    fn test_display_battle() {
        extern crate self as monsim;
        use crate::sim::*;
        use crate::sim::{
            test_ability_dex::FlashFire,
            test_monster_dex::{Zombler, Merkey, Squirecoal, Dandyleo},
            test_move_dex::{Bubble, Ember, Scratch, Tackle},
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
                            .with_nickname("Cheerio")
                        )
                )
                .build();

        println!("{}", test_battle);
        assert_eq!(
            format!["{}", test_battle],
            String::from(
                "Ally Team
\t├── Ruby the Squirecoal (First Ally) [HP: 152/152] @ AllySideCentre
\t│\t│
\t│\t├──    type: Fire
\t│\t├── ability: Flash Fire
\t│\t├──    move: Ember
\t│\t└──    move: Scratch
\t│\t
\t├── Merkey (Second Ally) [HP: 157/157] @ Benched
\t│\t│
\t│\t├──    type: Water/Bug
\t│\t├── ability: Flash Fire
\t│\t├──    move: Tackle
\t│\t└──    move: Bubble
\t│\t
\t└── Emerald the Dandyleo (Third Ally) [HP: 147/147] @ Benched
\t \t│
\t \t├──    type: Grass
\t \t├── ability: Flash Fire
\t \t├──    move: Scratch
\t \t└──    move: Ember
\t \t
Opponent Team
\t└── Cheerio the Zombler (First Opponent) [HP: 197/197] @ OpponentSideCentre
\t \t│
\t \t├──    type: Ghost/Dark
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
        let event_handler = FlashFire.event_handlers().on_try_move_hit.unwrap();
        println!("{:?}", event_handler);
    }

    #[test]
    #[cfg(feature = "debug")]
    fn test_print_event_handler_deck() {
        use crate::sim::test_ability_dex::FlashFire;
        println!("{:#?}", FlashFire.event_handlers());
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