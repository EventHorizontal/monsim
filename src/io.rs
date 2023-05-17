use super::game_mechanics::{BattlerUID, MoveUID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    Move {
        move_uid: MoveUID,
        target_uid: BattlerUID,
    },
}

impl ActionChoice {
    pub(crate) fn chooser(&self) -> BattlerUID {
        match self {
            ActionChoice::Move {
                move_uid,
                target_uid: _,
            } => move_uid.battler_uid,
        }
    }

    pub(crate) fn target(&self) -> BattlerUID {
        match self {
            ActionChoice::Move {
                move_uid: _,
                target_uid,
            } => *target_uid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AvailableActions {
    pub ally_team_choices: TeamAvailableActions,
    pub opponent_team_choices: TeamAvailableActions,
}

pub type TeamAvailableActions = Vec<ActionChoice>;
pub type ChosenActions = Vec<ActionChoice>;
