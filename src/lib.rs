
pub mod tui;
pub mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

use tui::TuiResult;
use monsim_utils::Nothing;
use sim::BattleState;

pub fn run(battle: BattleState) -> TuiResult<Nothing> {
    cli::run(battle)
}