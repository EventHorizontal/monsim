#[cfg(features="monsim_tui")]
pub(crate) mod tui;
pub(crate) mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
mod test;

use std::error::Error;
pub use sim::*;
#[cfg(features="monsim_tui")]
pub use tui::run as run_tui;
pub use cli::run as run_cli;

pub type MonsimResult<S> = Result<S, Box<dyn Error>>;

pub fn run(battle: Battle) -> MonsimResult<Nothing> {
    run_cli(battle)
}