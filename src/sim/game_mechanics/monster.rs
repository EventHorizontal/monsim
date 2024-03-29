use core::{fmt::Debug, panic};
use std::{fmt::{Display, Formatter}, ops::{Index, IndexMut}};

use monsim_utils::MaxSizedVec;

use super::{Ability, MoveNumber, MoveSet, MoveUID, TeamUID };
use crate::{sim::{event::OwnedEventHandlerDeck, ActivationOrder, EventFilteringOptions, EventHandlerDeck, Type}, Move};

#[derive(Debug, Clone)]
pub struct Monster {
    pub uid: MonsterUID,
    pub(crate) nickname: Option<&'static str>,
    pub level: u16,
    pub max_health: u16,
    pub nature: MonsterNature,
    pub stats: StatSet,
    pub stat_modifiers: StatModifierSet,
    pub is_fainted: bool,
    pub current_health: u16,
    pub species: &'static MonsterSpecies,
    pub moveset: MaxSizedVec<Move, 4>,
    pub ability: Ability,
}

impl PartialEq for Monster {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Monster {}

impl Display for Monster {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        if let Some(nickname) = self.nickname {
            out.push_str(
                format![
                    "{} the {} ({}) [HP: {}/{}]\n\t│\t│\n",
                    nickname, self.species.name, self.uid, self.current_health, self.max_health
                ]
                .as_str(),
            );
        } else {
            out.push_str(
                format![
                    "{} ({}) [HP: {}/{}]\n\t│\t│\n",
                    self.species.name, self.uid, self.current_health, self.max_health
                ]
                .as_str(),
            );
        }

        let number_of_effects = self.moveset.count();

        out.push_str("\t│\t├── ");
        out.push_str(format!["type {:?}/{:?} \n", self.species.primary_type, self.species.secondary_type].as_str());

        out.push_str("\t│\t├── ");
        out.push_str(format!["abl {}\n", self.ability.species.name].as_str());

        for (i, move_) in self.moveset.into_iter().enumerate() {
            if i < number_of_effects - 1 {
                out.push_str("\t│\t├── ");
            } else {
                out.push_str("\t│\t└── ");
            }
            out.push_str(format!["mov {}\n", move_.species.name].as_str());
        }

        write!(f, "{}", out)
    }
}

impl Monster {
    pub(crate) fn new(uid: MonsterUID, species: &'static MonsterSpecies, nickname: Option<&'static str>, moveset: MaxSizedVec<Move, 4>, ability: Ability) -> Self {
        let level = 50;
        // TODO: EVs and IVs are hardcoded for now. Decide what to do with this later.
        let iv_in_stat = 31;
        let ev_in_stat = 252;
        // In-game hp-stat determination formula
        let health_stat = ((2 * species.base_stats[Stat::Hp] + iv_in_stat + (ev_in_stat / 4)) * level) / 100 + level + 10;
        let nature = MonsterNature::Serious;

        // In-game non-hp-stat determination formula
        let get_non_hp_stat = |stat: Stat| -> u16 {
            // TODO: EVs and IVs are hardcoded for now. Decide what to do with this later.
            let iv_in_stat = 31;
            let ev_in_stat = 252;
            let mut out = ((2 * species.base_stats[stat] + iv_in_stat + (ev_in_stat / 4)) * level) / 100 + 5;
            out = f64::floor(out as f64 * nature[stat]) as u16;
            out
        };
        
        Monster {
            uid,
            nickname,
            level,
            max_health: health_stat,
            nature,
            current_health: health_stat,
            is_fainted: false,
            species,
            moveset,
            ability,
            stats: StatSet {
                hp: health_stat,
                att: get_non_hp_stat(Stat::PhysicalAttack),
                def: get_non_hp_stat(Stat::PhysicalDefense),
                spa: get_non_hp_stat(Stat::SpecialAttack),
                spd: get_non_hp_stat(Stat::SpecialDefense),
                spe: get_non_hp_stat(Stat::Speed),
            },
            stat_modifiers: StatModifierSet {
                att: 0,
                def: 0,
                spa: 0,
                spd: 0,
                spe: 0,
            },
        }
    }

    pub fn name(&self) -> String {
        if let Some(nickname) = self.nickname {
            nickname.to_owned()
        } else {
            self.species.name.to_owned()
        }
    }

    pub fn full_name(&self) -> String {
        if let Some(nickname) = self.nickname {
            format!["{} the {}", nickname, self.species.name]
        } else {
            self.species.name.to_string()
        }
    }

    pub fn is_type(&self, test_type_: Type) -> bool {
        self.species.primary_type == test_type_ || self.species.secondary_type == Some(test_type_)
    }

    pub fn ability_event_handler_deck_instance(&self) -> OwnedEventHandlerDeck {
        let activation_order = ActivationOrder {
            priority: 0,
            speed: self.stats[Stat::Speed],
            order: self.ability.species.order,
        };
        OwnedEventHandlerDeck {
            event_handler_deck: self.ability.event_handler_deck(),
            owner_uid: self.uid,
            activation_order,
            filtering_options: EventFilteringOptions::default(),
        }
    }

    pub fn moveset_event_handler_deck_instances(&self, uid: MonsterUID) -> Vec<OwnedEventHandlerDeck> {
        self.moveset
            .iter()
            .map(|it| OwnedEventHandlerDeck {
                event_handler_deck: &it.species.event_handler_deck,
                owner_uid: uid,
                activation_order: ActivationOrder {
                    priority: it.species.priority,
                    speed: self.stats[Stat::Speed],
                    order: 0,
                },
                filtering_options: EventFilteringOptions::default(),
            })
            .collect::<Vec<_>>()
    }

    pub fn event_handler_deck_instances(&self) -> Vec<OwnedEventHandlerDeck> {
        let activation_order = ActivationOrder {
            priority: 0,
            speed: self.stats[Stat::Speed],
            order: 0,
        };
        let monster_event_handler_deck_instance = OwnedEventHandlerDeck {
            event_handler_deck: self.species.event_handler_deck,
            owner_uid: self.uid,
            activation_order,
            filtering_options: EventFilteringOptions::default(),
        };
        let mut out = vec![monster_event_handler_deck_instance];
        out.append(&mut self.moveset_event_handler_deck_instances(self.uid));
        out.push(self.ability_event_handler_deck_instance());
        out
    }

    pub(crate) fn move_uids(&self) -> Vec<MoveUID> {
        self.moveset
            .iter()
            .enumerate()
            .map(|(idx, _)| MoveUID {
                owner_uid: self.uid,
                move_number: MoveNumber::from(idx),
            })
            .collect()
    }

    pub fn status_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format![
            "{} ({}) [HP: {}/{}]\n",
            self.full_name(), self.uid, self.current_health, self.max_health
        ]);
        out
    }
}

#[derive(Clone, Copy)]
pub struct MonsterSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub primary_type: Type,
    pub secondary_type: Option<Type>,
    pub base_stats: StatSet,
    pub event_handler_deck: &'static EventHandlerDeck,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonsterUID {
    pub team_uid: TeamUID,
    pub monster_number: MonsterNumber,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonsterNumber {
    #[default]
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
}

pub const ALLY_1: MonsterUID = MonsterUID {
    team_uid: TeamUID::Allies,
    monster_number: MonsterNumber::_1,
};
pub const ALLY_2: MonsterUID = MonsterUID {
    team_uid: TeamUID::Allies,
    monster_number: MonsterNumber::_2,
};
pub const ALLY_3: MonsterUID = MonsterUID {
    team_uid: TeamUID::Allies,
    monster_number: MonsterNumber::_3,
};
pub const ALLY_4: MonsterUID = MonsterUID {
    team_uid: TeamUID::Allies,
    monster_number: MonsterNumber::_4,
};
pub const ALLY_5: MonsterUID = MonsterUID {
    team_uid: TeamUID::Allies,
    monster_number: MonsterNumber::_5,
};
pub const ALLY_6: MonsterUID = MonsterUID {
    team_uid: TeamUID::Allies,
    monster_number: MonsterNumber::_6,
};

pub const OPPONENT_1: MonsterUID = MonsterUID {
    team_uid: TeamUID::Opponents,
    monster_number: MonsterNumber::_1,
};
pub const OPPONENT_2: MonsterUID = MonsterUID {
    team_uid: TeamUID::Opponents,
    monster_number: MonsterNumber::_2,
};
pub const OPPONENT_3: MonsterUID = MonsterUID {
    team_uid: TeamUID::Opponents,
    monster_number: MonsterNumber::_3,
};
pub const OPPONENT_4: MonsterUID = MonsterUID {
    team_uid: TeamUID::Opponents,
    monster_number: MonsterNumber::_4,
};
pub const OPPONENT_5: MonsterUID = MonsterUID {
    team_uid: TeamUID::Opponents,
    monster_number: MonsterNumber::_5,
};
pub const OPPONENT_6: MonsterUID = MonsterUID {
    team_uid: TeamUID::Opponents,
    monster_number: MonsterNumber::_6,
};

impl Display for MonsterUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.team_uid {
            TeamUID::Allies => match self.monster_number {
                MonsterNumber::_1 => write!(f, "First Ally"),
                MonsterNumber::_2 => write!(f, "Second Ally"),
                MonsterNumber::_3 => write!(f, "Third Ally"),
                MonsterNumber::_4 => write!(f, "Fourth Ally"),
                MonsterNumber::_5 => write!(f, "Fifth Ally"),
                MonsterNumber::_6 => write!(f, "Sixth Ally"),
            },
            TeamUID::Opponents => match self.monster_number {
                MonsterNumber::_1 => write!(f, "First Opponent"),
                MonsterNumber::_2 => write!(f, "Second Opponent"),
                MonsterNumber::_3 => write!(f, "Third Opponent"),
                MonsterNumber::_4 => write!(f, "Fourth Opponent"),
                MonsterNumber::_5 => write!(f, "Fifth Opponent"),
                MonsterNumber::_6 => write!(f, "Sixth Opponent"),
            },
        }
    }
}

const MONSTER_DEFAULTS: MonsterSpecies = MonsterSpecies {
    dex_number: 000,
    name: "Unnamed",
    primary_type: Type::Normal,
    secondary_type: None,
    base_stats: StatSet::new(0, 0, 0, 0, 0, 0),
    event_handler_deck: &EventHandlerDeck::const_default(),
};

impl MonsterSpecies {
    pub const fn const_default() -> Self {
        MONSTER_DEFAULTS
    }
}

impl From<usize> for MonsterNumber {
    fn from(value: usize) -> Self {
        match value {
            0 => MonsterNumber::_1,
            1 => MonsterNumber::_2,
            2 => MonsterNumber::_3,
            3 => MonsterNumber::_4,
            4 => MonsterNumber::_5,
            5 => MonsterNumber::_6,
            _ => panic!("MonsterNumber can only be formed from usize 0 to 5."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Hp,
    PhysicalAttack,
    PhysicalDefense,
    SpecialAttack,
    SpecialDefense,
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatSet {
    hp: u16,
    att: u16,
    def: u16,
    spa: u16,
    spd: u16,
    spe: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatModifierSet {
    att: i8,
    def: i8,
    spa: i8,
    spd: i8,
    spe: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterNature {
    /// Neutral (+-Attack)
    Hardy,
    /// +Attack, -Defense
    Lonely,
    /// +Attack, -Speed
    Brave,
    /// +Attack, -Special Attack
    Adamant,
    /// +Attack, -Special Defense
    Naughty,

    /// Neutral (+-Defense)
    Docile,
    /// +Defense, -Attack
    Bold,
    /// +Defense, -Speed
    Relaxed,
    /// +Defense, -Special Attack
    Impish, // - Special Attack
    /// +Defense, -Special Defense
    Lax, // - Special Defense

    /// Neutral (+-Speed)
    Serious,
    /// +Speed, -Attack
    Timid,
    /// +Speed, -Defense
    Hasty,
    /// +Speed, -Special Attack
    Jolly,
    /// +Speed, -Special Defense
    Naive,

    /// Neutral (+-Special Attack)
    Bashful,
    /// +Special Attack, -Attack
    Modest,
    /// +Special Attack, -Defense
    Mild,
    /// +Special Attack, -Speed
    Quiet,
    /// +Special Attack, -Special Defense
    Rash,

    /// Neutral (+-Special Defense)
    Quirky,
    /// +Special Defense, -Attack
    Calm,
    /// +Special Defense, -Defense
    Gentle,
    /// +SpecialDefense, -Speed
    Sassy,
    /// +SpecialDefense, -Special Attack
    Careful,
}

impl Debug for MonsterSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:03} {},\n\t type: {:?}/{:?}",
            self.dex_number, self.name, self.primary_type, self.secondary_type
        )
    }
}

impl PartialEq for MonsterSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for MonsterSpecies {}

impl Index<Stat> for StatSet {
    type Output = u16;

    fn index(&self, index: Stat) -> &Self::Output {
        match index {
            Stat::Hp => &self.hp,
            Stat::PhysicalAttack => &self.att,
            Stat::PhysicalDefense => &self.def,
            Stat::SpecialAttack => &self.spa,
            Stat::SpecialDefense => &self.spd,
            Stat::Speed => &self.spe,
        }
    }
}

impl StatSet {
    pub const fn new(hp: u16, att: u16, def: u16, spa: u16, spd: u16, spe: u16) -> Self {
        Self { hp, att, def, spa, spd, spe }
    }
}

impl Index<Stat> for StatModifierSet {
    type Output = i8;

    fn index(&self, index: Stat) -> &Self::Output {
        match index {
            Stat::Hp => panic!("Error: StatModifierSet does not have an HP entry and so cannot be indexed by Stat::HP."),
            Stat::PhysicalAttack => &self.att,
            Stat::PhysicalDefense => &self.def,
            Stat::SpecialAttack => &self.spa,
            Stat::SpecialDefense => &self.spd,
            Stat::Speed => &self.spe,
        }
    }
}

impl IndexMut<Stat> for StatModifierSet {
    fn index_mut(&mut self, index: Stat) -> &mut Self::Output {
        match index {
            Stat::Hp => panic!("Error: StatModifierSet does not have an HP entry and so cannot be indexed by Stat::HP."),
            Stat::PhysicalAttack => &mut self.att,
            Stat::PhysicalDefense => &mut self.def,
            Stat::SpecialAttack => &mut self.spa,
            Stat::SpecialDefense => &mut self.spd,
            Stat::Speed => &mut self.spe,
        }
    }
}

impl StatModifierSet {
    pub const fn new(att: i8, def: i8, spa: i8, spd: i8, spe: i8) -> Self {
        Self { att, def, spa, spd, spe }
    }

    pub fn raise_stat(&mut self, stat: Stat, by_stages: u8) -> u8 {
        // So far stat rises have never been larger than +3, but this might be removed in the
        // interest of innovation.
        assert!(by_stages <= 3);
        let effective_stages = (6 - self[stat]).min(by_stages as i8);
        assert!(effective_stages >= 0);
        self[stat] += effective_stages;
        effective_stages as u8
    }

    pub fn lower_stat(&mut self, stat: Stat, by_stages: u8) -> u8 {
        // So far stat drops have never been larger than -3, but this might be removed in the
        // interest of innovation.
        assert!(by_stages <= 3);
        let effective_stages = (6 + self[stat]).min(by_stages as i8);
        assert!(effective_stages >= 0);
        self[stat] -= effective_stages;
        effective_stages as u8
    }
}

impl Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stat::Hp => write!(f, "HP"),
            Stat::PhysicalAttack => write!(f, "Attack"),
            Stat::PhysicalDefense => write!(f, "Defense"),
            Stat::SpecialAttack => write!(f, "Special Attack"),
            Stat::SpecialDefense => write!(f, "Special Defense"),
            Stat::Speed => write!(f, "Speed"),
        }
    }
}

impl Index<Stat> for MonsterNature {
    type Output = f64;

    fn index(&self, index: Stat) -> &Self::Output {
        const LOWERED: f64 = 0.9;
        const NEUTRAL: f64 = 1.0;
        const RAISED: f64 = 1.1;

        match self {
            MonsterNature::Hardy => &NEUTRAL,

            MonsterNature::Lonely => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Brave => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Adamant => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Naughty => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::SpecialDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Docile => &NEUTRAL,

            MonsterNature::Bold => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Relaxed => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Impish => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Lax => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Serious => &NEUTRAL,

            MonsterNature::Timid => match index {
                Stat::Speed => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Hasty => match index {
                Stat::Speed => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Jolly => match index {
                Stat::Speed => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Naive => match index {
                Stat::Speed => &RAISED,
                Stat::SpecialDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Bashful => &NEUTRAL,

            MonsterNature::Modest => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Mild => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Quiet => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Rash => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::SpecialDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Quirky => &NEUTRAL,

            MonsterNature::Calm => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Gentle => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Sassy => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Careful => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },
        }
    }
}
