use monsim::*;

mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::*;
use monster_dex::*;
use move_dex::*;

fn main() -> MonsimResult<()> {
    let battle = BattleState::spawn()
        .add_ally_team(
            MonsterTeam::spawn()
                .add_monster(
                    Dandyleo.spawn(
                        (
                            Scratch.spawn(),
                            Some(BulletSeed.spawn()),
                            None,
                            None,
                        ), 
                        FlashFire.spawn()
                    )
                    .with_nickname("Clover")
                )
                .add_monster(
                    Zombler.spawn(
                        (
                            Tackle.spawn()
                                .with_power_points(23),
                            Some(Growl.spawn()),
                            Some(DragonDance.spawn()),
                            None
                        ),
                        Spiteful.spawn()
                    )
                    .with_nickname("Rick")
                )
                .add_monster(
                    Squirecoal.spawn(
                        (
                            Ember.spawn(),
                            Some(Growl.spawn()),
                            Some(Scratch.spawn()),
                            None
                        ),
                        FlashFire.spawn()
                    )
                    .with_nickname("Lancelot")
                )
        )
        .add_opponent_team(
            MonsterTeam::spawn()
                .add_monster(
                    Merkey.spawn(
                        (
                            Bubble.spawn(),
                            Some(Tackle.spawn()),
                            None,
                            None
                        ),
                        FlashFire.spawn()
                    )
                    .with_nickname("Shrimp")        
                )
                .add_monster(
                    Zombler.spawn(
                        (
                            Tackle.spawn()
                                .with_power_points(23),
                            Some(Growl.spawn()),
                            Some(DragonDance.spawn()),
                            None
                        ),
                        FlashFire.spawn()
                    )
                    .with_nickname("Cordy")
                )
                .add_monster(
                    Squirecoal.spawn(
                        (
                            Ember.spawn(),
                            Some(Growl.spawn()),
                            Some(Scratch.spawn()),
                            None
                        ),
                        FlashFire.spawn()
                    )
                    .with_nickname("Epona")
                )
                .add_monster(
                    Merkey.spawn(
                        (
                            Growl.spawn(),
                            Some(Tackle.spawn()),
                            None,
                            None
                        ),
                        FlashFire.spawn()
                    )        
                )
        )
        .with_format(BattleFormat::Triple)
        .build();

    println!("{:?}", battle.format());

    #[cfg(feature="macro")]
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
