pub mod ability;
#[cfg(feature = "debug")]
pub(crate) mod ability_dex;
pub mod monster;
#[cfg(feature = "debug")]
pub(crate) mod monster_dex;
pub mod move_;
#[cfg(feature = "debug")]
pub(crate) mod move_dex;

use core::marker::Copy;
use std::fmt::{Debug, Display, Formatter};

use super::event::{
    ActivationOrder, EventFilterOptions, CompositeEventResponderInstance, CompositeEventResponderInstanceList,
};
pub use ability::*;
pub use monster::*;
pub use move_::*;

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlerTeam {
    battlers: Vec<Battler>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllyBattlerTeam(pub BattlerTeam);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpponentBattlerTeam(pub BattlerTeam);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamID {
    Allies,
    Opponents,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battler {
    pub uid: BattlerUID,
    pub on_field: bool,
    pub monster: Monster,
    pub moveset: MoveSet,
    pub ability: Ability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BattlerNumber {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
}

impl From<usize> for BattlerNumber {
    fn from(value: usize) -> Self {
        match value {
            0 => BattlerNumber::_1,
            1 => BattlerNumber::_2,
            2 => BattlerNumber::_3,
            3 => BattlerNumber::_4,
            4 => BattlerNumber::_5,
            5 => BattlerNumber::_6,
            _ => panic!("BattlerNumber can only be formed from usize 0 to 5."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllyBattler(Battler);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpponentBattler(Battler);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattlerUID {
    pub team_id: TeamID,
    pub battler_number: BattlerNumber,
}

impl Display for BattlerUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.team_id, self.battler_number)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MoveUID {
    pub battler_uid: BattlerUID,
    pub move_number: MoveNumber,
}

impl BattlerTeam {
    pub fn new(battlers: Vec<Battler>) -> Self {
        assert!(
            battlers.first().is_some(),
            "There is not a single monster in the team."
        );
        assert!(battlers.len() <= MAX_BATTLERS_PER_TEAM);
        BattlerTeam { battlers }
    }

    pub fn battlers(&self) -> &Vec<Battler> {
        &self.battlers
    }

    pub fn battlers_mut(&mut self) -> &mut Vec<Battler> {
        &mut self.battlers
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for battler in self.battlers() {
            out.push_str(&Self::battler_status_as_string(battler));
        }
        out
    }

    pub fn battler_status_as_string(battler: &Battler) -> String {
        let mut out = String::new();
        if battler.monster.nickname == battler.monster.species.name {
            out.push_str(&format![
                "{} ({}) [HP: {}/{}]\n",
                battler.monster.species.name,
                battler.uid,
                battler.monster.current_health,
                battler.monster.max_health
            ]);
        } else {
            out.push_str(&format![
                "{} the {} ({}) [HP: {}/{}]\n",
                battler.monster.nickname,
                battler.monster.species.name,
                battler.uid,
                battler.monster.current_health,
                battler.monster.max_health
            ]);
        }
        out
    }

    pub fn event_response_instances(&self) -> CompositeEventResponderInstanceList {
        let mut out = Vec::new();
        for battler in self.battlers.iter() {
            out.append(&mut battler.composite_event_responder_instances())
        }
        out
    }

    pub fn active_battler(&self) -> &Battler {
        &self.battlers[0]
    }
}

impl AllyBattlerTeam {
    pub fn new(battlers: Vec<Battler>) -> Self {
        assert!(
            battlers.first().is_some(),
            "There is not a single monster in the team."
        );
        assert!(battlers.len() <= MAX_BATTLERS_PER_TEAM);
        Self(BattlerTeam { battlers })
    }

    pub fn battlers(&self) -> &Vec<Battler> {
        &self.0.battlers
    }

    pub fn battlers_mut(&mut self) -> &mut Vec<Battler> {
        &mut self.0.battlers
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for battler in self.0.battlers() {
            out.push_str(&BattlerTeam::battler_status_as_string(battler));
        }
        out
    }

    pub fn composite_event_responder_instances(&self) -> CompositeEventResponderInstanceList {
        let mut out = Vec::new();
        self.0
            .battlers
            .iter()
            .for_each(|battler| out.append(&mut battler.composite_event_responder_instances()));
        out
    }

    pub fn active_battler(&self) -> &Battler {
        &self.0.battlers[0]
    }

    pub fn unwrap(&self) -> BattlerTeam {
        self.0.clone()
    }
}

impl OpponentBattlerTeam {
    pub fn new(battlers: Vec<Battler>) -> Self {
        assert!(
            battlers.first().is_some(),
            "There is not a single monster in the team."
        );
        assert!(battlers.len() <= MAX_BATTLERS_PER_TEAM);
        Self(BattlerTeam { battlers })
    }

    pub fn battlers(&self) -> &Vec<Battler> {
        &self.0.battlers
    }

    pub fn battlers_mut(&mut self) -> &mut Vec<Battler> {
        &mut self.0.battlers
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for battler in self.0.battlers() {
            out.push_str(&BattlerTeam::battler_status_as_string(battler));
        }
        out
    }

    pub fn composite_event_responder_instances(&self) -> CompositeEventResponderInstanceList {
        let mut out = Vec::new();
        for battler in self.0.battlers.iter() {
            out.append(&mut battler.composite_event_responder_instances())
        }
        out
    }

    pub fn active_battler(&self) -> &Battler {
        &self.0.battlers[0]
    }

    pub fn unwrap(&self) -> BattlerTeam {
        self.0.clone()
    }
}

impl Display for Battler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        if self.monster.nickname == self.monster.species.name {
            out.push_str(
                format![
                    "{} ({}) [HP: {}/{}]\n\t│\t│\n",
                    self.monster.species.name,
                    self.uid,
                    self.monster.current_health,
                    self.monster.max_health
                ]
                .as_str(),
            );
        } else {
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
        }

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

    pub fn is_type(&self, test_type: ElementalType) -> bool {
        self.monster.is_type(test_type)
    }

    pub fn monster_composite_event_responder_instance(&self) -> CompositeEventResponderInstance {
        let activation_order = ActivationOrder {
            priority: 0,
            speed: self.monster.stats[Stat::Speed],
            order: 0,
        };
        CompositeEventResponderInstance {
            composite_event_responder: self.monster.composite_event_responder(),
            owner_uid: self.uid,
            activation_order,
            filters: EventFilterOptions::default(),
        }
    }

    pub fn ability_composite_event_responder_instance(&self) -> CompositeEventResponderInstance {
        let activation_order = ActivationOrder {
            priority: 0,
            speed: self.monster.stats[Stat::Speed],
            order: self.ability.species.order,
        };
        CompositeEventResponderInstance {
            composite_event_responder: self.ability.composite_event_responder(),
            owner_uid: self.uid,
            activation_order,
            filters: EventFilterOptions::default(),
        }
    }

    pub fn moveset_composite_event_responder_instances(&self, uid: BattlerUID) -> CompositeEventResponderInstanceList {
        self.moveset
            .moves()
            .map(|it| CompositeEventResponderInstance {
                composite_event_responder: it.species.composite_event_responder,
                owner_uid: uid,
                activation_order: ActivationOrder {
                    priority: it.species.priority,
                    speed: self.monster.stats[Stat::Speed],
                    order: 0,
                },
                filters: EventFilterOptions::default(),
            })
            .collect::<Vec<_>>()
    }

    pub fn fainted(&self) -> bool {
        self.monster.fainted()
    }

    pub fn composite_event_responder_instances(&self) -> CompositeEventResponderInstanceList {
        let mut out = Vec::new();
        out.push(self.monster_composite_event_responder_instance());
        out.append(&mut self.moveset_composite_event_responder_instances(self.uid));
        out.push(self.ability_composite_event_responder_instance());
        out
    }

    pub(crate) fn move_uids(&self) -> Vec<MoveUID> {
        self.moveset
            .moves()
            .enumerate()
            .map(|(idx, _)| MoveUID {
                battler_uid: self.uid,
                move_number: MoveNumber::from(idx),
            })
            .collect()
    }
}

impl AllyBattler {
    pub fn unwrap(&self) -> Battler {
        self.0.clone()
    }
}

impl OpponentBattler {
    pub fn unwrap(&self) -> Battler {
        self.0.clone()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ElementalType {
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
