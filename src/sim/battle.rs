mod message_log;

use utils::{not, Ally, MaxSizedVec, Opponent, TeamAffil};

use crate::sim::{
        AbilityData, FullySpecifiedChoice, ActivationOrder, AvailableChoices, MonsterData,
        MonsterTeamInternal, MonsterUID, MoveData, MoveUID, Stat, AvailableChoicesForTeam,
        utils,
};

use std::{cell::Cell, fmt::Display};

use super::{event::OwnedEventHandlerDeck, prng::{self, Prng}, Ability, AbilityUID, EventFilteringOptions, Monster, MonsterTeam, Move, MoveSet, PartiallySpecifiedChoice, PerTeam, TeamUID, ALLY_1, OPPONENT_1};
use message_log::MessageLog;

/// The main data struct that contains all the information one could want to know about the current battle. This is meant to be passed around as a unit and queried for battle-related information.
#[derive(Debug, Clone)]
pub struct Battle {
    // Sub-objects
    pub prng: Prng,
    pub message_log: MessageLog,
    // Data
    pub is_finished: bool,
    pub turn_number: u16,
    // Mechanics
    active_monsters: PerTeam<Cell<MonsterUID>>,
    monsters: PerTeam<MaxSizedVec<Cell<MonsterData>,6>>,
    abilities: MaxSizedVec<Cell<AbilityData>,12>,
    moves: MaxSizedVec<Cell<MoveData>,48>,

    placeholder_move: Cell<MoveData>,

    teams: PerTeam<MonsterTeamInternal>,
    // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
}

impl Battle {
    
    pub fn new(
        ally_monsters: Vec<Cell<MonsterData>>, 
        opponent_monsters: Vec<Cell<MonsterData>>, 
        moves: Vec<Cell<MoveData>>, 
        abilities: Vec<Cell<AbilityData>>
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
            moves: MaxSizedVec::from_vec_with_default_padding(moves),
            placeholder_move: Cell::new(MoveData::default()),
            teams: PerTeam::new(
                    Ally(MonsterTeamInternal::new(MaxSizedVec::from_vec_with_default_padding(ally_monster_uids), TeamUID::Allies)),
                    Opponent(MonsterTeamInternal::new(MaxSizedVec::from_vec_with_default_padding(opponent_monster_uids), TeamUID::Opponents)),
                ),
        }
    }

    /* #region Teams ##################################################################### */
    pub fn team(&self, team_uid: TeamUID) -> TeamAffil<MonsterTeam> {
        match team_uid {
            TeamUID::Allies => self.monsters.ally_ref().map(|monsters| {
                    let monsters = monsters.map(|monster| { self.monster(monster.get().uid) } ); 
                    MonsterTeam::new(&self.active_monsters.ally_ref(), monsters, TeamUID::Allies)
                }).into(),
                TeamUID::Opponents => self.monsters.opponent_ref().map(|monsters| {
                    let monsters = monsters.map(|monster| { self.monster(monster.get().uid) } ); 
                    MonsterTeam::new(&self.active_monsters.opponent_ref(), monsters, TeamUID::Opponents)
                }).into(),
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

    pub fn is_on_team(&self, monster: Monster, team: TeamUID) -> bool {
        self.team(team).monsters().iter().any(|it| monster.is(it.uid()))
    }

    pub fn are_opponents(&self, monster: Monster, other_monster: Monster) -> bool {
        if monster == other_monster {
            return false;
        }

        self.is_on_team(other_monster, monster.team())
    }

    pub fn are_allies(&self, monster: Monster, other_monster: Monster) -> bool {
        // a Monster is not considered its own Ally
        if monster == other_monster {
            return false;
        }

        self.is_on_team(other_monster, monster.team().other())
    }

    /* #endregion ########################################################################## */
    /* #region Ability ##################################################################### */

    fn ability(&self, owner_uid: MonsterUID) -> Ability {
        Ability::new(
            owner_uid,
            &self.abilities
                .iter()
                .find(|ability| { ability.get().uid == owner_uid })
                .expect("Expected the MonsterUID given to be pre-checked.")
        )
    }

    
    pub(crate) fn ability_event_handler_deck(&self, ability_uid: AbilityUID) -> OwnedEventHandlerDeck {
        
        let ability_owner = self.monster(ability_uid);
        let ability = self.ability(ability_uid);

        let activation_order = ActivationOrder {
            priority: 0,
            speed: ability_owner.stat(Stat::Speed),
            order: ability.species().order,
        };
        OwnedEventHandlerDeck {
            event_handler_deck: ability.event_handler_deck(),
            owner_uid: ability_uid,
            activation_order,
            filtering_options: EventFilteringOptions::default(),
        }
    }

    /* #endregion ########################################################################## */
    /* #region Moves # ##################################################################### */

    fn move_(&self, move_uid: MoveUID) -> Move {
        let move_ = self.moves
            .iter()
            .find(|move_| { move_.get().uid == move_uid })
            .expect("Expected the MoveUID to be pre-checked.");
        Move::new(move_uid, move_)
    }
    
    pub fn moveset(&self, owner_uid: MonsterUID) -> MaxSizedVec<Move, 4> {
        let moves = self.moves
            .iter()
            .filter_map(|move_| { 
                let move_uid = move_.get().uid; 
                if move_uid.owner_uid == owner_uid {
                    Some(Move::new(move_uid, move_))
                } else {
                    None
                }
            } )
            .collect::<Vec<_>>();
        
        let placeholder_element = Move::new(MoveUID { owner_uid, move_number: super::MoveNumber::_1 }, &self.placeholder_move);

        MaxSizedVec::from_vec(moves, placeholder_element)    
    }

    pub(crate) fn move_event_handler_deck(&self, move_uid: MoveUID) -> OwnedEventHandlerDeck {
        let move_ = self.move_(move_uid);
        let move_owner = self.monster(move_uid.owner_uid);
        OwnedEventHandlerDeck {
            event_handler_deck: move_.species().event_handler_deck,
            owner_uid: move_uid.owner_uid,
            activation_order: ActivationOrder {
                priority: move_.species().priority,
                speed: move_owner.stat(Stat::Speed),
                order: 0,
            },
            filtering_options: EventFilteringOptions::default(),
        }
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
            .find(|monster| { monster.is(monster_uid) })
            .expect("Expected the MonsterUID given to be pre-checked.")
    }

    pub fn active_monsters(&self) -> PerTeam<Monster> {
        self.active_monsters.map(|monster_uid| {
            self.monster(monster_uid.get())
        })
    }

    pub fn is_active_monster(&self, monster: Monster) -> bool {
        monster.is(self.teams[monster.uid().team_uid].active_monster_uid)
    }

    pub(crate) fn monster_event_handler_deck(&self, monster_uid: MonsterUID) -> OwnedEventHandlerDeck {
        let monster = self.monster(monster_uid);
        let activation_order = ActivationOrder {
            priority: 0,
            speed: monster.stat(Stat::Speed),
            order: 0,
        };
        OwnedEventHandlerDeck {
            event_handler_deck: monster.species().event_handler_deck,
            owner_uid: monster.uid(),
            activation_order,
            filtering_options: EventFilteringOptions::default(),
        }
    }

    /* #endregion ########################################################################## */
    
    pub fn event_handler_decks(&self) -> Vec<OwnedEventHandlerDeck> {
        const MAX_POSSIBLE_EVENT_HANDLERS: usize = 72;
        let mut out = Vec::with_capacity(MAX_POSSIBLE_EVENT_HANDLERS);
        let (mut ally_monster_event_handler_decks, mut opponent_monster_event_handler_decks) = self.monsters.map(|monsters|
            monsters
                .iter()
                .map(|monster| {
                    self.monster_event_handler_deck(monster.get().uid)
                })
                .collect::<Vec<_>>()
        ).unwrap();
        
        let mut move_event_handler_decks = self.moves
            .iter()
            .map(|move_| {
                self.move_event_handler_deck(move_.get().uid)
            })
            .collect::<Vec<_>>();

        let mut ability_event_handler_decks = self.abilities
            .iter()
            .map(|ability| {
                self.ability_event_handler_deck(ability.get().uid)
            })
            .collect::<Vec<_>>();
        
        out.append(&mut ally_monster_event_handler_decks);
        out.append(&mut opponent_monster_event_handler_decks);
        out.append(&mut move_event_handler_decks);
        out.append(&mut ability_event_handler_decks);
        
        out
    }

    /* #region Choices ##################################################################### */
    
    /// Given an action choice, computes its activation order. This is handled by `Battle` because the order is context sensitive.
    pub(crate) fn choice_activation_order(&self, choice: FullySpecifiedChoice) -> ActivationOrder {
        match choice {
            FullySpecifiedChoice::SwitchOut { switcher_uid, candidate_switchee_uids: _ } => ActivationOrder { 
                priority: 8, 
                speed: self.monster(switcher_uid).stat(Stat::Speed), 
                order: 0
            },
            FullySpecifiedChoice::Move { move_uid, target_uid: _ } => ActivationOrder {
                priority: self.move_(move_uid).species().priority,
                speed: self.monster(move_uid.owner_uid).stat(Stat::Speed),
                order: 0, //TODO: Think about how to restrict order to be mutually exclusive
            },
        }
    }

    pub fn available_choices(&self) -> AvailableChoices {
        let ally_team_available_choices = self.available_choices_for_team(TeamUID::Allies);
        let opponent_team_available_choices = self.available_choices_for_team(TeamUID::Opponents);

        AvailableChoices {
            ally_team_available_choices,
            opponent_team_available_choices,
        }
    }

    fn available_choices_for_team(&self, team_uid: TeamUID) -> AvailableChoicesForTeam {
        
        let active_monster_for_team = self.team(team_uid).map(|team| { team.active_monster() });
        let active_monster_for_other_team = self.team(team_uid.other()).map(|team| { team.active_monster() });
        
        let move_choices = active_monster_for_team.moveset()
            .iter()
            .map(|move_| {
                PartiallySpecifiedChoice::Move { 
                    move_uid: move_.uid(),
                    target_uid: active_monster_for_other_team.uid(),
                    display_text: move_.species().name 
                }
            })
            .collect::<Vec<_>>();
        
        let candidate_switchee_uids = self.valid_switchees_by_uid(team_uid);
        let are_there_any_valid_switchees = not!(candidate_switchee_uids.iter().count() == 0);
        let switch_choice = are_there_any_valid_switchees.then_some(PartiallySpecifiedChoice::SwitchOut { 
            switcher_uid: active_monster_for_team.uid(), 
            candidate_switchee_uids,
            display_text: "Switch Out",
        });

        AvailableChoicesForTeam::new(
            &move_choices, 
            switch_choice,
        )
    }

    /// Returns an array of options where all the `Some` variants are at the beginning.
    pub(crate) fn valid_switchees_by_uid(&self, team_uid: TeamUID) -> Vec<MonsterUID> {
        let candidate_switchees = self.team(team_uid)
            .monsters()
            .iter()
            .filter_map(|monster| {
                let is_active_monster_for_team = monster.is(self.teams[team_uid].active_monster_uid);
                let is_valid_switch_partner = not!(monster.is_fainted()) && not!(is_active_monster_for_team);
                if is_valid_switch_partner {
                    Some(monster.uid())
                } else {
                    None
                }
            }).collect::<Vec<_>>();
        candidate_switchees
    }
    
    /* #endregion ########################################################################## */
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

fn push_pretty_tree_for_team(output_string: &mut String, team_name: &str, team: MonsterTeam, number_of_monsters: usize) {
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

        let primary_type = monster.species().primary_type;
        let secondary_type = monster.species().secondary_type;
        let type_string = if let Some(secondary_type) = secondary_type {
            format!["   type: {:?}/{:?}\n", primary_type, secondary_type]
        } else {
            format!["   type: {:?}\n", primary_type]
        };
        output_string.push_str(&type_string);

        output_string.push_str(&(prefix_str.to_owned() + "├── "));
        output_string.push_str(format!["ability: {}\n", monster.ability().species().name].as_str());

        let number_of_moves = monster.moveset().len();
        for (j, move_) in monster.moveset().iter().enumerate() {
            let is_not_last_move = j < number_of_moves - 1;
            if is_not_last_move {
                output_string.push_str(&(prefix_str.to_owned() + "├── "));
            } else {
                output_string.push_str(&(prefix_str.to_owned() + "└── "));
            }
            output_string.push_str(format!["   move: {}\n", move_.species().name].as_str());
        }
        output_string.push_str(&(prefix_str.to_owned() + "\n"));
    }
}