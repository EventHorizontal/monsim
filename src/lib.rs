#![allow(clippy::let_and_return)]

pub(crate) mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;
#[cfg(feature = "monsim_tui")]
pub(crate) mod tui;

pub use cli::Cli;
pub use sim::*;
use std::error::Error;
#[cfg(feature = "monsim_tui")]
pub use tui::run as run_tui;

pub type MonsimResult<S> = Result<S, Box<dyn Error>>;

pub fn run(battle: Battle) -> MonsimResult<Nothing> {
    let simulator = BattleSimulator::init(battle);
    let cli = Cli::new();
    simulator.simulate(cli)?;
    Ok(NOTHING)
}
