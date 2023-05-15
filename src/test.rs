
#[cfg(test)]
mod main {
    use bcontext_macro::bcontext_internal;

    use crate::BattleContext;

	#[test]
	fn test_bcontext_macro() {
		let test_bcontext = bcontext_internal!(
			{
				AllyTeam {
					mon Torchic "Ruby" {
						mov Scratch,
						mov Ember,
						abl FlashFire,
					},
					mon Torchic "Sapphire" {
						mov Scratch,
						mov Ember,
						abl FlashFire,
					},
					mon Torchic "Emerald" {
						mov Scratch,
						mov Ember,
						abl FlashFire,
					},
				},
				OpponentTeam {
					mon Torchic "Cheerio" {
						mov Scratch,
						mov Ember,
						abl FlashFire,
					},
				}
			}
		);
		assert_eq!(
			test_bcontext,
			BattleContext::new(
				crate::game_mechanics::BattlerTeam::new(
					vec![
							(crate::game_mechanics::Battler::new(
								crate::game_mechanics::BattlerUID {
									team_id: crate::game_mechanics::TeamID::Ally,
									battler_number: crate::game_mechanics::monster::BattlerNumber::First,
								},
								true,
								crate::game_mechanics::monster::Monster::new(
									crate::game_mechanics::monster_dex::Torchic,
									"Ruby",
								),
								crate::game_mechanics::move_::MoveSet::new(
									vec![
										(crate::game_mechanics::move_::Move::new(
											crate::game_mechanics::move_dex::Scratch,
										)),
										(crate::game_mechanics::move_::Move::new(
											crate::game_mechanics::move_dex::Ember,
										)),
									],
								),
								crate::game_mechanics::ability::Ability::new(
									crate::game_mechanics::ability_dex::FlashFire,
								),
							)),
							(crate::game_mechanics::Battler::new(
								crate::game_mechanics::BattlerUID {
									team_id: crate::game_mechanics::TeamID::Ally,
									battler_number: crate::game_mechanics::monster::BattlerNumber::Second,
								},
								false,
								crate::game_mechanics::monster::Monster::new(
									crate::game_mechanics::monster_dex::Torchic,
									"Sapphire",
								),
								crate::game_mechanics::move_::MoveSet::new(
									vec![
										(crate::game_mechanics::move_::Move::new(
											crate::game_mechanics::move_dex::Scratch,
										)),
										(crate::game_mechanics::move_::Move::new(
											crate::game_mechanics::move_dex::Ember,
										)),
									],
								),
								crate::game_mechanics::ability::Ability::new(
									crate::game_mechanics::ability_dex::FlashFire,
								),
							)),
							(crate::game_mechanics::Battler::new(
								crate::game_mechanics::BattlerUID {
									team_id: crate::game_mechanics::TeamID::Ally,
									battler_number: crate::game_mechanics::monster::BattlerNumber::Third,
								},
								false,
								crate::game_mechanics::monster::Monster::new(
									crate::game_mechanics::monster_dex::Torchic,
									"Emerald",
								),
								crate::game_mechanics::move_::MoveSet::new(
									vec![
										(crate::game_mechanics::move_::Move::new(
											crate::game_mechanics::move_dex::Scratch,
										)),
										(crate::game_mechanics::move_::Move::new(
											crate::game_mechanics::move_dex::Ember,
										)),
									],
								),
								crate::game_mechanics::ability::Ability::new(
									crate::game_mechanics::ability_dex::FlashFire,
								),
							)),
						],
					
				),
				crate::game_mechanics::BattlerTeam::new(
					vec![
						(crate::game_mechanics::Battler::new(
							crate::game_mechanics::BattlerUID {
								team_id: crate::game_mechanics::TeamID::Opponent,
								battler_number: crate::game_mechanics::monster::BattlerNumber::First,
							},
							true,
							crate::game_mechanics::monster::Monster::new(
								crate::game_mechanics::monster_dex::Torchic,
								"Cheerio",
							),
							crate::game_mechanics::move_::MoveSet::new(
								vec![
									(crate::game_mechanics::move_::Move::new(
										crate::game_mechanics::move_dex::Scratch,
									)),
									(crate::game_mechanics::move_::Move::new(
										crate::game_mechanics::move_dex::Ember,
									)),
								],
							),
							crate::game_mechanics::ability::Ability::new(
								crate::game_mechanics::ability_dex::FlashFire,
							),
						))
					],
				),
			),
		);
	}
}

#[cfg(test)]
mod bcontext {
    use bcontext_macro::bcontext_internal;

    use crate::{BattleContext, prng::{Lcrng, self}, InBattleEvent, EventHandlerInfo, EventHandlerFilters, Battle, BattlerUID, TeamID, BattlerNumber};


	#[test]
	fn test_priority_sorting_deterministic() {
		let mut result = [Vec::new(), Vec::new()];
		for i in 0..=1 {
			let test_bcontext = bcontext_internal!(
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

			let mut prng = Lcrng::new(prng::seed_from_time_now());

			let event_handler_set_plus_info = test_bcontext.event_handler_sets_plus_info();
			use crate::event::event_dex::OnTryMove;
			let mut unwrapped_event_handler_plus_info = event_handler_set_plus_info
				.iter()
				.filter_map(|event_handler_set_info| {
					if let Some(handler) =
						OnTryMove.corresponding_handler(&event_handler_set_info.event_handler_set)
					{
						Some(EventHandlerInfo {
							event_handler: handler,
							owner_uid: event_handler_set_info.owner_uid,
							activation_order: event_handler_set_info.activation_order,
							filters: EventHandlerFilters::default(),
						})
					} else {
						None
					}
				})
				.collect::<Vec<_>>();

			Battle::priority_sort::<EventHandlerInfo<bool>>(
				&mut prng,
				&mut unwrapped_event_handler_plus_info,
				&mut |it| it.activation_order,
			);

			result[i] = unwrapped_event_handler_plus_info
				.into_iter()
				.map(|event_handler_info| {
					test_bcontext
						.monster(event_handler_info.owner_uid)
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
	fn test_event_filtering_for_event_sources() {
		let test_bcontext = bcontext_internal!(
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

		let passed_filter = test_bcontext.filter_event_handlers(
			BattlerUID {
				team_id: TeamID::Ally,
				battler_number: BattlerNumber::First,
			},
			BattlerUID {
				team_id: TeamID::Opponent,
				battler_number: BattlerNumber::First,
			},
			EventHandlerFilters::default(),
		);
		assert!(passed_filter);
	}

	#[test]
	fn test_priority_sorting_with_speed_ties() {
		let mut result = [Vec::new(), Vec::new()];
		for i in 0..=1 {
			let test_bcontext = bcontext_internal!(
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
			let mut prng = Lcrng::new(i as u64);

			let event_handler_set_plus_info = test_bcontext.event_handler_sets_plus_info();
			use crate::event::event_dex::OnTryMove;
			let mut unwrapped_event_handler_plus_info = event_handler_set_plus_info
				.iter()
				.filter_map(|event_handler_set_info| {
					if let Some(handler) =
						OnTryMove.corresponding_handler(&event_handler_set_info.event_handler_set)
					{
						Some(EventHandlerInfo {
							event_handler: handler,
							owner_uid: event_handler_set_info.owner_uid,
							activation_order: event_handler_set_info.activation_order,
							filters: EventHandlerFilters::default(),
						})
					} else {
						None
					}
				})
				.collect::<Vec<_>>();

			Battle::priority_sort::<EventHandlerInfo<bool>>(
				&mut prng,
				&mut unwrapped_event_handler_plus_info,
				&mut |it| it.activation_order,
			);

			result[i] = unwrapped_event_handler_plus_info
				.into_iter()
				.map(|event_handler_info| {
					test_bcontext
						.monster(event_handler_info.owner_uid)
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
	fn test_display_battle_context() {
		let test_bcontext = bcontext_internal!(
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
	}
}

#[cfg(test)]
mod event {

	#[test]
	fn test_print_event_handler_set() {
		use crate::ability_dex::FlashFire;
		println!("{:?}", FlashFire.event_handlers);
	}
}

#[cfg(test)]
mod prng {
    use std::time;

    use crate::prng::*;

	#[test]
	fn test_prng_percentage_chance() {
		let mut lcrng = Lcrng::new(seed_from_time_now());
		let mut dist = [0u64; 100];
		for _ in 0..=10_000_000 {
			let n = lcrng.generate_number_in_range(0..=99) as usize;
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
		let mut lcrng_1 = Lcrng::new(seed);
		let mut lcrng_2 = Lcrng::new(seed);
		for i in 0..10_000 {
			let generated_number_1 = lcrng_1.generate_number_in_range(0..=u16::MAX-1);
			let generated_number_2 = lcrng_2.generate_number_in_range(0..=u16::MAX-1);
			assert_eq!(generated_number_1, generated_number_2, "iteration {}", i);
		}
	}

	#[test]
	fn test_prng_chance() {
		let mut lcrng = Lcrng::new(
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
		let avg_probability_deviation = f64::floor(avg_probability_deviation * 100_000.0) / 100_000.0;
		println!(
			"Average probability of LCRNG is off by {}% (threshold is at 0.005%)",
			avg_probability_deviation
		);
		assert!(avg_probability_deviation < 5.0e-3);
	}
}