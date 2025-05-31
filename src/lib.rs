//! ### Event System
//! Events are an integral part of the `monsim` engine, they enable the engine to model any reactive game mechanics,
//! such as abilities or items. An example would be the item *Life Orb*, which reacts to the `on_calculate_attack_stat`
//! event, by raising the attack by 50%. It also reacts to the `on_move_used` event, by draining 10% of the user's max
//! HP.
//!
//! Each Event has a *broadcaster* and zero or more *receivers*. The broadcaster is responsible for emitting or triggering
//! the Event, and then each receiver returns an `EventHandler` that contains a callback function of the appropriate
//! type and some extra information about how and when to activate it, most prominently `EventFilteringOptions`. This is
//! then wrapped into an `OwnedEventHandler` that contains additional information about the owner of the EventHandler (i.e
//! the Monster whose EventHandler it is). The `EventDispatcher` is responsible for collecting, filtering and calling all
//! the callbacks of the appropriate EventHandlers.
//!
//! An Event is broadcasted during the turn-loop for two major reasons:
//! 1. To test if there are mechanics that forbid the next action, or alter it. These events are associated with
//!		functions of the form `on_try_<something>`. A reactive EventHandler may choose to disable this. Think moves like
//!		`Embargo` which prevents item use.
//! 2. To inform the entities in the battle that something specific happened. These events are associated with
//!		functions of the form `on_<something>_happened`. A reactive EventHandler may choose to do something every time
//!		that specific thing happens, or only if further conditions are satisfied. `Passho Berry` reacts to the Event
//!		`on_move_used` when used by an opponent, but only if the move is water-type and super-effective, which it then checks
//!		manually.
//!
//! The EventHandler returns a value, which tells the broadcaster how to modify the logic being evaluated. With the Life Orb
//! example, it returned a new value for the attack stat to be used when attacking. What kind of value an EventHandler returns
//! is decided by the Event it responds to. The `on_calculate_attack_stat` Event expects a `u16` - the modified attack stat.
//! Note that Life Orb may choose to return the original attack stat, which would correspond to having no effect. This is
//! desirable when an mechanic wants to affect the simulation only if certain conditions are met, it then returns the original
//! value when the condition is not met.
//!
//! The Event Dispatcher folds the return values of all the EventHandlers it collected from the Battle, and then the return
//! value is returned to the broadcaster. The execution may be short-circuited if a special value, decided by the broadcaster,
//! is obtained. Certain Events also require the specification of a default value to return if there happens (as it often does)
//! that there are no EventHandlers for that particular Event at the moment. For "trial" events, which encapsulate checking if
//! some action will be successful, have always have a default value of `Outcome::Success<()>` which means they succeed by default,
//! as would be expected.

#![allow(clippy::let_and_return)]

pub(crate) mod cli;
#[cfg(feature = "debug")]
pub mod debug;
pub mod sim;
#[cfg(feature = "monsim_tui")]
pub(crate) mod tui;

pub use cli::Cli;
pub use sim::*;
use std::error::Error;
#[cfg(feature = "monsim_tui")]
pub use tui::run as run_tui;

pub use monsim_utils::{not, Nothing, NOTHING};

pub type MonsimResult<S> = Result<S, Box<dyn Error>>;

pub fn run(battle: Battle) -> MonsimResult<Nothing> {
    let simulator = BattleSimulator::init(battle);
    let cli = Cli::new();
    simulator.simulate(cli)?;
    Ok(NOTHING)
}
