use monsim::{app::{self, AppResult}, sim::*};
mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::{FlashFire, WaterAbsorb};
use monster_dex::{Drifloon, Mudkip, Torchic, Treecko};
use move_dex::{Bubble, Ember, Growl, Scratch, Tackle};
use utils::Nothing;

fn main() -> AppResult<Nothing> {
    let battle = build_battle!(
        {
            Allies: BattlerTeam {
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
                    FlashFire: Ability,
                },
                Treecko: Monster = "Emerald" {
                    Scratch: Move,
                    Ember: Move,
                    WaterAbsorb: Ability,
                },
            },
            Opponents: BattlerTeam {
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
    app::run(battle)
}
