pub(super) mod builder;
mod message_log;

use crate::{
    sim::{Ability, ActivationOrder, AvailableChoices, Monster, MonsterID, MonsterTeam, Move, MoveID, Stat},
    AbilityID, Environment, Item, ItemID, PartiallySpecifiedActionChoice,
};
use monsim_utils::{not, Ally, MaxSizedVec, Opponent};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut, RangeInclusive},
};

use self::builder::BattleFormat;

use super::{
    prng::Prng,
    targetting::{BoardPosition, FieldPosition},
    PerTeam, TeamID,
};
use message_log::MessageLog;

/// The main data struct that contains all the information one could want to know about the current battle. This is meant to be passed around as a unit and queried for battle-related information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battle {
    pub(crate) prng: Prng,
    pub(crate) turn_number: u16,
    pub(crate) format: BattleFormat,
    // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
    pub message_log: MessageLog,
    pub state: BattleState,
}

impl Deref for Battle {
    type Target = BattleState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Battle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleState {
    teams: PerTeam<MonsterTeam>,
    environment: Environment,
}

impl Display for Battle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        attach_tree_for_team(&mut out, *self.ally_team());
        attach_tree_for_team(&mut out, *self.opponent_team());
        write!(f, "{}", out)
    }
}

fn attach_tree_for_team(output_string: &mut String, team: &MonsterTeam) {
    output_string.push_str(&(format!["{}", team.id] + "\n"));
    let number_of_monsters = team.monsters().count();
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

impl Battle {
    // Battle ---------------------------------------------------- //

    pub(crate) fn new(ally_team: Ally<MonsterTeam>, opponent_team: Opponent<MonsterTeam>, environment: Environment, format: BattleFormat, prng: Prng) -> Self {
        let teams = PerTeam::new(ally_team, opponent_team);
        Self {
            prng,
            turn_number: 0,
            message_log: MessageLog::new(),
            format,
            state: BattleState { teams, environment },
        }
    }

    #[inline(always)]
    pub fn format(&self) -> BattleFormat {
        self.format
    }

    pub fn is_finished(&self) -> bool {
        self.ally_team().monsters().all(|monster| monster.is_fainted()) || self.opponent_team().monsters().all(|monster| monster.is_fainted())
    }

    pub fn split(&mut self) -> (&mut Prng, &mut BattleState) {
        (&mut self.prng, &mut self.state)
    }

    // Message Log ------------------------------------------------ //

    /// The Battle queues a message in the message log to be displayed on the
    /// next request to `show_new_messages()`.
    pub fn queue_message(&mut self, message: impl ToString) {
        self.message_log.push(message)
    }

    pub fn queue_debug_message(&mut self, _message: impl ToString) {
        #[cfg(feature = "debug")]
        self.queue_message(_message);
    }

    pub(crate) fn queue_multiple_messages(&mut self, messages: &[impl ToString]) {
        self.message_log.extend(messages);
    }

    /// The Battle shows all the queued messages and then archives the messages.
    pub fn show_new_messages(&mut self) {
        self.message_log.show_new_messages()
    }

    // PRNG ------------------------------------------------------- //

    /// Returns `true` `num` out of `denom` times, pseudorandomly, using the method used in
    /// the games.
    ///
    /// To do this, the PRNG of the Battle is requested to yield the next random number. This
    /// number is used to normalised to a range of `1` to `denom`. This function then returns
    /// `true` if the resulting normalised random number is less than or equal ot `num`,
    /// otherwise it returns `false`.
    pub fn roll_chance(&mut self, num: u16, denom: u16) -> bool {
        self.prng.roll_chance(num, denom)
    }

    /// Returns a random number within the range given by `range`, with equal probability,
    /// of each number in the range.
    ///
    /// To do this, the PRNG of the Battle is requested to yield the next random number. This
    /// number is used to normalised between the start and end of the range. Note that the range
    /// is **inclusive** of both end points.
    pub fn roll_random_number_in_range(&mut self, range: RangeInclusive<u16>) -> u16 {
        self.prng.roll_random_number_in_range(range)
    }
}

impl BattleState {
    // Teams ------------------------------------------------------ //

    #[inline(always)]
    pub fn teams(&self) -> &PerTeam<MonsterTeam> {
        &self.teams
    }

    pub fn team(&self, team_id: TeamID) -> &MonsterTeam {
        &self.teams[team_id]
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
            false
        } else {
            monster_1_id.team_id == monster_2_id.team_id
        }
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub(crate) fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    // Monsters -------------------------------------------------- //

    /// The iterator yields ally monsters first, then yields opponent monsters, in id order.
    pub fn monsters(&self) -> impl Iterator<Item = &Monster> {
        let (ally_team, opponent_team) = self.teams.unwrap_ref();
        ally_team.monsters().chain(opponent_team.monsters())
    }

    /// The iterator yields ally monsters first, then yields opponent monsters, in id order.
    pub(crate) fn monsters_mut(&mut self) -> impl Iterator<Item = &mut Monster> {
        let (ally_team, opponent_team) = self.teams.unwrap_mut();
        ally_team.monsters_mut().chain(opponent_team.monsters_mut())
    }

    pub fn monster(&self, monster_id: MonsterID) -> &Monster {
        let team = self.team(monster_id.team_id);
        &team[monster_id.monster_number]
    }

    /**
    _The use of this method is discouraged_, if used wrong this could leave the monster in an invalid state.
    However sometimes niche use cases will require direct mutable access to a monster's data. Without this,
    we would need to be able to predict and provide a helper function for any niche manipulation of a
    Monster's data, which borders on impossible.
    */
    pub fn monster_mut(&mut self, monster_id: MonsterID) -> &mut Monster {
        let team = self.team_mut(monster_id.team_id);
        &mut team[monster_id.monster_number]
    }

    pub fn active_monsters_by_team(&self) -> PerTeam<Vec<&Monster>> {
        let ally_team_active_monsters = self.ally_team().map_consume(|team| team.active_monsters());
        let opponent_team_active_monsters = self.opponent_team().map_consume(|team| team.active_monsters());
        PerTeam::new(ally_team_active_monsters, opponent_team_active_monsters)
    }

    pub fn active_monster_ids(&self) -> impl Iterator<Item = MonsterID> {
        self.monsters()
            .filter_map(|monster| {
                if matches!(monster.board_position, BoardPosition::Field(_)) {
                    Some(monster.id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    // Abilities ---------------------------------------------------- //

    pub fn ability(&self, ability_id: AbilityID) -> &Ability {
        &self.monster(ability_id.owner_id).ability
    }

    /**
    _The use of this method is discouraged_, if used wrong this could leave the monster in an invalid state.
    However sometimes niche use cases will require direct mutable access to a monster's data. Without this,
    we would need to be able to predict and provide a helper function for any niche manipulation of a
    Monster's data, which borders on impossible.
    */
    pub fn ability_mut(&mut self, owner_id: MonsterID) -> &mut Ability {
        &mut self.monster_mut(owner_id).ability
    }

    // Moves -------------------------------------------------------- //

    pub fn move_(&self, move_id: MoveID) -> &Move {
        &self.monster(move_id.owner_id).moveset[move_id.move_number as usize]
    }

    /**
    _The use of this method is discouraged_, if used wrong this could leave the monster in an invalid state.
    However sometimes niche use cases will require direct mutable access to a monster's data. Without this,
    we would need to be able to predict and provide a helper function for any niche manipulation of a
    Monster's data, which borders on impossible.
    */
    pub fn move_mut(&mut self, move_id: MoveID) -> &mut Move {
        &mut self.monster_mut(move_id.owner_id).moveset[move_id.move_number as usize]
    }

    // Items --------------------------------------------------------- //

    pub fn item(&self, item_id: ItemID) -> Option<&Item> {
        self.monster(item_id.item_holder_id).held_item.as_ref()
    }

    /**
    _The use of this method is discouraged_, if used wrong this could leave the monster in an invalid state.
    However sometimes niche use cases will require direct mutable access to a monster's data. Without this,
    we would need to be able to predict and provide a helper function for any niche manipulation of a
    Monster's data, which borders on impossible.
    */
    pub fn item_mut(&mut self, item_id: ItemID) -> Option<&mut Item> {
        self.monster_mut(item_id.item_holder_id).held_item.as_mut()
    }

    // Choices -------------------------------------------------------- //

    const MAX_SWITCHES_PER_TURN: usize = 3;

    pub(crate) fn available_choices_for_monster(
        &self,
        monster: &Monster,
        monsters_already_selected_for_switch: &MaxSizedVec<MonsterID, { Self::MAX_SWITCHES_PER_TURN }>,
    ) -> AvailableChoices {
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
                };
                move_actions.push(partially_specified_choice);
            }
        }

        // Switch choice
        let switchable_benched_monster_ids = self.switchable_benched_monster_ids(monster.id.team_id, monsters_already_selected_for_switch);
        let any_switchable_monsters = not!(switchable_benched_monster_ids.is_empty());
        let switch_action = if any_switchable_monsters {
            Some(PartiallySpecifiedActionChoice::SwitchOut {
                active_monster_id: monster.id,
                switchable_benched_monster_ids,
                activation_order: ActivationOrder {
                    priority: 8,
                    speed: monster.stat(Stat::Speed),
                    order: 0,
                },
            })
        } else {
            None
        };

        AvailableChoices::new(move_actions, switch_action)
    }

    /// Returns an array of options where all the `Some` variants are at the beginning.
    pub(crate) fn switchable_benched_monster_ids(
        &self,
        team_id: TeamID,
        monsters_already_selected_for_switch: &MaxSizedVec<MonsterID, { Self::MAX_SWITCHES_PER_TURN }>,
    ) -> MaxSizedVec<MonsterID, 5> {
        let mut number_of_switchees = 0;
        let mut switchable_benched_monsters = Vec::with_capacity(5);
        for monster in self.team(team_id).monsters() {
            let is_active_monster_for_team = matches!(monster.board_position, BoardPosition::Field(_));
            let is_already_selected_for_switch = monsters_already_selected_for_switch.contains(&monster.id);
            let is_valid_switch_partner = not!(monster.is_fainted()) && not!(is_active_monster_for_team) && not!(is_already_selected_for_switch);
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

    // Positions & Targetting ------------------------------------------------ //

    pub(crate) fn monster_at_position(&self, field_position: FieldPosition) -> Option<&Monster> {
        self.monsters().find(|monster| {
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
                let targetter_position = self
                    .monster(move_.id.owner_id)
                    .board_position
                    .field_position()
                    .expect("The targetter must be on the field.");
                let targetted_position = active_monster
                    .board_position
                    .field_position()
                    .expect("The targetted position must be on the field.");
                if FieldPosition::is_position_relation_allowed_by_flags(targetter_position, targetted_position, move_.allowed_target_position_relation_flags())
                {
                    possible_targets_for_move.push(targetted_position);
                }
            }
        }

        MaxSizedVec::from_vec(possible_targets_for_move)
    }
}
