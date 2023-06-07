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
                    FlashFire: Ability,
                },
            },
            Opponents: BattlerTeam {
                Drifloon: Monster {
                    Scratch: Move,
                    Ember: Move,
                    FlashFire: Ability,
                },
            }
        }
    ));
    app::run(battle)
}
