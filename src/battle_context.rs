use super::*;

use std::{
    fmt::Display,
    iter::Chain,
    slice::{Iter, IterMut},
};

type BattlerIterator<'a> = Chain<Iter<'a, Battler>, Iter<'a, Battler>>;
type MutableBattlerIterator<'a> = Chain<IterMut<'a, Battler>, IterMut<'a, Battler>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleContext {
    pub current_action: Option<ActionChoice>,
    pub state: BattleState,
    pub ally_team: BattlerTeam,
    pub opponent_team: BattlerTeam,
    pub message_buffer: MessageBuffer,
}

pub type MessageBuffer = Vec<String>;
pub const CONTEXT_MESSAGE_BUFFER_SIZE: usize = 20;

impl BattleContext {
    pub fn new(ally_team: BattlerTeam, opponent_team: BattlerTeam) -> Self {
        Self {
            current_action: None,
            state: BattleState::UsingMove {
                move_uid: MoveUID {
                    battler_uid: BattlerUID {
                        team_id: TeamID::Ally,
                        battler_number: BattlerNumber::_1,
                    },
                    move_number: MoveNumber::_1,
                },
                target_uid: BattlerUID {
                    team_id: TeamID::Opponent,
                    battler_number: BattlerNumber::_1,
                },
            },
            ally_team,
            opponent_team,
            message_buffer: Vec::with_capacity(CONTEXT_MESSAGE_BUFFER_SIZE),
        }
    }

    pub fn battlers(&self) -> BattlerIterator {
        let left = self.ally_team.battlers();
        let right = self.opponent_team.battlers();

        left.into_iter().chain(right)
    }

    fn battlers_mut(&mut self) -> MutableBattlerIterator {
        let left = self.ally_team.battlers_mut();
        let right = self.opponent_team.battlers_mut();

        left.into_iter().chain(right)
    }

    fn find_battler(&self, battler_uid: BattlerUID) -> &Battler {
        self.battlers().find(|it| it.uid == battler_uid).expect(
            "Error: Requested look up for a monster with ID that does not exist in this battle.",
        )
    }

    pub fn is_battler_on_field(&self, battler_uid: BattlerUID) -> bool {
        self.find_battler(battler_uid).on_field
    }

    pub fn current_action_user(&self) -> Option<&Battler> {
        if let Some(current_action) = self.current_action {
            Some(self.find_battler(current_action.chooser()))
        } else {
            None
        }
    }

    pub fn is_current_action_user(&self, test_monster_uid: BattlerUID) -> Option<bool> {
        if let Some(current_action) = self.current_action {
            Some(test_monster_uid == current_action.chooser())
        } else {
            None
        }
    }

    pub fn current_action_target(&self) -> Option<&Battler> {
        if let Some(current_action) = self.current_action {
            Some(self.find_battler(current_action.target()))
        } else {
            None
        }
    }

    pub fn is_current_action_target(&self, test_monster_uid: BattlerUID) -> Option<bool> {
        if let Some(current_action) = self.current_action {
            Some(test_monster_uid == current_action.target())
        } else {
            None
        }
    }

    pub fn monster(&self, uid: BattlerUID) -> &Monster {
        &self
            .battlers()
            .find(|it| it.uid == uid)
            .expect(format!["Theres should exist a monster with id {:?}", uid].as_str())
            .monster
    }

    pub fn monster_mut(&mut self, uid: BattlerUID) -> &mut Monster {
        &mut self
            .battlers_mut()
            .find(|it| it.uid == uid)
            .expect(format!["Theres should exist a monster with id {:?}", uid].as_str())
            .monster
    }

    pub fn ability(&self, owner_uid: BattlerUID) -> &Ability {
        &self
            .battlers()
            .find(|it| it.uid == owner_uid)
            .expect(format!["Theres should exist a monster with id {:?}", owner_uid].as_str())
            .ability
    }

    pub fn ability_mut(&mut self, owner_uid: BattlerUID) -> &mut Ability {
        &mut self
            .battlers_mut()
            .find(|it| it.uid == owner_uid)
            .expect(format!["Theres should exist a monster with id {:?}", owner_uid].as_str())
            .ability
    }

    pub fn move_(&self, move_uid: MoveUID) -> &Move {
        let owner_uid = move_uid.battler_uid;
        self.battlers()
            .find(|it| it.uid == owner_uid)
            .expect(format!["Theres should exist a monster with id {:?}", owner_uid].as_str())
            .moveset
            .move_(move_uid.move_number)
    }

    pub fn move_mut(&mut self, move_uid: MoveUID) -> &mut Move {
        let owner_uid = move_uid.battler_uid;
        self.battlers_mut()
            .find(|it| it.uid == owner_uid)
            .expect(format!["Theres should exist a monster with id {:?}", owner_uid].as_str())
            .moveset
            .move_mut(move_uid.move_number)
    }

    pub fn event_handler_sets_plus_info(&self) -> EventHandlerSetInfoList {
        let mut out = Vec::new();
        out.append(&mut self.ally_team.event_handlers());
        out.append(&mut self.opponent_team.event_handlers());
        out
    }

    pub(crate) fn filter_event_handlers(
        &self,
        event_caller_uid: BattlerUID,
        owner_uid: BattlerUID,
        event_handler_filters: EventHandlerFilters,
    ) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if event_caller_uid == owner_uid {
                bitmask |= TargetFlags::SELF.bits()
            } // 0x01
            if self.are_allies(owner_uid, event_caller_uid) {
                bitmask |= TargetFlags::ALLIES.bits()
            } // 0x02
            if self.are_opponents(owner_uid, event_caller_uid) {
                bitmask |= TargetFlags::OPPONENTS.bits()
            } //0x04
              // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = event_handler_filters.whose_event.bits() == bitmask;
        let on_battlefield_passed = self.is_battler_on_field(owner_uid);

        event_source_filter_passed && on_battlefield_passed
    }

    fn is_on_ally_team(&self, uid: BattlerUID) -> bool {
        self.ally_team.battlers().iter().any(|it| it.uid == uid)
    }

    fn is_on_opponent_team(&self, uid: BattlerUID) -> bool {
        self.opponent_team.battlers().iter().any(|it| it.uid == uid)
    }

    fn are_opponents(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_opponent_team(event_caller_uid))
            || (self.is_on_ally_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    fn are_allies(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_ally_team(event_caller_uid))
            || (self.is_on_opponent_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    pub fn battlers_on_field(&self) -> Vec<&Battler> {
        self.battlers().filter(|it| it.on_field).collect::<Vec<_>>()
    }

    /// Given an action choice, computes its activation order. This is handled by BattleContext because the order is
    /// context dependent.
    pub(crate) fn choice_activation_order(&self, choice: ActionChoice) -> ActivationOrder {
        match choice {
            ActionChoice::Move {
                move_uid,
                target_uid: _,
            } => ActivationOrder {
                priority: self.move_(move_uid).species.priority,
                speed: self.monster(move_uid.battler_uid).stats[Stat::Speed],
                order: 0,
            },
        }
    }

    pub fn generate_available_actions(&self) -> AvailableActions {
        let ally_active_battler = self.ally_team.active_battler();
        let opponent_active_battler = self.opponent_team.active_battler();

        let ally_moves = ally_active_battler.move_uids();
        let opponent_moves = opponent_active_battler.move_uids();

        let mut ally_team_choices: TeamAvailableActions = Vec::with_capacity(4);
        for move_uid in ally_moves {
            ally_team_choices.push(ActionChoice::Move {
                move_uid,
                target_uid: opponent_active_battler.uid,
            });
        }

        let mut opponent_team_choices: TeamAvailableActions = Vec::with_capacity(4);
        for move_uid in opponent_moves {
            opponent_team_choices.push(ActionChoice::Move {
                move_uid,
                target_uid: ally_active_battler.uid,
            });
        }

        AvailableActions {
            ally_team_choices,
            opponent_team_choices,
        }
    }
}

impl Display for BattleContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        push_teamwise_pretty_tree(
            &mut out, 
            "Ally Team\n", 
            &self.ally_team, 
            self.ally_team
                .battlers()
                .iter()
                .count()
        );
        push_teamwise_pretty_tree(
            &mut out, 
            "Opponent Team\n", 
            &self.opponent_team, 
            self.opponent_team
                .battlers()
                .iter()
                .count()
        );
        write!(f, "{}", out)
    }
}

fn push_teamwise_pretty_tree(out: &mut String, team_name: &str, team: &BattlerTeam, number_of_monsters: usize) {
    out.push_str(team_name);
    for (i, battler) in team.battlers().iter().enumerate() {
        let is_not_last_monster = i < number_of_monsters - 1;
        if is_not_last_monster {
            out.push_str("\t├── ");
            out.push_str(
                format![
                    "{} the {} ({}) [HP: {}/{}]\n",
                    battler.monster.nickname,
                    battler.monster.species.name,
                    battler.uid,
                    battler.monster.current_health,
                    battler.monster.max_health
                ]
                .as_str(),
            );
            out.push_str("\t│\t│\n");
            let number_of_effects = battler.moveset.moves().count();
            out.push_str("\t│\t├── ");

            out.push_str(
                format![
                    "type {:?}/{:?} \n",
                    battler.monster.species.primary_type,
                    battler.monster.species.secondary_type
                ]
                .as_str(),
            );
            out.push_str("\t│\t├── ");
            out.push_str(format!["abl {}\n", battler.ability.species.name].as_str());

            for (j, move_) in battler.moveset.moves().enumerate() {
                let is_not_last_move = j < number_of_effects - 1;
                if is_not_last_move {
                    out.push_str("\t│\t├── ");
                } else {
                    out.push_str("\t│\t└── ");
                }
                out.push_str(format!["mov {}\n", move_.species.name].as_str());
            }
            out.push_str("\t│\t\n");
        } else {
            out.push_str("\t└── ");
            out.push_str(
                format![
                    "{} the {} ({}) [HP: {}/{}]\n",
                    battler.monster.nickname,
                    battler.monster.species.name,
                    battler.uid,
                    battler.monster.current_health,
                    battler.monster.max_health
                ]
                .as_str(),
            );
            out.push_str("\t\t│\n");
            let number_of_effects = battler.moveset.moves().count();
            out.push_str("\t\t├── ");

            out.push_str(
                format![
                    "type {:?}/{:?} \n",
                    battler.monster.species.primary_type,
                    battler.monster.species.secondary_type
                ]
                .as_str(),
            );
            out.push_str("\t\t├── ");

            out.push_str(format!["abl {}\n", battler.ability.species.name].as_str());

            for (j, move_) in battler.moveset.moves().enumerate() {
                let is_not_last_move = j < number_of_effects - 1;
                if is_not_last_move {
                    out.push_str("\t\t├── ");
                } else {
                    out.push_str("\t\t└── ");
                }
                out.push_str(format!["mov {}\n", move_.species.name].as_str());
            }
            out.push_str("\t\t\n");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleState {
    UsingMove {
        move_uid: MoveUID,
        target_uid: BattlerUID,
    },
    Finished,
}
