use monsim::*;

mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::*;
use monster_dex::*;
use move_dex::*;

fn main() -> MonsimResult<()> {
    let battle = BattleState::builder()
        .add_ally_team(
            MonsterTeam::builder()
                .add_monster(
                    Monster::of_species(&Drifloon)
                        .add_move(
                            Move::of_species(&Tackle)
                                .with_power_points(23)
                        )
                        .add_move(
                            Move::of_species(&Growl)
                        )
                        .add_ability(&FlashFire)
                        
                )
        )
        .add_opponent_team(
            MonsterTeam::builder()
                .add_monster(
                    Monster::of_species(&Torchic)
                        .add_move(
                            Move::of_species(&Growl)
                        )
                        .add_move(
                            Move::of_species(&Bubble)
                        )
                        .add_ability(&WaterAbsorb)
                        
                )
        )
        .build();

    let _battle2 = battle!(
        team: Opponents
        {
            Mudkip: "Blub" {
                moveset: (Bubble, Tackle),
                ability: FlashFire,
            },
            Torchic: "Cheep" {
                moveset: (Scratch, Tackle, Growl),
                ability: WaterAbsorb
            }
        },
        team: Allies
        {
            Drifloon: "Cheerio" {
                moveset: (Scratch, Ember),
                ability: FlashFire,
            },
            Torchic: "Cheep" {
                moveset: (Scratch { power_points: 23 }, Tackle, Growl),
                ability: WaterAbsorb
            }
        },
    );
    monsim::run(battle)
}
