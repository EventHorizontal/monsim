
pub mod app;
pub mod terminal_interface;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

use app::AppResult;
use monsim_utils::Nothing;
use sim::Battle;

pub fn run(battle: Battle) -> AppResult<Nothing> {
    terminal_interface::run(battle)
}