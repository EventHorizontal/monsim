mod message_log;

use utils::{not, Ally, MaxSizedVec, Opponent, Team};

use crate::sim::{
        AbilityInternal, FullySpecifiedChoice, ActivationOrder, AvailableChoices, MonsterInternal,
        MonsterTeamInternal, MonsterUID, Move, MoveUID, Stat, AvailableChoicesForTeam,
        utils,
};

use std::{cell::Cell, fmt::Display};

use super::{event::OwnedEventHandlerDeck, prng::{self, Prng}, Ability, Monster, MonsterTeam, MoveSet, PartiallySpecifiedChoice, PerTeam, TeamUID, ALLY_1, OPPONENT_1};
use message_log::MessageLog;

/// The main data struct that contains all the information one could want to know about the current battle. This is meant to be passed around as a unit and queried for battle-related information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battle {
    // Sub-objects
    pub prng: Prng,
    pub message_log: MessageLog,
    // Data
    pub is_finished: bool,
    pub turn_number: u16,
    // Mechanics
    active_monsters: PerTeam<Cell<MonsterUID>>,
    monsters: PerTeam<MaxSizedVec<Cell<MonsterInternal>, 6>>,
    abilities: MaxSizedVec<Cell<AbilityInternal>,12>,

    teams: PerTeam<MonsterTeamInternal>,
    // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
}

impl Battle {
    
    pub fn new(
        ally_monsters: Vec<Cell<MonsterInternal>>, 
        opponent_monsters: Vec<Cell<MonsterInternal>>, 
        moves: Vec<Cell<Move>>, 
        abilities: Vec<Cell<AbilityInternal>>
    ) -> Self {
        
        let ally_monster_uids = ally_monsters.iter()
            .map(|monster| { monster.get().uid })
            .collect::<Vec<_>>();

        let opponent_monster_uids = opponent_monsters.iter()
            .map(|monster| { monster.get().uid })
            .collect::<Vec<_>>();

        Self {
            prng: Prng::new(prng::seed_from_time_now()),
            message_log: MessageLog::new(),
            
            is_finished: false,
            turn_number: 0,

            active_monsters: PerTeam::new(Ally(Cell::new(ALLY_1)), Opponent(Cell::new(OPPONENT_1))),
            monsters: PerTeam::new(
                Ally(MaxSizedVec::from_vec_with_default_padding(ally_monsters)), 
                Opponent(MaxSizedVec::from_vec_with_default_padding(opponent_monsters))
            ),
            abilities: MaxSizedVec::from_vec_with_default_padding(abilities),
            teams: PerTeam::new(
                    Ally(MonsterTeamInternal::new(MaxSizedVec::from_vec_with_default_padding(ally_monster_uids), TeamUID::Allies)),
                    Opponent(MonsterTeamInternal::new(MaxSizedVec::from_vec_with_default_padding(opponent_monster_uids), TeamUID::Opponents)),
                ),
        }
    }

    /* #region Teams ##################################################################### */
    pub fn team(&self, team_uid: TeamUID) -> Team<MonsterTeam> {
        match team_uid {
            TeamUID::Allies => self.monsters.ally_ref().map(|monsters| {
                    MonsterTeam::new(self.active_monsters.ally_ref().unwrap(), monsters, TeamUID::Allies)
                }).into(),
            TeamUID::Opponents => self.monsters.opponent_ref().map(|monsters| {
                    MonsterTeam::new(self.active_monsters.opponent_ref().unwrap(), monsters, TeamUID::Opponents)
                }).into(),
        }
    }

    pub fn team_mut(&mut self, team_uid: TeamUID) -> Team<&mut MonsterTeamInternal> {
        match team_uid {
            TeamUID::Allies => Team::Ally(self.teams.ally_mut()),
            TeamUID::Opponents => Team::Opponent(self.teams.opponent_mut()),
        }
    }

    pub fn ally_team(&self) -> Ally<MonsterTeam> {
        self.team(TeamUID::Allies)
            .expect_ally()
    }

    pub fn opponent_team(&self) -> Opponent<MonsterTeam> {
        self.team(TeamUID::Opponents)
            .expect_opponent()
    }

    pub fn is_on_ally_team(&self, uid: MonsterUID) -> bool {
        self.ally_team().monsters().iter().any(|it| it.get().uid == uid)
    }

    pub fn is_on_opponent_team(&self, uid: MonsterUID) -> bool {
        self.opponent_team().monsters().iter().any(|it| it.get().uid == uid)
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

    /* #endregion ########################################################################## */
    /* #region Ability ##################################################################### */

    fn ability(&self, owner_uid: MonsterUID) -> Ability {
        Ability::new(
            &self.abilities
                .iter()
                .find(|ability| { ability.get().owner_uid == owner_uid })
                .expect("Expected the MonsterUID given to be pre-checked.")
        )
    }

    /* #endregion ########################################################################## */
    /* #region Moves # ##################################################################### */

    pub fn moveset(&self, owner_uid: MonsterUID) -> MoveSet {
        
    }

    pub fn move_(&self, move_uid: MoveUID) -> &Move {
        self.monster(move_uid.owner_uid)
            .get()
            .moveset
            .move_(move_uid.move_number)
    }
    
    /* #endregion ########################################################################## */
    /* #region Monster ##################################################################### */

    pub fn monsters(&self) -> impl Iterator<Item = Monster> {
        let ally_monsters = self.monsters.ally_ref();
        let opponent_monsters = self.monsters.opponent_ref();
        ally_monsters.iter().chain(opponent_monsters.iter()).map(|monster| {
            let monster_uid = monster.get().uid;
            Monster::new(monster, self.moveset(monster_uid), self.ability(monster_uid))
        })
    }

    pub fn monster(&self, monster_uid: MonsterUID) -> Monster {
        self.monsters()
            .find(|monster| { monster.get().uid == monster_uid })
            .expect("Expected the MonsterUID given to be pre-checked.")
    }

    pub fn active_monsters(&self) -> PerTeam<&Cell<MonsterInternal>> {
        self.active_monsters.map(|monster_uid| {
            self.monster(monster_uid.get())
        })
    }

    #[deprecated]
    pub(crate) fn active_monster_uids(&self) -> PerTeam<MonsterUID> {
        self.active_monsters().map(|monster| { monster.get().uid })
    }

    pub fn is_active_monster(&self, monster: &Cell<MonsterInternal>) -> bool {
        let monster = monster.get();
        self.teams[monster.uid.team_uid].active_monster_uid == monster.uid
    }

    /* #endregion ########################################################################## */
    
    pub fn event_handler_deck_instances(&self) -> Vec<OwnedEventHandlerDeck> {
        let mut out = Vec::new();
        out.append(&mut self.ally_team().event_handler_deck_instances());
        out.append(&mut self.opponent_team().event_handler_deck_instances());
        out
    }

    // Choice -------------------------------------

    /// Given an action choice, computes its activation order. This is handled by `Battle` because the order is context sensitive.
    pub(crate) fn choice_activation_order(&self, choice: FullySpecifiedChoice) -> ActivationOrder {
        match choice {
            FullySpecifiedChoice::Move { move_uid, target_uid: _ } => ActivationOrder {
                priority: self.move_(move_uid).species.priority,
                speed: self.monster(move_uid.owner_uid).get().stats[Stat::Speed],
                order: 0, //TODO: Think about how to restrict order to be mutually exclusive
            },
            FullySpecifiedChoice::SwitchOut { switcher_uid, candidate_switchee_uids: _ } => ActivationOrder { 
                priority: 8, 
                speed: self.monster(switcher_uid).get().stats[Stat::Speed], 
                order: 0
            }
        }
    }

    pub fn available_choices(&self) -> AvailableChoices {
        let ally_team_available_actions = self.available_choices_for_team(TeamUID::Allies);
        let opponent_team_available_actions = self.available_choices_for_team(TeamUID::Opponents);

        AvailableChoices {
            ally_team_available_choices: ally_team_available_actions,
            opponent_team_available_choices: opponent_team_available_actions,
        }
    }

    fn available_choices_for_team(&self, team_uid: TeamUID) -> AvailableChoicesForTeam {
        
        let active_monster_for_team = self.team(team_uid).map(|team| { 
            self.monster(team.active_monster_uid) 
        });
        let active_monster_for_other_team = self.team(team_uid.other()).map(|team| { 
            self.monster(team.active_monster_uid) 
        });
        
        let moves = active_monster_for_team.get().move_uids();
        let mut move_actions = Vec::with_capacity(4);
        for move_uid in moves {
            let partially_specified_choice = PartiallySpecifiedChoice::Move { 
                move_uid,
                target_uid: active_monster_for_other_team.get().uid,
                display_text: self.move_(move_uid).species.name 
            };
            move_actions.push(partially_specified_choice);
        }

        let candidate_switchee_uids = self.valid_switchees_by_uid(team_uid);
        let any_valid_switchees = not!(candidate_switchee_uids.iter().count() == 0 );
        let switch_action = if any_valid_switchees {
            Some(PartiallySpecifiedChoice::SwitchOut { 
                switcher_uid: active_monster_for_team.get().uid, 
                candidate_switchee_uids,
                display_text: "Switch Out",
            })
        } else {
            None
        };

        AvailableChoicesForTeam::new(
            &move_actions, 
            switch_action,
        )
    }

    /// Returns an array of options where all the `Some` variants are at the beginning.
    pub(crate) fn valid_switchees_by_uid(&self, team_uid: TeamUID) -> Vec<MonsterUID> {
        let candidate_switchees = self.team(team_uid)
            .monsters()
            .iter()
            .filter_map(|monster| {
                let is_active_monster_for_team = monster.get().uid == self.teams[team_uid].active_monster_uid;
                let is_valid_switch_partner = not!(monster.get().is_fainted) && not!(is_active_monster_for_team);
                if is_valid_switch_partner {
                    Some(monster.get().uid)
                } else {
                    None
                }
            }).collect::<Vec<_>>();
        candidate_switchees
    }
}

impl Display for Battle {
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

fn push_pretty_tree_for_team(output_string: &mut String, team_name: &str, team: &MonsterTeamInternal, number_of_monsters: usize) {
    output_string.push_str(team_name);
    for (i, monster) in team.monsters().iter().enumerate() {
        let is_not_last_monster = i < number_of_monsters - 1;
        let (prefix_str, suffix_str) = if is_not_last_monster {
            ("\t│\t", "├── ")
        } else {
            ("\t \t", "└── ")
        };
        output_string.push_str(&("\t".to_owned() + suffix_str));
        output_string.push_str(&monster.get().status_string());
        output_string.push_str(&(prefix_str.to_owned() + "│\n"));
        output_string.push_str(&(prefix_str.to_owned() + "├── "));

        let primary_type = monster.get().species.primary_type;
        let secondary_type = monster.get().species.secondary_type;
        let type_string = if let Some(secondary_type) = secondary_type {
            format!["   type: {:?}/{:?}\n", primary_type, secondary_type]
        } else {
            format!["   type: {:?}\n", primary_type]
        };
        output_string.push_str(&type_string);

        output_string.push_str(&(prefix_str.to_owned() + "├── "));
        output_string.push_str(format!["ability: {}\n", monster.get().ability.species.name].as_str());

        let number_of_moves = monster.get().moveset.moves().count();
        for (j, move_) in monster.get().moveset.moves().enumerate() {
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