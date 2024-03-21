
// pub mod tui;
pub mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

use std::error::Error;

// use tui::TuiResult;
use monsim_utils::Nothing;
use sim::Battle;

pub(crate) type MonsimResult<T> = Result<T, Box<dyn Error>>;

pub fn run(battle: Battle) -> MonsimResult<Nothing> {
    cli::run(battle)
}