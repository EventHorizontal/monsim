mod message_log;
pub(super) mod builders;

use std::fmt::Display;
use monsim_utils::{not, Ally, MaxSizedVec, Opponent};
use crate::{sim::{Ability, ActivationOrder, AvailableChoices, Monster, MonsterID, MonsterTeam, Move, MoveID, Stat}, AbilityID, Event, OwnedEventHandler, PartiallySpecifiedActionChoice, TargetFlags};

use self::builders::BattleFormat;

use super::{prng::Prng, targetting::{BoardPosition, FieldPosition}, PerTeam, TeamID};
use message_log::MessageLog;

/// The main data struct that contains all the information one could want to know about the current battle. This is meant to be passed around as a unit and queried for battle-related information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleState {

    pub(crate) prng: Prng,
    pub(crate) turn_number: u16,
    pub(crate) format: BattleFormat,
    // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
    pub message_log: MessageLog,
    
    teams: PerTeam<MonsterTeam>,
}

impl BattleState {

    pub(crate) fn new(ally_team: Ally<MonsterTeam>, opponent_team: Opponent<MonsterTeam>, format: BattleFormat) -> Self {
        let teams = PerTeam::new(ally_team, opponent_team);
        Self {
            prng: Prng::from_current_time(),
            turn_number: 0,
            teams,
            message_log: MessageLog::new(),
            format,
        }
    }

    #[inline(always)]
    pub fn format(&self) -> BattleFormat {
        self.format
    }
    

    pub fn is_finished(&self) -> bool {
        self.ally_team().monsters().all(|monster| {
            monster.is_fainted()
        })
        ||
        self.opponent_team().monsters().all(|monster| {
            monster.is_fainted()
        })
    }

    // Teams -----------------

    pub fn team(&self, team_id: TeamID) -> &MonsterTeam {
        & self.teams[team_id]
    }

    pub(crate) fn team_mut(&mut self, team_id: TeamID) -> &mut MonsterTeam {
        &mut self.teams[team_id]
    }

    pub fn ally_team(&self) -> Ally<&MonsterTeam> {
        self.teams.ally_ref()
    }

    pub(crate) fn _ally_team_mut(&mut self) -> Ally<&mut MonsterTeam> {
        self.teams.ally_mut()
    }

    pub fn opponent_team(&self) -> Opponent<&MonsterTeam> {
        self.teams.opponent_ref()
    }

    pub(crate) fn _opponent_team_mut(&mut self) -> Opponent<&mut MonsterTeam> {
        self.teams.opponent_mut()
    }

    pub fn is_on_ally_team(&self, monster_id: MonsterID) -> bool {
        monster_id.team_id == TeamID::Allies
    }

    pub fn is_on_opponent_team(&self, monster_id: MonsterID) -> bool {
        monster_id.team_id == TeamID::Opponents
    }

    pub fn are_opponents(&self, monster_1_id: MonsterID, monster_2_id: MonsterID) -> bool {
        monster_1_id.team_id != monster_2_id.team_id
    }

    /// A monster is not considered its own ally.
    pub fn are_allies(&self, monster_1_id: MonsterID, monster_2_id: MonsterID) -> bool {
        if monster_1_id == monster_2_id {
            return false;
        } else {
            monster_1_id.team_id == monster_2_id.team_id
        }
    }

    pub fn event_handlers_for<E: Event>(&self, event: E) -> Vec<OwnedEventHandler<E>> {
        let mut out = Vec::new();
        out.append(&mut self.ally_team().event_handlers_for(event));
        out.append(&mut self.opponent_team().event_handlers_for(event));
        out
    }

    // Monsters -----------------

    pub fn monsters(&self) -> impl Iterator<Item = &Monster> {
        let (ally_team, opponent_team) = self.teams.unwrap_ref();
        ally_team.monsters().chain(opponent_team.monsters())
    }

    pub(crate) fn _monsters_mut(&mut self) -> impl Iterator<Item = &mut Monster> {
        let (ally_team, opponent_team) = self.teams.unwrap_mut();
        ally_team.monsters_mut().chain(opponent_team.monsters_mut())
    }

    pub fn monster(&self, monster_id: MonsterID) -> &Monster {
        let team = self.team(monster_id.team_id);
        &team[monster_id.monster_number]
    }

    pub(crate) fn monster_mut(&mut self, monster_id: MonsterID) -> &mut Monster {
        let team = self.team_mut(monster_id.team_id);
        &mut team[monster_id.monster_number]
    }

    pub fn active_monsters_by_team(&self) -> PerTeam<Vec<&Monster>> {
        let ally_team_active_monsters = self.ally_team().map_consume(|team| { team.active_monsters() });
        let opponent_team_active_monsters = self.opponent_team().map_consume(|team| team.active_monsters() );
        PerTeam::new(ally_team_active_monsters, opponent_team_active_monsters)
    }

    pub fn active_monsters(&self) -> impl Iterator<Item = &Monster> {
        self.monsters()
            .filter(|monster| {
                matches!(monster.board_position, BoardPosition::Field(_))
            })
    }

    // Abilities -----------------

    pub fn ability(&self, ability_id: AbilityID) -> &Ability {
        &self.monster(ability_id.owner_id)
            .ability
    }

    pub(crate) fn _ability_mut(&mut self, owner_id: MonsterID) -> &mut Ability {
        &mut self
            .monster_mut(owner_id)
            .ability
    }

    // Moves -----------------

    pub fn move_(&self, move_id: MoveID) -> &Move {
        &self.monster(move_id.owner_id)
            .moveset[move_id.move_number as usize]
    }

    pub(crate) fn move_mut(&mut self, move_id: MoveID) -> &mut Move {
        &mut self.monster_mut(move_id.owner_id)
            .moveset[move_id.move_number as usize]
    }

    // Choice -------------------------------------

    // pub fn available_choices(&self) -> PerTeam<AvailableChoicesForTeam> {
    //     self.teams.map_clone(|team| {
    //         self.available_choices_for(team.id)
    //     })
    // }

    pub(crate) fn available_choices_for(&self, monster: &Monster) -> AvailableChoices {
        
        // Move choices
        let mut move_actions = Vec::with_capacity(4);
        for move_ in monster.moveset().iter() {
            /*
            The move is only choosable if it still has power points. FEATURE: We might want to emit 
            "inactive" choices in order to show a greyed out version of the choice (in this case that 
            the monster has that move but its out of PP).
            */
            if move_.current_power_points > 0 {
                let partially_specified_choice = PartiallySpecifiedActionChoice::Move { 
                    move_id: move_.id,
                    possible_target_positions: self.possible_targets_for_move(move_),
                    activation_order: ActivationOrder {
                        priority: move_.priority(),
                        speed: monster.stat(Stat::Speed),
                        order: 0, //TODO: Think about how to restrict order to be mutually exclusive
                    },
                    display_text: move_.name()  
                };
                move_actions.push(partially_specified_choice);
            }
        }

        // Switch choice
        let switchable_benched_monster_ids = self.switchable_benched_monster_ids(monster.id.team_id);
        let any_switchable_monsters = not!(switchable_benched_monster_ids.is_empty());
        let switch_action = if any_switchable_monsters {
            Some(PartiallySpecifiedActionChoice::SwitchOut { 
                active_monster_id: monster.id, 
                switchable_benched_monster_ids,
                activation_order: ActivationOrder { 
                    priority: 8, 
                    speed: monster.stat(Stat::Speed), 
                    order: 0
                },
                display_text: "Switch Out",
            })
        } else {
            None
        };

        AvailableChoices::new(
            move_actions, 
            switch_action,
        )
    }

    // TODO: Once we have multitargeting/multiple active monsters, if one monster has selected
    // to switch out with a particular benched monster, that benched monster will need to be excluded.
    // Perhaps the ui will take care of that though?
    /// Returns an array of options where all the `Some` variants are at the beginning.
    pub(crate) fn switchable_benched_monster_ids(&self, team_id: TeamID) -> MaxSizedVec<MonsterID, 5> {
        let mut number_of_switchees = 0;
        let mut switchable_benched_monsters = Vec::with_capacity(5);
        for monster in self.team(team_id).monsters() {
            let is_active_monster_for_team = matches!(monster.board_position, BoardPosition::Field(_));
            let is_valid_switch_partner = not!(monster.is_fainted()) && not!(is_active_monster_for_team);
            if is_valid_switch_partner {
                switchable_benched_monsters.push(monster.id);
                number_of_switchees += 1;
                assert!(number_of_switchees < 6);
            }
        }
        if switchable_benched_monsters.is_empty() {
            MaxSizedVec::empty()
        } else {
            MaxSizedVec::from_vec(switchable_benched_monsters)
        }
    }
    
    pub(crate) fn monster_at_position(&self, field_position: FieldPosition) -> Option<&Monster> {
        self.monsters()
            .find(|monster| {
                if let Some(monster_field_position) = monster.field_position() {
                    monster_field_position == field_position
                } else {
                    false
                }
            })
    }
    
    fn possible_targets_for_move(&self, move_: &Move) -> MaxSizedVec<FieldPosition, 6> {
        let mut possible_targets_for_move = Vec::new();

        for active_monsters_per_team in self.active_monsters_by_team() {
            for active_monster in active_monsters_per_team {
                let targetter_position = self.monster(
                    move_.id.owner_id).board_position.field_position().expect("The targetter must be on the field.");
                let targetted_position = active_monster.board_position.field_position().expect("The targetted position must be on the field.");
                if self.is_valid_target_position(
                    targetter_position, 
                    move_.allowed_target_flags(), 
                    targetted_position
                ) {
                    possible_targets_for_move.push(targetted_position);
                }
            }
        }

        MaxSizedVec::from_vec(possible_targets_for_move)
    }

    pub(crate) fn is_valid_target_position(&self, targetter_position: FieldPosition, allowed_target_flags: TargetFlags, targetted_position: FieldPosition) -> bool {
        let mut targetted_position_flags = TargetFlags::empty();
        // FEATURE: BENCHED adjacency flag?
        if targetter_position == targetted_position {
            targetted_position_flags |= TargetFlags::SELF
        } else if targetter_position.is_on_the_opposite_side_of(targetted_position) {
            targetted_position_flags |= TargetFlags::OPPONENTS
        } else {
            targetted_position_flags |= TargetFlags::ALLIES
        } 
        // Only calculate adjacency if not self
        if not!(targetted_position_flags == TargetFlags::SELF) {
            if targetter_position.is_adjacent_to(targetted_position) {
                targetted_position_flags |= TargetFlags::ADJACENT
            } else  {
                targetted_position_flags |= TargetFlags::NONADJACENT
            }
        }
        allowed_target_flags.contains(targetted_position_flags)
    }
}

impl Display for BattleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        push_pretty_tree_for_team(
            &mut out,
            "Ally Team\n", 
            *self.ally_team(), 
            self.ally_team().monsters().count(),
        );
        push_pretty_tree_for_team(
            &mut out,
            "Opponent Team\n",
            *self.opponent_team(),
            self.opponent_team().monsters().count(),
        );
        write!(f, "{}", out)
    }
}

fn push_pretty_tree_for_team(output_string: &mut String, team_name: &str, team: &MonsterTeam, number_of_monsters: usize) {
    output_string.push_str(team_name);
    for (i, monster) in team.monsters().enumerate() {
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

        let primary_type = monster.species.primary_type();
        let secondary_type = monster.species.secondary_type();
        let type_string = if let Some(secondary_type) = secondary_type {
            format!["   type: {:?}/{:?}\n", primary_type, secondary_type]
        } else {
            format!["   type: {:?}\n", primary_type]
        };
        output_string.push_str(&type_string);

        output_string.push_str(&(prefix_str.to_owned() + "├── "));
        output_string.push_str(format!["ability: {}\n", monster.ability.name()].as_str());

        let number_of_moves = monster.moveset.count();
        for (j, move_) in monster.moveset.into_iter().enumerate() {
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