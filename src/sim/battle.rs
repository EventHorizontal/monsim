use utils::not;

use crate::sim::{
        event::CompositeEventResponderInstanceList, Ability, ChosenAction, ActivationOrder, AllyBattlerTeam, AvailableActions, Battler, BattlerNumber,
        BattlerTeam, BattlerUID, Monster, Move, MoveUID, OpponentBattlerTeam, Stat, TeamAvailableActions,
        utils,
};

use std::{
    collections::HashMap,
    fmt::Display,
    iter::Chain,
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use super::{
    prng::{self, Prng},
    TeamID, ALLY_1, ALLY_2, ALLY_3, ALLY_4, ALLY_5, ALLY_6, OPPONENT_1, OPPONENT_2, OPPONENT_3, OPPONENT_4, OPPONENT_5, OPPONENT_6, ChoosableAction,
};

type BattlerIterator<'a> = Chain<Iter<'a, Battler>, Iter<'a, Battler>>;
type MutableBattlerIterator<'a> = Chain<IterMut<'a, Battler>, IterMut<'a, Battler>>;

pub type MessageBuffer = Vec<String>;
pub const CONTEXT_MESSAGE_BUFFER_SIZE: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battle {
    pub current_action: Option<ChosenAction>,
    pub prng: Prng,
    pub ally_team: BattlerTeam,
    pub opponent_team: BattlerTeam,
    // TODO: Special text format for storing metadata with text (colour and modifiers like italic and bold).
    pub message_buffer: MessageBuffer,
    pub active_battlers: BattlerMap<bool>,
    pub fainted_battlers: BattlerMap<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlerMap<T> {
    map: HashMap<BattlerUID, T>,
}

impl<T> BattlerMap<T> {
    pub fn new(map: HashMap<BattlerUID, T>) -> Self {
        for team_id in [TeamID::Allies, TeamID::Opponents].into_iter() {
            for number in 0..=5 {
                let battler_number = BattlerNumber::from(number);
                let battler_uid = BattlerUID { team_id, battler_number };
                assert!(
                    map.contains_key(&battler_uid),
                    "Could not find {battler_uid} in hash_map for BattlerMap construction"
                )
            }
        }
        Self { map }
    }
}

impl<T> Index<BattlerUID> for BattlerMap<T> {
    type Output = T;

    fn index(&self, index: BattlerUID) -> &Self::Output {
        self.map.get(&index).expect("All BattlerUIDs should have a bool value")
    }
}


impl<T> IndexMut<BattlerUID> for BattlerMap<T> {
    fn index_mut(&mut self, index: BattlerUID) -> &mut Self::Output {
        self.map.get_mut(&index).expect("All BattlerUIDs should have a bool value")
    }
}

impl Battle {
    pub fn new(ally_team: AllyBattlerTeam, opponent_team: OpponentBattlerTeam) -> Self {
        Self {
            current_action: None,
            prng: Prng::new(prng::seed_from_time_now()),
            ally_team: ally_team.0,
            opponent_team: opponent_team.0,
            message_buffer: Vec::with_capacity(CONTEXT_MESSAGE_BUFFER_SIZE),
            active_battlers: BattlerMap::new(utils::collection!(
                ALLY_1     => true,
                ALLY_2     => false,
                ALLY_3     => false,
                ALLY_4     => false,
                ALLY_5     => false,
                ALLY_6     => false,
                OPPONENT_1 => true,
                OPPONENT_2 => false,
                OPPONENT_3 => false,
                OPPONENT_4 => false,
                OPPONENT_5 => false,
                OPPONENT_6 => false,
            )),
            fainted_battlers: BattlerMap::new(utils::collection!(
                ALLY_1     => false,
                ALLY_2     => false,
                ALLY_3     => false,
                ALLY_4     => false,
                ALLY_5     => false,
                ALLY_6     => false,
                OPPONENT_1 => false,
                OPPONENT_2 => false,
                OPPONENT_3 => false,
                OPPONENT_4 => false,
                OPPONENT_5 => false,
                OPPONENT_6 => false,
            ))
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
        self.battlers()
            .find(|it| it.uid == battler_uid)
            .expect("Error: Requested look up for a monster with ID that does not exist in this battle.")
    }

    pub fn is_battler_on_field(&self, battler_uid: BattlerUID) -> bool {
        self.active_battlers[battler_uid]
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

    pub fn composite_event_responder_instances(&self) -> CompositeEventResponderInstanceList {
        let mut out = Vec::new();
        out.append(&mut self.ally_team.composite_event_responder_instances());
        out.append(&mut self.opponent_team.composite_event_responder_instances());
        out
    }

    pub fn is_on_ally_team(&self, uid: BattlerUID) -> bool {
        self.ally_team.battlers().iter().any(|it| it.uid == uid)
    }

    pub fn is_on_opponent_team(&self, uid: BattlerUID) -> bool {
        self.opponent_team.battlers().iter().any(|it| it.uid == uid)
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

    pub fn active_battlers(&self) -> Vec<&Battler> {
        self.battlers()
            .filter(|it| self.active_battlers[it.uid])
            .collect()
    }

    /// Returns `(FirstBattler, Option<SecondBattler>)` depending on 
    /// if a second battler is on the field.
    pub fn active_battlers_on_team(&self, team_id: TeamID) -> (&Battler, Option<&Battler>) {
        let mut active_battlers = self
            .battlers()
            .filter(|it| self.active_battlers[it.uid] && it.uid.team_id == team_id);
        let first_battler = active_battlers
            .next()
            .expect("`Battle` should ensure that there is always at least one battler on field.");
        let maybe_second_battler = active_battlers.next();
        assert_eq!(active_battlers.next(), None, "There should never be more than 2 battlers on field.");
        (first_battler, maybe_second_battler)
    }

    /// Given an action choice, computes its activation order. This is handled by `Battle` because the order is
    /// context sensitive.
    pub(crate) fn choice_activation_order(&self, choice: ChosenAction) -> ActivationOrder {
        match choice {
            ChosenAction::Move { move_uid, target_uid: _ } => ActivationOrder {
                priority: self.move_(move_uid).species.priority,
                speed: self.monster(move_uid.battler_uid).stats[Stat::Speed],
                order: 0,
            },
            ChosenAction::SwitchOut { switcher_uid: active_battler_uid, switchee_uid: _ } => ActivationOrder { 
                priority: 8, 
                speed: self.monster(active_battler_uid).stats[Stat::Speed], 
                order: 0
            }
        }
    }

    pub fn available_actions(&self) -> AvailableActions {
        let ally_team_available_actions = self.team_available_actions(TeamID::Allies);
        let opponent_team_available_actions = self.team_available_actions(TeamID::Opponents);

        AvailableActions {
            ally_team_available_actions,
            opponent_team_available_actions,
        }
    }

    fn team_available_actions(&self, team_id: TeamID) -> TeamAvailableActions {
        
        let team_active_battler = self.active_battlers_on_team(team_id).0;
        let team = self.team(team_id);
        
        let moves = team_active_battler.move_uids();
        let mut count = 0usize;
        let mut move_actions = Vec::with_capacity(4);
        for move_uid in moves {
            let choosable_action = ChoosableAction::Move(move_uid);
            move_actions.push((count, choosable_action));
            count += 1;
        }

        let any_benched_ally_battlers = team.battlers().len() > 1;
        let switch_action = if any_benched_ally_battlers {
            Some((count, ChoosableAction::SwitchOut { switcher_uid: team_active_battler.uid }))
        } else {
            None
        };

        TeamAvailableActions::new(
            move_actions, 
            switch_action,
        )
    }

    pub fn push_message(&mut self, message: &dyn Display) {
        self.message_buffer.push(format!["{}", message]);
    }

    pub fn push_messages(&mut self, messages: &[&dyn Display]) {
        for message in messages {
            self.message_buffer.push(format!["{}", message]);
        }
    }

    // TODO: Remove Ally and Opponent varients of BattlerTeam and Battler, 
    // or find a way to make them work. Perhaps methods with TeamID parameter like I used here? Or maybe something more clever while retaining the types...
    pub(crate) fn switch_partners_on_team(&self, team_id: TeamID) -> Vec<&Battler> {
        let team = match team_id {
            TeamID::Allies => &self.ally_team,
            TeamID::Opponents => &self.opponent_team,
        };
        
        team.battlers()
            .iter()
            .filter(|battler| {
                let is_valid_switch_partner = not!(self.is_battler_fainted(battler.uid)) && not!(self.active_battlers[battler.uid]);
                #[allow(clippy::let_and_return)]
                is_valid_switch_partner
            })
            .collect()
    }

    pub fn is_battler_fainted(&self, battler_uid: BattlerUID) -> bool {
        self.fainted_battlers[battler_uid]
    }

    fn team(&self, team_id: TeamID) -> &BattlerTeam {
        match team_id {
            TeamID::Allies => &self.ally_team,
            TeamID::Opponents => &self.opponent_team,
        }
    }

    fn team_mut(&mut self, team_id: TeamID) -> &mut BattlerTeam {
        match team_id {
            TeamID::Allies => &mut self.ally_team,
            TeamID::Opponents => &mut self.opponent_team,
        }
    }
}

impl Display for Battle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        push_pretty_tree_for_team(
            &mut out,
            "Ally Team\n", 
            &self.ally_team, 
            self.ally_team.battlers().iter().count(),
        );
        push_pretty_tree_for_team(
            &mut out,
            "Opponent Team\n",
            &self.opponent_team,
            self.opponent_team.battlers().iter().count(),
        );
        write!(f, "{}", out)
    }
}

fn push_pretty_tree_for_team(output_string: &mut String, team_name: &str, team: &BattlerTeam, number_of_monsters: usize) {
    output_string.push_str(team_name);
    for (i, battler) in team.battlers().iter().enumerate() {
        let is_not_last_monster = i < number_of_monsters - 1;
        let (prefix_str, suffix_str) = if is_not_last_monster {
            ("\t│\t", "├── ")
        } else {
            ("\t \t", "└── ")
        };
        output_string.push_str(&("\t".to_owned() + suffix_str));
        output_string.push_str(&BattlerTeam::battler_status_as_string(battler));
        output_string.push_str(&(prefix_str.to_owned() + "│\n"));
        output_string.push_str(&(prefix_str.to_owned() + "├── "));

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
