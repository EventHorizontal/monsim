use monsim::*;
mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::{FlashFire, WaterAbsorb};
use monster_dex::{Drifloon, Mudkip, Torchic, Treecko};
use move_dex::{Bubble, Ember, Growl, Scratch, Tackle};
use utils::Nothing;

fn main() -> MonsimResult<Nothing> {
    let battle = battle_state!(
        {
            Allies: MonsterTeam {
                Torchic: Monster = "Ruby" {
                    Ember: Move,
                    Scratch: Move,
                    Growl: Move,
                    Bubble: Move,
                    FlashFire: Ability,
                },
                Mudkip: Monster = "Sapphire" {
                    Tackle: Move,
                    Bubble: Move,
                    WaterAbsorb: Ability,
                },
                Treecko: Monster = "Emerald" {
                    Scratch: Move,
                    Ember: Move,
                    WaterAbsorb: Ability,
                },
            },
            Opponents: MonsterTeam {
                Drifloon: Monster {
                    Scratch: Move,
                    Ember: Move,
                    FlashFire: Ability,
                },
                Mudkip: Monster = "Aquamarine" {
                    Scratch: Move,
                    Bubble: Move,
                    WaterAbsorb: Ability
                }
            }
        }
    );

    monsim::run(battle)
}
