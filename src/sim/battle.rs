mod message_log;
pub(super) mod builders;

use std::fmt::Display;
use monsim_utils::{not, Ally, MaxSizedVec, Opponent};
use crate::sim::{
        Ability, ActivationOrder, AvailableChoicesForTeam, Monster, MonsterTeam, MonsterUID, Move, MoveUID, Stat
};

use super::{event::OwnedEventHandlerDeck, prng::Prng, PartiallySpecifiedChoice, PerTeam, TeamUID};
use message_log::MessageLog;

/// The main data struct that contains all the information one could want to know about the current battle. This is meant to be passed around as a unit and queried for battle-related information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleState {

    pub(crate) prng: Prng,
    pub turn_number: u16,
    pub is_finished: bool,
    // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
    pub message_log: MessageLog,
    
    teams: PerTeam<MonsterTeam>,
}

impl BattleState {

    pub(crate) fn new(ally_team: MonsterTeam, opponent_team: MonsterTeam) -> Self {
        let teams = PerTeam::new(Ally::new(ally_team), Opponent::new(opponent_team));
        Self {
            prng: Prng::from_current_time(),
            is_finished: false,
            turn_number: 0,
            teams,
            message_log: MessageLog::new(),
        }
    }

    // Teams -----------------

    pub fn team(&self, team_uid: TeamUID) -> &MonsterTeam {
        & self.teams[team_uid]
    }

    pub fn team_mut(&mut self, team_uid: TeamUID) -> &mut MonsterTeam {
        &mut self.teams[team_uid]
    }

    pub fn ally_team(&self) -> Ally<&MonsterTeam> {
        self.teams.ally_ref()
    }

    pub fn ally_team_mut(&mut self) -> Ally<&mut MonsterTeam> {
        self.teams.ally_mut()
    }

    pub fn opponent_team(&self) -> Opponent<&MonsterTeam> {
        self.teams.opponent_ref()
    }

    pub fn opponent_team_mut(&mut self) -> Opponent<&mut MonsterTeam> {
        self.teams.opponent_mut()
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

    pub fn event_handler_deck_instances(&self) -> Vec<OwnedEventHandlerDeck> {
        let mut out = Vec::new();
        out.append(&mut self.ally_team().event_handler_deck_instances());
        out.append(&mut self.opponent_team().event_handler_deck_instances());
        out
    }

    // Monsters -----------------

    pub fn monsters(&self) -> impl Iterator<Item = &Monster> {
        let (ally_team, opponent_team) = self.teams.unwrap_ref();
        ally_team.monsters().iter().chain(opponent_team.monsters().iter())
    }

    pub fn monsters_mut(&mut self) -> impl Iterator<Item = &mut Monster> {
        let (ally_team, opponent_team) = self.teams.unwrap_mut();
        ally_team.monsters_mut().iter_mut().chain(opponent_team.monsters_mut().iter_mut())
    }

    pub fn monster(&self, monster_uid: MonsterUID) -> &Monster {
        let team = self.team(monster_uid.team_uid);
        &team[monster_uid.monster_number]
    }

    pub fn monster_mut(&mut self, monster_uid: MonsterUID) -> &mut Monster {
        let team = self.team_mut(monster_uid.team_uid);
        &mut team[monster_uid.monster_number]
    }

    pub fn active_monsters(&self) -> PerTeam<&Monster> {
        let ally_team_active_monster = self.ally_team().map_consume(|team| { self.monster(team.active_monster_uid) });
        let opponent_team_active_monster = self.opponent_team().map_consume(|team| { self.monster(team.active_monster_uid) });
        PerTeam::new(ally_team_active_monster, opponent_team_active_monster)
    }

    pub(crate) fn active_monster_uids(&self) -> PerTeam<MonsterUID> {
        self.active_monsters().map_consume(|monster| { monster.uid })
    }

    /// Returns a singular monster for now. TODO: This will need to updated for double and multi battle support.
    pub fn active_monsters_on_team(&self, team_uid: TeamUID) -> &Monster {
        self.monster(self.teams[team_uid].active_monster_uid)
    }

    pub fn is_active_monster(&self, monster_uid: MonsterUID) -> bool {
        self.teams[monster_uid.team_uid].active_monster_uid == monster_uid
    }

    // Abilities -----------------

    pub fn ability(&self, owner_uid: MonsterUID) -> &Ability {
        &self.monster(owner_uid)
            .ability
    }

    pub fn ability_mut(&mut self, owner_uid: MonsterUID) -> &mut Ability {
        &mut self
            .monster_mut(owner_uid)
            .ability
    }

    // Moves -----------------

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

    // Choice -------------------------------------

    pub fn available_choices(&self) -> PerTeam<AvailableChoicesForTeam> {
        self.teams.map_clone(|team| {
            self.available_choices_for_team(team.id)
        })
    }

    fn available_choices_for_team(&self, team_uid: TeamUID) -> AvailableChoicesForTeam {
        
        let active_monster_on_team = self.active_monsters_on_team(team_uid);
        
        // Move choices
        let moves = active_monster_on_team.move_uids();
        let mut move_actions = Vec::with_capacity(4);
        for move_uid in moves {
            let partially_specified_choice = PartiallySpecifiedChoice::Move { 
                attacker_uid: move_uid.owner_uid,
                move_uid,
                target_uid: self.active_monsters_on_team(team_uid.other()).uid,
                activation_order: ActivationOrder {
                    priority: self.move_(move_uid).species.priority,
                    speed: self.monster(move_uid.owner_uid).stats[Stat::Speed],
                    order: 0, //TODO: Think about how to restrict order to be mutually exclusive
                },
                display_text: self.move_(move_uid).species.name 
            };
            move_actions.push(partially_specified_choice);
        }

        // Switch choice
        let switchable_benched_monster_uids = self.switchable_benched_monster_uids(team_uid);
        let any_switchable_monsters = not!(switchable_benched_monster_uids.is_empty());
        let switch_action = if any_switchable_monsters {
            Some(PartiallySpecifiedChoice::SwitchOut { 
                active_monster_uid: active_monster_on_team.uid, 
                switchable_benched_monster_uids,
                activation_order: ActivationOrder { 
                    priority: 8, 
                    speed: self.monster(active_monster_on_team.uid).stats[Stat::Speed], 
                    order: 0
                },
                display_text: "Switch Out",
            })
        } else {
            None
        };

        AvailableChoicesForTeam::new(
            move_actions, 
            switch_action,
        )
    }

    // TODO: Once we have multitargeting/multiple active monsters, if one monster has selected
    // to switch out with a particular benched monster, that benched monster will need to be excluded.
    // Perhaps the ui will take care of that though?
    /// Returns an array of options where all the `Some` variants are at the beginning.
    pub(crate) fn switchable_benched_monster_uids(&self, team_uid: TeamUID) -> MaxSizedVec<MonsterUID, 5> {
        let mut number_of_switchees = 0;
        let mut switchable_benched_monsters = Vec::with_capacity(5);
        for monster in self.team(team_uid).monsters().iter() {
            let is_active_monster_for_team = monster.uid == self.teams[team_uid].active_monster_uid;
            let is_valid_switch_partner = not!(self.monster(monster.uid).is_fainted) && not!(is_active_monster_for_team);
            if is_valid_switch_partner {
                switchable_benched_monsters.push(monster.uid);
                number_of_switchees += 1;
                assert!(number_of_switchees < 6);
            }
        }
        MaxSizedVec::from_vec(switchable_benched_monsters)
    }
}

impl Display for BattleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        push_pretty_tree_for_team(
            &mut out,
            "Ally Team\n", 
            *self.ally_team(), 
            self.ally_team().monsters().iter().count(),
        );
        push_pretty_tree_for_team(
            &mut out,
            "Opponent Team\n",
            *self.opponent_team(),
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