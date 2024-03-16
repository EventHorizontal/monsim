
pub mod tui;
pub mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

use tui::TuiResult;
use monsim_utils::Nothing;
use sim::Battle;

pub fn run(battle: Battle) -> TuiResult<Nothing> {
    cli::run(battle)
}