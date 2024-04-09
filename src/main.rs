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
                    Zombler.spawn(
                        (
                            Tackle.spawn()
                                .with_power_points(23),
                            Some(Growl.spawn()),
                            None,
                            None
                        ),
                        FlashFire.spawn()
                    )
                )
        )
        .add_opponent_team(
            MonsterTeam::spawn()
                .add_monster(
                    Merkey.spawn(
                        (
                            Growl.spawn(),
                            Some(Tackle.spawn()),
                            None,
                            None
                        ),
                        WaterAbsorb.spawn()
                    )
                        
                )
        )
        .build();

    let _battle2 = battle!(
        team: Opponents
        {
            Merkey: "Blub" {
                moveset: (Bubble, Tackle),
                ability: FlashFire,
            },
            Squirecoal: "Cheep" {
                moveset: (Scratch, Tackle, Growl),
                ability: WaterAbsorb
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
                ability: WaterAbsorb
            }
        },
    );
    monsim::run(battle)
}
