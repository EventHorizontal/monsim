pub mod ability;
pub mod ability_dex;
pub mod monster;
pub mod monster_dex;
pub mod move_;
pub mod move_dex;

use core::marker::Copy;
use std::fmt::{Debug, Display, Formatter};

use super::event::{
    ActivationOrder, EventHandlerFilters, EventHandlerSet, EventHandlerSetInfo,
    EventHandlerSetInfoList,
};
pub use ability::*;
pub use monster::*;
pub use move_::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamID {
    Ally,
    Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattlerUID {
    pub team_id: TeamID,
    pub battler_number: BattlerNumber,
}

impl Display for BattlerUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}_{:?}", self.battler_number, self.team_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveUID {
    pub battler_uid: BattlerUID,
    pub move_number: MoveNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlerTeam {
    battlers: Vec<Battler>,
}

const MAX_BATTLERS_PER_TEAM: usize = 6;

impl BattlerTeam {
    pub fn new(monsters: Vec<Battler>) -> Self {
        assert!(
            monsters.first() != None,
            "There is not a single monster in the team."
        );
        assert!(monsters.len() <= MAX_BATTLERS_PER_TEAM);
        return BattlerTeam { battlers: monsters };
    }

    pub fn battlers(&self) -> &Vec<Battler> {
        &self.battlers
    }

    pub fn battlers_mut(&mut self) -> &mut Vec<Battler> {
        &mut self.battlers
    }

    pub fn event_handlers(&self) -> EventHandlerSetInfoList {
        let mut out = Vec::new();
        for battler in self.battlers.iter() {
            out.append(&mut battler.event_handlers())
        }
        out
    }

    pub fn active_battler(&self) -> &Battler {
        &self.battlers[0]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battler {
    pub uid: BattlerUID,
    pub on_field: bool,
    pub monster: Monster,
    pub moveset: MoveSet,
    pub ability: Ability,
}

impl Display for Battler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        out.push_str(
            format![
                "{} the {} ({}) [HP: {}/{}]\n\t│\t│\n",
                self.monster.nickname,
                self.monster.species.name,
                self.uid,
                self.monster.current_health,
                self.monster.max_health
            ]
            .as_str(),
        );
        let number_of_effects = self.moveset.moves().count();

        out.push_str("\t│\t├── ");
        out.push_str(
            format![
                "type {:?}/{:?} \n",
                self.monster.species.primary_type, self.monster.species.secondary_type
            ]
            .as_str(),
        );

        out.push_str("\t│\t├── ");
        out.push_str(format!["abl {}\n", self.ability.species.name].as_str());

        for (i, move_) in self.moveset.moves().enumerate() {
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

impl Battler {
    pub fn new(
        uid: BattlerUID,
        on_field: bool,
        monster: Monster,
        moveset: MoveSet,
        ability: Ability,
    ) -> Self {
        Battler {
            uid,
            on_field,
            monster,
            moveset,
            ability,
        }
    }

    fn is_type(&self, test_type: MonType) -> bool {
        self.monster.is_type(test_type)
    }

    pub fn monster_event_handler_info(&self) -> EventHandlerSetInfo {
        let activation_order = ActivationOrder {
            priority: 0,
            speed: self.monster.stats[Stat::Speed],
            order: 0,
        };
        EventHandlerSetInfo {
            event_handler_set: self.monster.event_handlers(),
            owner_uid: self.uid,
            activation_order,
            filters: EventHandlerFilters::default(),
        }
    }

    pub fn ability_event_handler_info(&self) -> EventHandlerSetInfo {
        let activation_order = ActivationOrder {
            priority: 0,
            speed: self.monster.stats[Stat::Speed],
            order: self.ability.species.order,
        };
        EventHandlerSetInfo {
            event_handler_set: self.ability.event_handlers(),
            owner_uid: self.uid,
            activation_order,
            filters: EventHandlerFilters::default(),
        }
    }

    pub fn moveset_event_handler_info(&self, uid: BattlerUID) -> EventHandlerSetInfoList {
        self.moveset
            .moves()
            .map(|it| EventHandlerSetInfo {
                event_handler_set: it.species.event_handlers,
                owner_uid: uid,
                activation_order: ActivationOrder {
                    priority: it.species.priority,
                    speed: self.monster.stats[Stat::Speed],
                    order: 0,
                },
                filters: EventHandlerFilters::default(),
            })
            .collect::<Vec<_>>()
    }

    pub fn fainted(&self) -> bool {
        self.monster.fainted()
    }

    pub fn event_handlers(&self) -> EventHandlerSetInfoList {
        let mut out = Vec::new();
        out.push(self.monster_event_handler_info());
        out.append(&mut self.moveset_event_handler_info(self.uid));
        out.push(self.ability_event_handler_info());
        out
    }

    pub(crate) fn move_uids(&self) -> Vec<MoveUID> {
        self.moveset
            .moves()
            .enumerate()
            .map(|(idx, _)| {
                MoveUID { 
                    battler_uid: self.uid, 
                    move_number: MoveNumber::from(idx) 
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MonType {
    None, // For empty type-slots

    Bug,
    Dark,
    Dragon,
    Electric,
    Fairy,
    Fighting,
    Fire,
    Flying,
    Ghost,
    Grass,
    Ground,
    Ice,
    Normal,
    Poison,
    Psychic,
    Rock,
    Steel,
    Water,
}
