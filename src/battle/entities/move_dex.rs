use crate::battle::entities::{EventHandlerSet, MonType, MoveSpecies};

        pub const Tackle: MoveSpecies = MoveSpecies {
            dex: 33,
            name: "Tackle",
            _type: MonType::Normal,
            base_power: 50,
            base_accuracy: 100,
            event_handlers: EventHandlerSet {
                on_calc_move_type: Some(|| -> MonType {
                    // TODO: Tackle temporarily has an event handler for testing purposes. Remove this after the test is over.
                    println!("The move's type changed to Rock!");
                    MonType::Rock
                }),
                ..EventHandlerSet::default()
            },
        };