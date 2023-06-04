use monsim::{
    app::{self, MonsimResult},
    sim::*,
};
mod ability_dex;
mod monster_dex;
mod move_dex;

use ability_dex::FlashFire;
use monster_dex::{Drifloon, Mudkip, Torchic, Treecko};
use move_dex::{Bubble, Ember, Growl, Scratch, Tackle};

fn main() -> MonsimResult {
    let battle = BattleSimulator::new(build_battle!(
        {
            AllyTeam {
                let Torchic: Monster = "Ruby" {
                    Ember: Move,
                    Scratch: Move,
                    Growl: Move,
                    Bubble: Move,
                    FlashFire: Ability,
                },
                let Mudkip: Monster = "Sapphire" {
                    Tackle: Move,
                    Bubble: Move,
                    FlashFire: Ability,
                },
                let Treecko: Monster = "Emerald" {
                    Scratch: Move,
                    Ember: Move,
                    FlashFire: Ability,
                },
            },
            OpponentTeam {
                let Drifloon: Monster {
                    Scratch: Move,
                    Ember: Move,
                    FlashFire: Ability,
                },
            }
        }
    ));
    app::run(battle)
}
