pub(crate) mod tui;
pub(crate) mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

use std::error::Error;

pub use sim::*;

pub type MonsimResult<S> = Result<S, Box<dyn Error>>;

pub fn run(battle: BattleState) -> MonsimResult<Nothing> {
    cli::run(battle)
}