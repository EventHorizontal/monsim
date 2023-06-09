
use std::{
    fmt::Display,
    iter::Chain,
    slice::{Iter, IterMut},
};
use crate::sim::{Battler, ActionChoice, AllyBattlerTeam, OpponentBattlerTeam, BattlerTeam, MoveUID, BattlerUID, TeamID, BattlerNumber, MoveNumber, Monster, Move, Ability, ActivationOrder, TeamAvailableActions, AvailableActions, Stat, event::EventResponderInstanceList};

type BattlerIterator<'a> = Chain<Iter<'a, Battler>, Iter<'a, Battler>>;
type MutableBattlerIterator<'a> = Chain<IterMut<'a, Battler>, IterMut<'a, Battler>>;

pub type MessageBuffer = Vec<String>;
pub const CONTEXT_MESSAGE_BUFFER_SIZE: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battle {
    pub current_action: Option<ActionChoice>,
    pub sim_state: SimState,
    pub ally_team: AllyBattlerTeam,
    pub opponent_team: OpponentBattlerTeam,
    pub message_buffer: MessageBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimState {
    UsingMove {
        move_uid: MoveUID,
        target_uid: BattlerUID,
    },
    Finished,
}

impl Battle {
    pub fn new(ally_team: AllyBattlerTeam, opponent_team: OpponentBattlerTeam) -> Self {
        Self {
            current_action: None,
            sim_state: SimState::UsingMove {
                move_uid: MoveUID {
                    battler_uid: BattlerUID {
                        team_id: TeamID::Allies,
                        battler_number: BattlerNumber::_1,
                    },
                    move_number: MoveNumber::_1,
                },
                target_uid: BattlerUID {
                    team_id: TeamID::Opponents,
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

        left.iter().chain(right)
    }

    pub fn battlers_mut(&mut self) -> MutableBattlerIterator {
        let left = self.ally_team.battlers_mut();
        let right = self.opponent_team.battlers_mut();

        left.iter_mut().chain(right)
    }

    pub fn find_battler(&self, battler_uid: BattlerUID) -> &Battler {
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
        self.current_action
            .map(|current_action| test_monster_uid == current_action.chooser())
    }

    pub fn current_action_target(&self) -> Option<&Battler> {
        if let Some(current_action) = self.current_action {
            Some(self.find_battler(current_action.target()))
        } else {
            None
        }
    }

    pub fn is_current_action_target(&self, test_monster_uid: BattlerUID) -> Option<bool> {
        self.current_action
            .map(|current_action| test_monster_uid == current_action.target())
    }

    pub fn monster(&self, uid: BattlerUID) -> &Monster {
        &self
            .battlers()
            .find(|it| it.uid == uid)
            .unwrap_or_else(|| panic!("Theres should exist a monster with id {:?}", uid))
            .monster
    }

    pub fn monster_mut(&mut self, uid: BattlerUID) -> &mut Monster {
        &mut self
            .battlers_mut()
            .find(|it| it.uid == uid)
            .unwrap_or_else(|| panic!("Theres should exist a monster with id {:?}", uid))
            .monster
    }

    pub fn ability(&self, owner_uid: BattlerUID) -> &Ability {
        &self
            .battlers()
            .find(|it| it.uid == owner_uid)
            .unwrap_or_else(|| panic!("Theres should exist a monster with id {:?}", owner_uid))
            .ability
    }

    pub fn ability_mut(&mut self, owner_uid: BattlerUID) -> &mut Ability {
        &mut self
            .battlers_mut()
            .find(|it| it.uid == owner_uid)
            .unwrap_or_else(|| panic!("Theres should exist a monster with id {:?}", owner_uid))
            .ability
    }

    pub fn move_(&self, move_uid: MoveUID) -> &Move {
        let owner_uid = move_uid.battler_uid;
        self.battlers()
            .find(|it| it.uid == owner_uid)
            .unwrap_or_else(|| panic!("Theres should exist a monster with id {:?}", owner_uid))
            .moveset
            .move_(move_uid.move_number)
    }

    pub fn move_mut(&mut self, move_uid: MoveUID) -> &mut Move {
        let owner_uid = move_uid.battler_uid;
        self.battlers_mut()
            .find(|it| it.uid == owner_uid)
            .unwrap_or_else(|| panic!("Theres should exist a monster with id {:?}", owner_uid))
            .moveset
            .move_mut(move_uid.move_number)
    }

    pub fn event_responder_instances(&self) -> EventResponderInstanceList {
        let mut out = Vec::new();
        out.append(&mut self.ally_team.event_responder_instances());
        out.append(&mut &mut self.opponent_team.event_responder_instances());
        out
    }

    pub fn is_on_ally_team(&self, uid: BattlerUID) -> bool {
        self.ally_team.battlers().iter().any(|it| it.uid == uid)
    }

    pub fn is_on_opponent_team(&self, uid: BattlerUID) -> bool {
        self.opponent_team
            .battlers()
            .iter()
            .any(|it| it.uid == uid)
    }

    pub fn are_opponents(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_opponent_team(event_caller_uid))
            || (self.is_on_ally_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    pub fn are_allies(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_ally_team(event_caller_uid))
            || (self.is_on_opponent_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    pub fn battlers_on_field(&self) -> Vec<&Battler> {
        self.battlers().filter(|it| it.on_field).collect::<Vec<_>>()
    }

    /// Given an action choice, computes its activation order. This is handled by `Battle` because the order is
    /// context sensitive.
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

    pub fn get_current_action_as_move(&self) -> Option<&Move> {
        match self.current_action.unwrap() {
            ActionChoice::Move {
                move_uid,
                target_uid: _,
            } => Some(self.move_(move_uid)),
        }
    }

    pub fn push_message(&mut self, message: &dyn Display) {
        self.message_buffer.push(format!["{}", message]);
    }

    pub fn push_messages(&mut self, messages: &[&dyn Display]) {
        for message in messages {
            self.message_buffer.push(format!["{}", message]);
        }
    }
}

impl Display for Battle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        push_pretty_tree_for_team(
            &mut out,
            "Ally Team\n",
            &self.ally_team.unwrap(),
            self.ally_team.battlers().iter().count(),
        );
        push_pretty_tree_for_team(
            &mut out,
            "Opponent Team\n",
            &self.opponent_team.unwrap(),
            self.opponent_team.battlers().iter().count(),
        );
        write!(f, "{}", out)
    }
}

fn push_pretty_tree_for_team(
    output_string: &mut String,
    team_name: &str,
    team: &BattlerTeam,
    number_of_monsters: usize,
) {
    output_string.push_str(team_name);
    for (i, battler) in team.battlers().iter().enumerate() {
        let is_not_last_monster = i < number_of_monsters - 1;
        let prefix_str;
        let suffix_str;
        if is_not_last_monster { 
            prefix_str = "\t│\t";
            suffix_str = "├── "
        } else { 
            prefix_str = "\t \t";
            suffix_str = "└── "
        }
        output_string.push_str(&("\t".to_owned() + suffix_str));
        output_string.push_str(&BattlerTeam::battler_status_as_string(battler));
        output_string.push_str(&(prefix_str.to_owned() + "│\n"));
        output_string.push_str( &(prefix_str.to_owned() + "├── "));
        
        let primary_type = battler.monster.species.primary_type;
        let secondary_type = battler.monster.species.secondary_type;
        let type_string = if let Some(secondary_type) = secondary_type {
            format!["   type: {:?}/{:?}\n", primary_type, secondary_type]
        } else {
            format!["   type: {:?}\n", primary_type]
        };
        output_string.push_str(&type_string);
        
        output_string.push_str(&(prefix_str.to_owned() + "├── "));
        output_string.push_str(format!["ability: {}\n", battler.ability.species.name].as_str());
            
        let number_of_moves = battler.moveset.moves().count();
        for (j, move_) in battler.moveset.moves().enumerate() {
            let is_not_last_move = j < number_of_moves - 1;
            if is_not_last_move {
                output_string.push_str(&(prefix_str.to_owned() + "├── "));
            } else {
                output_string.push_str(&(prefix_str.to_owned() + "└── "));
            }
            output_string.push_str(format!["   move: {}\n", move_.species.name].as_str());
        }
        output_string.push_str(&(prefix_str.to_owned() + "\n"));
    }
}

