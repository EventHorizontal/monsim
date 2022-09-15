use crate::battle::entities::{EventHandlerSet, MonType, MonsterSpecies, Stats};

        pub const Shroomish: MonsterSpecies = MonsterSpecies {
            dex: 285,
            name: "Shroomish",
            primary_type: MonType::Grass,
            secondary_type: MonType::None,
            base_stats: Stats {
                hp: 60,
                att: 40,
                def: 60,
                spa: 40,
                spd: 60,
                spe: 35,
            },
            event_handlers: EventHandlerSet::default(),
        };

        pub const Trapinch: MonsterSpecies = MonsterSpecies {
            dex: 328,
            name: "Trapinch",
            primary_type: MonType::Ground,
            secondary_type: MonType::None,
            base_stats: Stats {
                hp: 45,
                att: 100,
                def: 45,
                spa: 45,
                spd: 45,
                spe: 10,
            },
            event_handlers: EventHandlerSet::default(),
        };