use monsim::*;
mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::{FlashFire, WaterAbsorb};
use monster_dex::{Drifloon, Mudkip, Torchic, Treecko};
use move_dex::{Bubble, Ember, Growl, Scratch, Tackle};
use monsim_utils::{IntoAlly, IntoOpponent, Nothing};

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
    monsim::run(battle)
}
