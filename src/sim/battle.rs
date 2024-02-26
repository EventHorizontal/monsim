use utils::{not, ArrayOfOptionals, Nothing, NOTHING};

use crate::sim::{
        event::CompositeEventResponderInstanceList, Ability, FullySpecifiedAction, ActivationOrder, AvailableActions, Monster,
        MonsterTeam, MonsterUID, Move, MoveUID, Stat, AvailableActionsForTeam,
        utils,
};

use std::{
    fmt::Display,
    iter::Chain,
    slice::{Iter, IterMut},
};

use super::{
    prng::{self, Prng}, PartiallySpecifiedAction, PerTeam, TeamID, ALLY_1, OPPONENT_1,
};

type MonsterIterator<'a> = Chain<Iter<'a, Monster>, Iter<'a, Monster>>;
type MutableMonsterIterator<'a> = Chain<IterMut<'a, Monster>, IterMut<'a, Monster>>;

pub type MessageLog = Vec<String>;
pub const CONTEXT_MESSAGE_BUFFER_SIZE: usize = 20;

/// The main data struct that contains all the information one could want to know about the current battle. This is meant to be passed around as a unit and queried for battle-related information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battle {
    pub is_finished: bool,
    pub turn_number: u16,
    pub prng: Prng,
    teams: PerTeam<MonsterTeam>,
    // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
    pub message_log: MessageLog,
    pub active_monster_uids: PerTeam<MonsterUID>,
}

impl Battle {
    pub fn new(teams: PerTeam<MonsterTeam>) -> Self {
        Self {
            is_finished: false,
            turn_number: 0,
            prng: Prng::new(prng::seed_from_time_now()),
            teams,
            message_log: Vec::with_capacity(CONTEXT_MESSAGE_BUFFER_SIZE),
            active_monster_uids: PerTeam::new(ALLY_1, OPPONENT_1),
        }
    }

    /// Tries to increment the turn number by checked addition, and returns an error if the turn number limit (u16::MAX) is exceeded.
    pub(crate) fn increment_turn_number(&mut self) -> Result<Nothing, &str> {
        match self.turn_number.checked_add(1) {
            Some(turn_number) => { self.turn_number = turn_number; Ok(NOTHING)},
            None => Err("Turn limit (65535) exceeded."),
        }
    }

    pub fn monsters(&self) -> MonsterIterator {
        let (ally_team, opponent_team) = self.teams.unwrap();
        ally_team.monsters().iter().chain(opponent_team.monsters())
    }

    pub fn monsters_mut(&mut self) -> MutableMonsterIterator {
        let (ally_team, opponent_team) = self.teams.unwrap_mut();
        ally_team.monsters_mut().iter_mut().chain(opponent_team.monsters_mut().iter_mut())
    }

    pub fn monster(&self, monster_uid: MonsterUID) -> &Monster {
        let team = self.team(monster_uid.team_id);
        team.monsters()
            .get(monster_uid.monster_number as usize)
            .expect("Only valid MonsterUIDs are expected to be passed to this")
    }

    pub fn monster_mut(&mut self, monster_uid: MonsterUID) -> &mut Monster {
        let team = self.team_mut(monster_uid.team_id);
        team.monsters_mut()
            .get_mut(monster_uid.monster_number as usize)
            .expect("Only valid MonsterUIDs are expected to be passed to this")
    }

    pub fn is_active_monster(&self, monster_uid: MonsterUID) -> bool {
        self.active_monster_uids[monster_uid.team_id] == monster_uid
    }

    pub fn ability(&self, owner_uid: MonsterUID) -> &Ability {
        &self.monster(owner_uid)
            .ability
    }

    pub fn ability_mut(&mut self, owner_uid: MonsterUID) -> &mut Ability {
        &mut self
            .monster_mut(owner_uid)
            .ability
    }

    pub fn move_(&self, move_uid: MoveUID) -> &Move {
        self.monster(move_uid.owner_uid)
            .moveset
            .move_(move_uid.move_number)
    }

    pub fn move_mut(&mut self, move_uid: MoveUID) -> &mut Move {
        self.monster_mut(move_uid.owner_uid)
            .moveset
            .move_mut(move_uid.move_number)
    }

    pub fn composite_event_responder_instances(&self) -> CompositeEventResponderInstanceList {
        let mut out = Vec::new();
        out.append(&mut self.ally_team().composite_event_responder_instances());
        out.append(&mut self.opponent_team().composite_event_responder_instances());
        out
    }

    pub fn is_on_ally_team(&self, uid: MonsterUID) -> bool {
        self.ally_team().monsters().iter().any(|it| it.uid == uid)
    }

    pub fn is_on_opponent_team(&self, uid: MonsterUID) -> bool {
        self.opponent_team().monsters().iter().any(|it| it.uid == uid)
    }

    pub fn are_opponents(&self, owner_uid: MonsterUID, event_caller_uid: MonsterUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_opponent_team(event_caller_uid))
            || (self.is_on_ally_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    pub fn are_allies(&self, owner_uid: MonsterUID, event_caller_uid: MonsterUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_ally_team(event_caller_uid))
            || (self.is_on_opponent_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    pub fn active_monsters(&self) -> PerTeam<&Monster> {
        let ally_team_active_monster = self.active_monsters_on_team(TeamID::Allies);
        let opponent_team_active_monster = self.active_monsters_on_team(TeamID::Opponents);
        PerTeam::new(ally_team_active_monster, opponent_team_active_monster)
    }

    /// Returns a singular monster for now. TODO: This will need to updated for double and multi battle support.
    pub fn active_monsters_on_team(&self, team_id: TeamID) -> &Monster {
        self.monster(self.active_monster_uids[team_id])
    }

    /// Given an action choice, computes its activation order. This is handled by `Battle` because the order is context sensitive.
    pub(crate) fn choice_activation_order(&self, choice: FullySpecifiedAction) -> ActivationOrder {
        match choice {
            FullySpecifiedAction::Move { move_uid, target_uid: _ } => ActivationOrder {
                priority: self.move_(move_uid).species.priority,
                speed: self.monster(move_uid.owner_uid).stats[Stat::Speed],
                order: 0, //TODO: Think about ordering
            },
            FullySpecifiedAction::SwitchOut { switcher_uid: active_monster_uid, switchee_uid: _ } => ActivationOrder { 
                priority: 8, 
                speed: self.monster(active_monster_uid).stats[Stat::Speed], 
                order: 0
            }
        }
    }

    pub fn available_actions(&self) -> AvailableActions {
        let ally_team_available_actions = self.available_actions_by_team(TeamID::Allies);
        let opponent_team_available_actions = self.available_actions_by_team(TeamID::Opponents);

        AvailableActions {
            ally_team_available_actions,
            opponent_team_available_actions,
        }
    }

    fn available_actions_by_team(&self, team_id: TeamID) -> AvailableActionsForTeam {
        
        let team_active_monster = self.active_monsters_on_team(team_id);
        let team = self.team(team_id);
        
        let moves = team_active_monster.move_uids();
        let mut move_actions = Vec::with_capacity(4);
        for move_uid in moves {
            let partial_action = PartiallySpecifiedAction::Move { 
                move_uid,
                target_uid: self.active_monsters_on_team(team_id.other()).uid,
                display_text: self.move_(move_uid).species.name 
            };
            move_actions.push(partial_action);
        }

        let any_benched_ally_monsters = team.monsters().len() > 1;
        let switch_action = if any_benched_ally_monsters {
            Some(PartiallySpecifiedAction::SwitchOut { 
                switcher_uid: team_active_monster.uid, 
                possible_switchee_uids: self.valid_switchees_by_uid(team_id),
            })
        } else {
            None
        };

        AvailableActionsForTeam::new(
            &move_actions, 
            switch_action,
        )
    }

    pub fn push_message_to_log(&mut self, message: &str) {
        self.message_log.push(message.to_string());
    }

    pub fn push_messages_to_log(&mut self, messages: &[&str]) {
        for message in messages {
            self.message_log.push(message.to_string());
        }
    }

    /// Returns an array of options where all the `Some` variants are at the beginning.
    pub(crate) fn valid_switchees_by_uid(&self, team_id: TeamID) -> ArrayOfOptionals<MonsterUID, 5> {
        let mut number_of_switchees = 0;
        let mut switchees = [None; 5];
        for monster in self.team(team_id).monsters().iter() {
            let is_active_monster_for_team = monster.uid == self.active_monster_uids[team_id];
            let is_valid_switch_partner = not!(self.monster(monster.uid).is_fainted) && not!(is_active_monster_for_team);
            if is_valid_switch_partner {
                switchees[number_of_switchees] = Some(monster.uid);
                number_of_switchees += 1;
                assert!(number_of_switchees < 6);
            }
        }
        switchees
    }

    pub fn team(&self, team_id: TeamID) -> &MonsterTeam {
        & self.teams[team_id]
    }

    pub fn team_mut(&mut self, team_id: TeamID) -> &mut MonsterTeam {
        &mut self.teams[team_id]
    }

    pub fn ally_team(&self) -> &MonsterTeam {
        &self.teams[TeamID::Allies]
    }

    pub fn ally_team_mut(&mut self) -> &mut MonsterTeam {
        &mut self.teams[TeamID::Allies]
    }

    pub fn opponent_team(&self) -> &MonsterTeam {
        &self.teams[TeamID::Opponents]
    }

    pub fn opponent_team_mut(&mut self) -> &mut MonsterTeam {
        &mut self.teams[TeamID::Opponents]
    }
}

impl Display for Battle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        push_pretty_tree_for_team(
            &mut out,
            "Ally Team\n", 
            self.ally_team(), 
            self.ally_team().monsters().iter().count(),
        );
        push_pretty_tree_for_team(
            &mut out,
            "Opponent Team\n",
            self.opponent_team(),
            self.opponent_team().monsters().iter().count(),
        );
        write!(f, "{}", out)
    }
}

fn push_pretty_tree_for_team(output_string: &mut String, team_name: &str, team: &MonsterTeam, number_of_monsters: usize) {
    output_string.push_str(team_name);
    for (i, monster) in team.monsters().iter().enumerate() {
        let is_not_last_monster = i < number_of_monsters - 1;
        let (prefix_str, suffix_str) = if is_not_last_monster {
            ("\t│\t", "├── ")
        } else {
            ("\t \t", "└── ")
        };
        output_string.push_str(&("\t".to_owned() + suffix_str));
        output_string.push_str(&monster.status_string());
        output_string.push_str(&(prefix_str.to_owned() + "│\n"));
        output_string.push_str(&(prefix_str.to_owned() + "├── "));

        let primary_type = monster.species.primary_type;
        let secondary_type = monster.species.secondary_type;
        let type_string = if let Some(secondary_type) = secondary_type {
            format!["   type: {:?}/{:?}\n", primary_type, secondary_type]
        } else {
            format!["   type: {:?}\n", primary_type]
        };
        output_string.push_str(&type_string);

        output_string.push_str(&(prefix_str.to_owned() + "├── "));
        output_string.push_str(format!["ability: {}\n", monster.ability.species.name].as_str());

        let number_of_moves = monster.moveset.moves().count();
        for (j, move_) in monster.moveset.moves().enumerate() {
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