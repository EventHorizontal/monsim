mod message_log;
mod prng;

use utils::{not, Ally, MaxSizedVec, Opponent, TeamAffil};
use std::{cell::Cell, fmt::Display};

use super::{event::OwnedEventHandlerDeck, AbilityRef, AbilityUID, EventFilteringOptions, MonsterRef, MonsterTeam, MoveRef, PartiallySpecifiedChoice, PerTeam, TeamUID, ALLY_1, OPPONENT_1, Ability, FullySpecifiedChoice, ActivationOrder, Monster,MonsterUID, Move, MoveUID, Stat, AvailableChoicesForTeam,
    utils,};

use message_log::MessageLog;
pub use prng::*;

/// The main data struct that contains all the information one could want to know about the current battle. This is meant to be passed around as a unit and queried for battle-related information.
#[derive(Debug, Clone)]
pub struct Battle {
    // Sub-objects
    pub prng: Prng,
        // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
    pub message_log: MessageLog,
    // Data
    pub is_finished: bool,
    pub turn_number: u16,
    // Mechanics
    active_monsters: PerTeam<Cell<MonsterUID>>,
    monsters: PerTeam<MaxSizedVec<Monster,6>>,
    abilities: MaxSizedVec<Ability,12>,
    moves: MaxSizedVec<Move,48>,
}

impl Battle {
    
    pub fn new(
        ally_monsters: Vec<Monster>, 
        opponent_monsters: Vec<Monster>, 
        moves: Vec<Move>, 
        abilities: Vec<Ability>
    ) -> Self {
        Self {
            prng: Prng::new(prng::seed_from_time_now()),
            message_log: MessageLog::new(),
            
            is_finished: false,
            turn_number: 0,

            active_monsters: PerTeam::new(Ally(Cell::new(ALLY_1)), Opponent(Cell::new(OPPONENT_1))),
            monsters: PerTeam::new(
                Ally(MaxSizedVec::from_vec(ally_monsters)), 
                Opponent(MaxSizedVec::from_vec(opponent_monsters))
            ),
            abilities: MaxSizedVec::from_vec(abilities),
            moves: MaxSizedVec::from_vec(moves),
        }
    }

    /* #region Teams ##################################################################### */
    pub fn team(&self, team_uid: TeamUID) -> TeamAffil<MonsterTeam> {
        match team_uid {
            TeamUID::Allies => self.monsters.ally_ref().map(|monsters| {
                    let monsters = monsters.to_owned().map(|monster| { self.monster(monster.uid) } ); 
                    MonsterTeam::new(&self.active_monsters.ally_ref(), monsters, TeamUID::Allies)
                }).into(),
                TeamUID::Opponents => self.monsters.opponent_ref().map(|monsters| {
                    let monsters = monsters.to_owned().map(|monster| { self.monster(monster.uid) } ); 
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

    pub fn is_on_team(&self, monster: MonsterRef, team: TeamUID) -> bool {
        self.team(team).monsters().iter().any(|it| monster.is(it.uid()))
    }

    pub fn are_opponents(&self, monster: MonsterRef, other_monster: MonsterRef) -> bool {
        if monster == other_monster {
            return false;
        }

        self.is_on_team(other_monster, monster.team())
    }

    pub fn are_allies(&self, monster: MonsterRef, other_monster: MonsterRef) -> bool {
        // a Monster is not considered its own Ally
        if monster == other_monster {
            return false;
        }

        self.is_on_team(other_monster, monster.team().other())
    }

    /* #endregion ########################################################################## */
    /* #region Ability ##################################################################### */

    fn ability(&self, owner_uid: MonsterUID) -> AbilityRef {
        AbilityRef::new(
            self.abilities
                .iter()
                .find(|ability| { ability.uid == owner_uid })
                .expect("Expected the MonsterUID given to be pre-checked.")
        )
    }

    
    pub(crate) fn ability_event_handler_deck(&self, ability_uid: AbilityUID) -> OwnedEventHandlerDeck {
        
        let ability_owner = self.monster(ability_uid);
        let ability = self.ability(ability_uid);

        let activation_order = ActivationOrder {
            priority: 0,
            speed: ability_owner.stat(Stat::Speed),
            order: ability.species.get().order,
        };
        OwnedEventHandlerDeck {
            event_handler_deck: ability.event_handler_deck(),
            owner: ability_owner,
            activation_order,
            filtering_options: EventFilteringOptions::default(),
        }
    }

    /* #endregion ########################################################################## */
    /* #region Moves # ##################################################################### */

    fn move_(&self, move_uid: MoveUID) -> MoveRef {
        let move_ = self.moves
            .iter()
            .find(|move_| { move_.uid == move_uid })
            .expect("Expected the MoveUID to be pre-checked.");
        MoveRef::new(&move_)
    }
    
    pub fn moveset(&self, owner_uid: MonsterUID) -> MaxSizedVec<MoveRef, 4> {
        let moves = self.moves
            .iter()
            .filter_map(|move_| { 
                let move_uid = move_.uid; 
                if move_uid.owner_uid == owner_uid {
                    Some(MoveRef::new(move_))
                } else {
                    None
                }
            } )
            .collect::<Vec<_>>();
        
        MaxSizedVec::from_vec(moves)    
    }

    pub(crate) fn move_event_handler_deck(&self, move_uid: MoveUID) -> OwnedEventHandlerDeck {
        let move_ = self.move_(move_uid);
        let move_owner = self.monster(move_uid.owner_uid);
        OwnedEventHandlerDeck {
            event_handler_deck: move_.event_handler_deck(),
            owner: move_owner,
            activation_order: ActivationOrder {
                priority: move_.priority.get(),
                speed: move_owner.stat(Stat::Speed),
                order: 0,
            },
            filtering_options: EventFilteringOptions::default(),
        }
    }

    /* #endregion ########################################################################## */
    /* #region Monster ##################################################################### */

    pub fn monsters(&self) -> impl Iterator<Item = MonsterRef> {
        let ally_monsters = self.monsters.ally_ref();
        let opponent_monsters = self.monsters.opponent_ref();
        ally_monsters.iter().chain(opponent_monsters.iter())
            .map(|monster| {
                MonsterRef::new(monster, self.moveset(monster.uid), self.ability(monster.uid))
            })
    }

    pub fn monster(&self, monster_uid: MonsterUID) -> MonsterRef {
        self.monsters()
            .find(|monster| { monster.is(monster_uid) })
            .expect("Expected the MonsterUID given to be pre-checked.")
    }

    pub fn active_monsters(&self) -> PerTeam<MonsterRef> {
        self.active_monsters.map(|monster_uid| {
            self.monster(monster_uid.get())
        })
    }

    pub fn is_active_monster(&self, monster: MonsterRef) -> bool {
        monster.is(self.active_monsters[monster.team()].get())
    }

    fn monster_event_handler_deck(&self, monster_uid: MonsterUID) -> OwnedEventHandlerDeck {
        let monster = self.monster(monster_uid);
        let activation_order = ActivationOrder {
            priority: 0,
            speed: monster.stat(Stat::Speed),
            order: 0,
        };
        OwnedEventHandlerDeck {
            event_handler_deck: monster.event_handler_deck(),
            owner: monster,
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
                    self.monster_event_handler_deck(monster.uid)
                })
                .collect::<Vec<_>>()
        ).unwrap();
        
        let mut move_event_handler_decks = self.moves
            .iter()
            .map(|move_| {
                self.move_event_handler_deck(move_.uid)
            })
            .collect::<Vec<_>>();

        let mut ability_event_handler_decks = self.abilities
            .iter()
            .map(|ability| {
                self.ability_event_handler_deck(ability.uid)
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
            FullySpecifiedChoice::SwitchOut { active_monster: active_monster, benched_monster: _ } => ActivationOrder { 
                priority: 8, 
                speed: active_monster.stat(Stat::Speed), 
                order: 0
            },
            FullySpecifiedChoice::Move { attacker, move_, target: _ } => ActivationOrder {
                priority: move_.priority.get(),
                speed: attacker.stat(Stat::Speed),
                order: 0, //TODO: Think about how to restrict order to be mutually exclusive (Maybe we don't want it to be mutually exclusive).
            },
        }
    }

    pub fn available_choices(&self) -> PerTeam<AvailableChoicesForTeam> {
        let ally_team_available_choices = Ally(self.available_choices_for_team(TeamUID::Allies));
        let opponent_team_available_choices = Opponent(self.available_choices_for_team(TeamUID::Opponents));

        PerTeam::new(ally_team_available_choices, opponent_team_available_choices)
    }

    fn available_choices_for_team(&self, team_uid: TeamUID) -> AvailableChoicesForTeam {
        
        let active_monster_for_team = self.monster(self.active_monsters[team_uid].get());
        let active_monster_for_other_team = self.monster(self.active_monsters[team_uid.other()].get());
        
        let move_choices = active_monster_for_team.moveset()
            .iter()
            .map(|move_| {
                PartiallySpecifiedChoice::Move {
                    attacker: active_monster_for_team.clone(), 
                    move_: *move_,
                    target: active_monster_for_other_team.clone(),
                    display_text: move_.name()
                }
            })
            .collect::<Vec<_>>();
        
        let swicthable_benched_monsters = self.switchable_benched_monsters(team_uid);
        let are_there_any_valid_switchees = swicthable_benched_monsters.is_empty();
        let switch_choice = are_there_any_valid_switchees.then_some(PartiallySpecifiedChoice::SwitchOut { 
            active_monster: active_monster_for_team.clone(), 
            switchable_benched_monsters: swicthable_benched_monsters,
            display_text: "Switch Out",
        });

        AvailableChoicesForTeam::new(
            move_choices, 
            switch_choice,
        )
    }

    pub(crate) fn switchable_benched_monsters(&self, team_uid: TeamUID) -> Vec<MonsterRef> {
        let candidate_switchees: Vec<MonsterRef> = self.team(team_uid)
            .monsters()
            .into_iter()
            .filter_map(|monster| {
                let is_active_monster_for_team = self.is_active_monster(monster);
                let is_valid_switch_partner = not!(monster.is_fainted.get() || is_active_monster_for_team);
                if is_valid_switch_partner {
                    Some(monster)
                } else {
                    None
                }
            }).collect();
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

        let primary_type = monster.primary_type.get();
        let secondary_type = monster.secondary_type.get();
        let type_string = if let Some(secondary_type) = secondary_type {
            format!["   type: {:?}/{:?}\n", primary_type, secondary_type]
        } else {
            format!["   type: {:?}\n", primary_type]
        };
        output_string.push_str(&type_string);

        output_string.push_str(&(prefix_str.to_owned() + "├── "));
        output_string.push_str(format!["ability: {}\n", monster.ability().species.get().name].as_str());

        let number_of_moves = monster.moveset().len();
        for (j, move_) in monster.moveset().iter().enumerate() {
            let is_not_last_move = j < number_of_moves - 1;
            if is_not_last_move {
                output_string.push_str(&(prefix_str.to_owned() + "├── "));
            } else {
                output_string.push_str(&(prefix_str.to_owned() + "└── "));
            }
            output_string.push_str(format!["   move: {}\n", move_.name()].as_str());
        }
        output_string.push_str(&(prefix_str.to_owned() + "\n"));
    }
}