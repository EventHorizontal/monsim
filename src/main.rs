#![allow(clippy::let_and_return)]

use monsim::*;

mod ability_dex;
mod item_dex;
mod monster_dex;
mod move_dex;
mod status_dex;
mod terrain_dex;
mod trap_dex;
mod weather_dex;

use ability_dex::*;
use item_dex::*;
use monster_dex::*;
use move_dex::*;
// use terrain_dex::*;
use trap_dex::*;
use weather_dex::*;

fn main() -> MonsimResult<()> {
    #[cfg(feature = "debug")]
    std::env::set_var("RUST_BACKTRACE", "1");

    let battle = Battle::spawn()
        .with_ally_team(
            MonsterTeam::spawn()
                .with_monster(
                    Monstrossive
                        .spawn((Ember.spawn(), Some(Growl.spawn()), Some(Confusion.spawn()), Some(Spikes.spawn())))
                        .with_nickname("Clover")
                        .with_item(LifeOrb.spawn())
                        .with_hitpoints(120),
                )
                .with_monster(
                    Zombler
                        .spawn((StoneEdge.spawn(), Some(Growl.spawn()), Some(DragonDance.spawn()), Some(ShadowBall.spawn())))
                        .with_nickname("Rick"),
                )
                .with_monster(
                    Squirecoal
                        .spawn((Ember.spawn(), Some(Growl.spawn()), Some(ShadowBall.spawn()), Some(Recycle.spawn())))
                        .with_nickname("Lancelot")
                        .with_item(PasshoBerry.spawn())
                        .with_ability(FlashFire.spawn()),
                ),
        )
        .with_opponent_team(
            MonsterTeam::spawn()
                .with_monster(
                    Merkey
                        .spawn((Bubble.spawn(), Some(DoubleTeam.spawn()), Some(Swift.spawn()), Some(Confusion.spawn())))
                        .with_nickname("Shrimp"),
                )
                .with_monster(
                    Zombler
                        .spawn((StoneEdge.spawn(), Some(Growl.spawn()), Some(DragonDance.spawn()), None))
                        .with_nickname("Cordy")
                        .with_hitpoints(20),
                )
                .with_monster(
                    Squirecoal
                        .spawn((Ember.spawn(), Some(Confusion.spawn()), Some(Scratch.spawn()), None))
                        .with_nickname("Epona"),
                ),
        )
        .with_environment(Environment::spawn().with_weather(&HarshSunlight))
        // .with_format(BattleFormat::Triple)
        .build();

    println!("{:?}", battle.format());

    #[cfg(feature = "macros")]
    let _battle2 = battle!(
        team: Opponents
        {
            Merkey: "Blub" {
                moveset: (Bubble, Tackle),
                ability: FlashFire,
            },
            Squirecoal: "Cheep" {
                moveset: (Scratch, Tackle, Growl),
                ability: FlashFire
            }
        },
        team: Allies
        {
            Zombler: "Cheerio" {
                moveset: (Scratch, Ember),
                ability: FlashFire,
            },
            Squirecoal: "Cheep" {
                moveset: (Scratch { power_points: 23 }, Tackle, Growl),
                ability: FlashFire
            }
        },
    );
    monsim::run(battle)
}
