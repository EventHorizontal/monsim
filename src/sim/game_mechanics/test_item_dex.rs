use monsim_utils::Percent;

use crate::{item::{ItemDexData, ItemKind, ItemSpecies}, source_code_location, EventFilteringOptions, EventHandler, EventHandlerDeck, TargetFlags};


pub const LifeOrb: ItemSpecies = ItemSpecies::from_dex_data(
    ItemDexData {
        dex_number: 001,
        name: "Life Orb",
        kind: ItemKind::Misc,
        event_handlers: EventHandlerDeck {
            on_modify_damage: Some(EventHandler {
                #[cfg(feature = "debug")]
                source_code_location: source_code_location!(),
                response: |_sim, _broadcaster_id, _receiver_id, _, damage| {
                    damage * Percent(130)
                },
            }),
            ..EventHandlerDeck::empty()
        },
        event_filtering_options: EventFilteringOptions {
            allowed_broadcaster_relation_flags: TargetFlags::SELF,
            ..EventFilteringOptions::default()
        },
    }
);