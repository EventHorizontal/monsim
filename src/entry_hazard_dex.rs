#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{EntryHazardDexEntry, EntryHazardSpecies, EventListener, Nothing};

pub const PointedStones: EntryHazardSpecies = EntryHazardSpecies::from_dex_entry(EntryHazardDexEntry {
    dex_number: 001,
    name: "Pointed Stones",
    event_listener: &PointedStonesEventListener,
    on_start_message: "Pointed rocks were scattered around the opponents feet!",
    on_clear_message: "The pointed rocks were scattered away.",
});

struct PointedStonesEventListener;

impl EventListener<Nothing, Nothing> for PointedStonesEventListener {}
