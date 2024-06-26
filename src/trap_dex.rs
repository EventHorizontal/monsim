#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{EventListener, Nothing, TrapDexEntry, TrapSpecies};

pub const PointedStones: TrapSpecies = TrapSpecies::from_dex_entry(TrapDexEntry {
    dex_number: 001,
    name: "Pointed Stones",
    event_listener: &PointedStonesEventListener,
    on_start_message: "Pointed rocks were scattered around the opponents feet!",
    on_clear_message: "The pointed rocks were scattered away.",
});

struct PointedStonesEventListener;

impl EventListener<Nothing, Nothing> for PointedStonesEventListener {}
