#![allow(clippy::let_and_return)]

#[cfg(feature = "monsim_tui")]
pub(crate) mod tui;
pub(crate) mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

use std::error::Error;
pub use sim::*;
#[cfg(feature = "monsim_tui")]
pub use tui::run as run_tui;
pub use cli::Cli;

pub type MonsimResult<S> = Result<S, Box<dyn Error>>;

pub fn run(battle: BattleState) -> MonsimResult<Nothing> {
    let simulator = BattleSimulator::init(battle);
    let cli = Cli::new();
    simulator.simulate(cli)?;
    Ok(NOTHING)
}