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
                mon Torchic "Ruby" {
                    mov Ember,
                    mov Scratch,
                    mov Growl,
                    mov Bubble,
                    abl FlashFire,
                },
                mon Mudkip "Sapphire" {
                    mov Tackle,
                    mov Bubble,
                    abl FlashFire,
                },
                mon Treecko "Emerald" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            },
            OpponentTeam {
                mon Drifloon "Cheerio" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            }
        }
    ));
    app::run(battle)
}
