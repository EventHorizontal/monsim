use core::{fmt::Debug, panic};
use std::{fmt::{Display, Formatter}, ops::{Index, IndexMut}};

use monsim_utils::MaxSizedVec;
use tap::Pipe;

use super::{Ability, TeamID};
use crate::{sim::{targetting::{BoardPosition, FieldPosition}, ActivationOrder, EventFilteringOptions, EventHandlerDeck, Type}, Event, Move, OwnedEventHandler};

#[derive(Debug, Clone)]
pub struct Monster {
    pub(crate) id: MonsterID,
    
    pub(crate) nickname: Option<&'static str>,
    pub(crate) effort_values: StatSet,
    pub(crate) current_health: u16,
    pub(crate) individual_values: StatSet,
    pub(crate) level: u16,
    pub(crate) nature: MonsterNature,
    pub(crate) board_position: BoardPosition,
    pub(crate) stat_modifiers: StatModifierSet,
    pub(crate) species: &'static MonsterSpecies,
    
    pub(crate) moveset: MaxSizedVec<Move, 4>,
    pub(crate) ability: Ability,
}

impl PartialEq for Monster {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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
                    nickname, self.species.name, self.id, self.current_health, self.max_health()
                ]
                .as_str(),
            );
        } else {
            out.push_str(
                format![
                    "{} ({}) [HP: {}/{}]\n\t│\t│\n",
                    self.species.name, self.id, self.current_health, self.max_health()
                ]
                .as_str(),
            );
        }

        let number_of_effects = self.moveset.count();

        out.push_str("\t│\t├── ");
        if let Some(secondary_type) = self.species().secondary_type {
            out.push_str(format!["type {:?}/{:?} \n", self.species.primary_type, secondary_type].as_str());
        } else {
            out.push_str(format!["type {:?} \n", self.species.primary_type].as_str());
        }

        out.push_str("\t│\t├── ");
        out.push_str(format!["abl {}\n", self.ability.name()].as_str());

        for (i, move_) in self.moveset.into_iter().enumerate() {
            if i < number_of_effects - 1 {
                out.push_str("\t│\t├── ");
            } else {
                out.push_str("\t│\t└── ");
            }
            out.push_str(format!["mov {}\n", move_.name()].as_str());
        }

        write!(f, "{}", out)
    }
}

impl Monster { // public
    pub fn name(&self) -> String {
        if let Some(nickname) = self.nickname {
            nickname.to_owned()
        } else {
            self.species.name.to_owned()
        }
    }

    pub fn is_type(&self, test_type_: Type) -> bool {
        self.species.primary_type == test_type_ || self.species.secondary_type == Some(test_type_)
    }

    #[inline(always)]
    pub fn max_health(&self) -> u16 {
        // INFO: Unless this turns out to be a bad idea, we just calculate every time its 
        // requested. The only situation I would need to change this is if the formula
        // were to change, which I don't think will happen.
        Monster::calculate_max_health(self.species.base_stat(Stat::Hp), self.individual_values[Stat::Hp], self.effort_values[Stat::Hp], self.level)
    }

    #[inline(always)]
    pub fn stat(&self, stat: Stat) -> u16 {
        match stat {
            Stat::Hp => self.max_health(),
            _ => {
                // TODO: Division is supposed to be floating point here.
                ((2 * self.species.base_stats[stat] + self.individual_values[stat] + (self.effort_values[stat] / 4)) * self.level) / 100 + 5 // * self.nature[stat]
            }
        }
    }
    
    #[inline(always)]
    pub fn current_health(&self) -> u16 {
        self.current_health
    }

    #[inline(always)]
    pub fn nature(&self) -> MonsterNature {
        self.nature
    }

    #[inline(always)]
    pub fn is_fainted(&self) -> bool {
        self.current_health == 0
    }

    #[inline(always)]
    pub fn stat_modifier(&self, stat: Stat) -> i8 {
        self.stat_modifiers[stat]
    }

    #[inline(always)]
    pub fn ability(&self) -> &Ability {
        &self.ability
    }

    #[inline(always)]
    pub fn moveset(&self) -> &MaxSizedVec<Move, 4> {
        &self.moveset
    }
    
    #[inline(always)]
    pub fn species(&self) -> &'static MonsterSpecies {
        self.species
    }

    #[inline(always)]
    pub fn iv_in_stat(&self, stat: Stat) -> u16 {
        self.individual_values[stat]
    }

    #[inline(always)]
    pub fn ev_in_stat(&self, stat: Stat) -> u16 {
        self.effort_values[stat]
    }
    
    pub(crate) fn field_position(&self) -> Option<FieldPosition> {
        match self.board_position {
            BoardPosition::Bench => None,
            BoardPosition::Field(field_position) => Some(field_position),
        }
    }
    
    pub(crate) fn is_active(&self) -> bool {
        matches!(self.board_position, BoardPosition::Field(_))
    }
    
}

impl Monster { // private

    pub(crate) fn calculate_max_health(base_hp: u16, hp_iv: u16, hp_ev: u16, level: u16) -> u16 {
        ((2 * base_hp + hp_iv + (hp_ev / 4)) * level) / 100 + level + 10
    }

    pub(crate) fn ability_event_handler_for<E: Event>(&self, event: E) -> Option<OwnedEventHandler<E>> {
        event.corresponding_handler(self.ability.event_handlers()) 
            .map(|event_handler| { // Add an OwnedEventHandler if an EventHandler exists.
                OwnedEventHandler {
                    event_handler,
                    owner_id: self.id,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: self.stat(Stat::Speed),
                        order: self.ability.order(),
                    },
                    filtering_options: EventFilteringOptions::default(),
                }
            }
        )
    }

    pub(crate) fn moveset_event_handlers_for<E: Event>(&self, event: E) -> Vec<OwnedEventHandler<E>> {
        self.moveset
            .iter()
            .filter_map(|move_| {
                event.corresponding_handler(move_.event_handlers()) 
                    .map(|event_handler| { // Add an OwnedEventHandler if an EventHandler exists.
                        OwnedEventHandler {
                            event_handler,
                            owner_id: self.id,
                            activation_order: ActivationOrder {
                                priority: move_.priority(),
                                speed: self.stat(Stat::Speed),
                                order: 0,
                            },
                            filtering_options: EventFilteringOptions::default(),
                        }
                    })
                })
                .collect::<Vec<_>>()
    }

    pub(crate) fn event_handlers_for<E: Event>(&self, event: E) -> Vec<OwnedEventHandler<E>> {
        let mut out = Vec::new();
        event.corresponding_handler((self.species.event_handlers)()) 
            .map(|event_handler| { // Add an OwnedEventHandler if an EventHandler exists.
                OwnedEventHandler {
                    event_handler,
                    owner_id: self.id,
                    activation_order:  ActivationOrder {
                        priority: 0,
                        speed: self.stat(Stat::Speed),
                        order: 0,
                    },
                    filtering_options: EventFilteringOptions::default(),
                }
            })
            .pipe(|optional_owned_event_handler| { // TODO: pipe_if_some
                if let Some(owned_event_handler) = optional_owned_event_handler {
                    out.push(owned_event_handler);
                }
            });
        self.ability_event_handler_for(event).pipe(|optional_owned_event_handler| {
            if let Some(owned_event_handler) = optional_owned_event_handler {
                out.push(owned_event_handler)
            }
        });
        out.append(&mut self.moveset_event_handlers_for(event));
        out
    }

    pub(crate) fn full_name(&self) -> String {
        if let Some(nickname) = self.nickname {
            format!["{} the {}", nickname, self.species.name]
        } else {
            self.species.name.to_string()
        }
    }

    pub(crate) fn status_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format![
            "{} ({}) [HP: {}/{}] @ {}\n",
            self.full_name(), self.id, self.current_health, self.max_health(), self.board_position
        ]);
        out
    }
}

#[derive(Clone, Copy)]
pub struct MonsterSpecies {
    dex_number: u16,
    name: &'static str,
    primary_type: Type,
    secondary_type: Option<Type>,
    base_stats: StatSet,
    event_handlers: fn() -> EventHandlerDeck,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonsterID {
    pub team_id: TeamID,
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

pub const ALLY_1: MonsterID = MonsterID {
    team_id: TeamID::Allies,
    monster_number: MonsterNumber::_1,
};
pub const ALLY_2: MonsterID = MonsterID {
    team_id: TeamID::Allies,
    monster_number: MonsterNumber::_2,
};
pub const ALLY_3: MonsterID = MonsterID {
    team_id: TeamID::Allies,
    monster_number: MonsterNumber::_3,
};
pub const ALLY_4: MonsterID = MonsterID {
    team_id: TeamID::Allies,
    monster_number: MonsterNumber::_4,
};
pub const ALLY_5: MonsterID = MonsterID {
    team_id: TeamID::Allies,
    monster_number: MonsterNumber::_5,
};
pub const ALLY_6: MonsterID = MonsterID {
    team_id: TeamID::Allies,
    monster_number: MonsterNumber::_6,
};

pub const OPPONENT_1: MonsterID = MonsterID {
    team_id: TeamID::Opponents,
    monster_number: MonsterNumber::_1,
};
pub const OPPONENT_2: MonsterID = MonsterID {
    team_id: TeamID::Opponents,
    monster_number: MonsterNumber::_2,
};
pub const OPPONENT_3: MonsterID = MonsterID {
    team_id: TeamID::Opponents,
    monster_number: MonsterNumber::_3,
};
pub const OPPONENT_4: MonsterID = MonsterID {
    team_id: TeamID::Opponents,
    monster_number: MonsterNumber::_4,
};
pub const OPPONENT_5: MonsterID = MonsterID {
    team_id: TeamID::Opponents,
    monster_number: MonsterNumber::_5,
};
pub const OPPONENT_6: MonsterID = MonsterID {
    team_id: TeamID::Opponents,
    monster_number: MonsterNumber::_6,
};

impl Display for MonsterID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.team_id {
            TeamID::Allies => match self.monster_number {
                MonsterNumber::_1 => write!(f, "First Ally"),
                MonsterNumber::_2 => write!(f, "Second Ally"),
                MonsterNumber::_3 => write!(f, "Third Ally"),
                MonsterNumber::_4 => write!(f, "Fourth Ally"),
                MonsterNumber::_5 => write!(f, "Fifth Ally"),
                MonsterNumber::_6 => write!(f, "Sixth Ally"),
            },
            TeamID::Opponents => match self.monster_number {
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

impl MonsterSpecies {
    pub const fn from_dex_entry(dex_entry: MonsterDexEntry) -> Self {
        let MonsterDexEntry { dex_number, name, primary_type, secondary_type, base_stats, event_handlers } = dex_entry;
        Self {
            dex_number,
            name,
            primary_type,
            secondary_type,
            base_stats,
            event_handlers,
        }
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.name
    }
    
    #[inline(always)]
    pub fn type_(&self) -> (Type, Option<Type>) {
        (self.primary_type, self.secondary_type)
    }

    #[inline(always)]
    pub fn primary_type(&self) -> Type {
        self.primary_type
    }

    #[inline(always)]
    pub fn secondary_type(&self) -> Option<Type> {
        self.secondary_type
    }

    #[inline(always)]
    pub fn base_stat(&self, stat: Stat) -> u16 {
        self.base_stats[stat]
    }
    
    #[inline(always)]
    pub fn event_handlers(&self) -> EventHandlerDeck {
        (self.event_handlers)()
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

#[derive(Clone, Copy)]
pub struct MonsterDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub primary_type: Type,
    pub secondary_type: Option<Type>,
    pub base_stats: StatSet,
    pub event_handlers: fn() -> EventHandlerDeck,
}

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
