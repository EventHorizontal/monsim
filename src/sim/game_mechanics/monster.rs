mod stats;

use core::{fmt::Debug, panic};
use std::fmt::{Display, Formatter};

use monsim_utils::MaxSizedVec;
pub use stats::*;
use tap::Pipe;

use super::{Ability, TeamID};
use crate::{
    sim::{
        event_dispatcher::EventContext,
        targetting::{BoardPosition, FieldPosition},
        ActivationOrder, EventHandlerSet, Type,
    },
    status::{PersistentStatus, VolatileStatus, VolatileStatusSpecies},
    Broadcaster, EventHandler, Item, Move, OwnedEventHandler,
};

#[derive(Debug, Clone)]
pub struct Monster {
    pub(crate) id: MonsterID,
    pub(crate) species: &'static MonsterSpecies,

    pub(crate) nickname: Option<&'static str>,
    pub(crate) effort_values: StatSet,
    pub(crate) current_health: u16,
    pub(crate) individual_values: StatSet,
    pub(crate) level: u16,
    pub(crate) nature: MonsterNature,
    pub(crate) board_position: BoardPosition,
    pub(crate) stat_modifiers: StatModifierSet,

    pub(crate) moveset: MaxSizedVec<Move, 4>,
    pub(crate) ability: Ability,
    pub(crate) persistent_status: Option<PersistentStatus>,
    pub(crate) volatile_statuses: MaxSizedVec<VolatileStatus, 16>,
    pub(crate) held_item: Option<Item>,
    pub(crate) consumed_item: Option<Item>,
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
                    nickname,
                    self.species.name,
                    self.id,
                    self.current_health,
                    self.max_health()
                ]
                .as_str(),
            );
        } else {
            out.push_str(
                format![
                    "{} ({}) [HP: {}/{}]\n\t│\t│\n",
                    self.species.name,
                    self.id,
                    self.current_health,
                    self.max_health()
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

impl Monster {
    // public
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
        Monster::calculate_max_health(
            self.species.base_stat(Stat::Hp),
            self.individual_values[Stat::Hp],
            self.effort_values[Stat::Hp],
            self.level,
        )
    }

    #[inline(always)]
    pub fn stat(&self, stat: Stat) -> u16 {
        match stat {
            Stat::Hp => self.max_health(),
            _ => {
                // TODO: Division is supposed to be floating point here.
                ((2 * self.species.base_stats[stat] + self.individual_values[stat] + (self.effort_values[stat] / 4)) * self.level) / 100 + 5
                // * self.nature[stat]
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
    pub fn stat_modifier(&self, stat: ModifiableStat) -> i8 {
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

    #[inline(always)]
    pub fn held_item(&self) -> &Option<Item> {
        &self.held_item
    }

    #[inline(always)]
    pub fn held_item_mut(&mut self) -> &mut Option<Item> {
        &mut self.held_item
    }

    #[inline(always)]
    pub fn consumed_item(&self) -> &Option<Item> {
        &self.consumed_item
    }

    #[inline(always)]
    pub fn consumed_item_mut(&mut self) -> &mut Option<Item> {
        &mut self.consumed_item
    }

    pub fn field_position(&self) -> Option<FieldPosition> {
        match self.board_position {
            BoardPosition::Bench => None,
            BoardPosition::Field(field_position) => Some(field_position),
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.board_position, BoardPosition::Field(_))
    }

    /// Returns the `VolatileStatus` of Monster of corresponding to a particular `VolatileStatusSpecies`, if the Monster
    /// has that particular status.
    pub fn volatile_status(&self, marker_species: VolatileStatusSpecies) -> Option<&VolatileStatus> {
        self.volatile_statuses.iter().find(|marker| *marker.species == marker_species)
    }

    // TODO: We might change this if we decide that we want the current type of the
    // monster to be different from the species' type.
    #[inline(always)]
    pub fn type_(&self) -> (Type, Option<Type>) {
        self.species.type_()
    }
}

impl Monster {
    // private

    pub(crate) fn calculate_max_health(base_hp: u16, hp_iv: u16, hp_ev: u16, level: u16) -> u16 {
        ((2 * base_hp + hp_iv + (hp_ev / 4)) * level) / 100 + level + 10
    }

    pub(crate) fn owned_event_handlers<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy>(
        &self,
        event_handler_selector: fn(EventHandlerSet) -> Vec<Option<EventHandler<R, C, B>>>,
    ) -> Vec<OwnedEventHandler<R, C, B>> {
        let mut output_owned_event_handlers = Vec::new();

        // of the Monster itself
        event_handler_selector((self.species.event_handlers)())
            .into_iter()
            .flatten()
            .map(|event_handler| {
                // Add an OwnedEventHandler if an EventHandler exists.
                OwnedEventHandler {
                    event_handler,
                    owner_id: self.id,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: self.stat(Stat::Speed),
                        order: 0,
                    },
                }
            })
            .pipe(|owned_event_handlers| {
                output_owned_event_handlers.extend(owned_event_handlers);
            });

        // from the Monster's ability
        event_handler_selector((&self).ability.event_handlers())
            .into_iter()
            .flatten()
            .map(|event_handler| {
                // Add an OwnedEventHandler if an EventHandler exists.
                OwnedEventHandler {
                    event_handler,
                    owner_id: (&self).id,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: (&self).stat(Stat::Speed),
                        order: (&self).ability.order(),
                    },
                }
            })
            .pipe(|owned_event_handlers| {
                output_owned_event_handlers.extend(owned_event_handlers);
            });

        // INFO: Moves don't have EventHandlers any more. This may be reverted in the future.

        // from the Monster's volatile statuses
        self.volatile_statuses.into_iter().for_each(|volatile_status| {
            let owned_event_handlers = event_handler_selector(volatile_status.event_handlers())
                .into_iter()
                .flatten()
                .map(|event_handler| OwnedEventHandler {
                    event_handler,
                    owner_id: self.id,
                    activation_order: ActivationOrder {
                        priority: 0,
                        speed: self.stat(Stat::Speed),
                        order: 0,
                    },
                });
            output_owned_event_handlers.extend(owned_event_handlers)
        });

        // from the Monster's persistent status
        if let Some(persistent_status) = self.persistent_status {
            event_handler_selector(persistent_status.event_handlers())
                .into_iter()
                .flatten()
                .for_each(|event_handler| {
                    let owned_event_handler = OwnedEventHandler {
                        event_handler,
                        owner_id: self.id,
                        activation_order: ActivationOrder {
                            priority: 0,
                            speed: self.stat(Stat::Speed),
                            order: 0,
                        },
                    };
                    output_owned_event_handlers.extend([owned_event_handler].into_iter());
                });
        }

        // from the Monster's held item
        if let Some(held_item) = self.held_item {
            event_handler_selector(held_item.event_handlers())
                .into_iter()
                .flatten()
                .for_each(|event_handler| {
                    let owned_event_handler = OwnedEventHandler {
                        event_handler,
                        owner_id: self.id,
                        activation_order: ActivationOrder {
                            priority: 0,
                            speed: self.stat(Stat::Speed),
                            order: 0,
                        },
                    };
                    output_owned_event_handlers.extend([owned_event_handler].into_iter());
                });
        }

        output_owned_event_handlers
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
        let persistent_status = match self.persistent_status {
            Some(persistent_status) => persistent_status.name(),
            None => "None",
        };
        let held_item = match self.held_item {
            Some(held_item) => held_item.name().to_string(),
            None => "None".to_string(),
        };

        let health_fraction = self.current_health() as f64 / self.max_health() as f64;
        let health_fraction = (health_fraction * 10.0).round() as usize;

        use colored::Colorize;
        let mut health_bar_filled = String::new();
        let mut health_bar_empty = String::new();
        for i in 1..=10 {
            if i <= health_fraction {
                health_bar_filled += "▓"
            } else {
                health_bar_empty += "░"
            }
        }

        out.push_str(&format![
            "{} ({}) HP:[{}{}] {}/{} | Position: {} | Persistent Status: {} | Volatile Statuses: {} | Held Item: {}\n",
            self.full_name(),
            self.id,
            health_bar_filled.green(),
            health_bar_empty.red(),
            self.current_health,
            self.max_health(),
            self.board_position,
            persistent_status,
            self.volatile_statuses,
            held_item
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
    event_handlers: fn() -> EventHandlerSet,
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
        let MonsterDexEntry {
            dex_number,
            name,
            primary_type,
            secondary_type,
            base_stats,
            event_handlers,
        } = dex_entry;
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
    pub fn event_handlers(&self) -> EventHandlerSet {
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
    pub event_handlers: fn() -> EventHandlerSet,
}
