use monsim::{tui::TuiResult, sim::*};
mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::{FlashFire, WaterAbsorb};
use monster_dex::{Drifloon, Mudkip, Torchic, Treecko};
use move_dex::{Bubble, Ember, Growl, Scratch, Tackle};
use utils::Nothing;

fn main() -> TuiResult<Nothing> {
    let battle = build_battle!(
        {
            Allies: MonsterTeamInternal {
                Torchic: Monster = "Ruby" {
                    Ember: Move,
                    Scratch: Move,
                    Growl: Move,
                    Bubble: Move,
                    FlashFire: AbilityInternal,
                },
                Mudkip: Monster = "Sapphire" {
                    Tackle: Move,
                    Bubble: Move,
                    WaterAbsorb: AbilityInternal,
                },
                Treecko: Monster = "Emerald" {
                    Scratch: Move,
                    Ember: Move,
                    WaterAbsorb: AbilityInternal,
                },
            },
            Opponents: MonsterTeamInternal {
                Drifloon: Monster {
                    Scratch: Move,
                    Ember: Move,
                    FlashFire: AbilityInternal,
                },
                Mudkip: Monster = "Aquamarine" {
                    Scratch: Move,
                    Bubble: Move,
                    WaterAbsorb: AbilityInternal
                }
            }
        }
    );

    monsim::run(battle)
}
