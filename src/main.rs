use monsim::*;
mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::{FlashFire, WaterAbsorb};
use monster_dex::{Drifloon, Mudkip, Torchic, Treecko};
use move_dex::{Bubble, Ember, Growl, Scratch, Tackle};
use monsim_utils::Nothing;

fn main() -> MonsimResult<Nothing> {
    let battle = BattleState::empty()
        .with_ally_team(
            [
                Monster::of_species(&Drifloon)
                    .build()
            ]
        )
        .build();

    monsim::run(battle)
}
