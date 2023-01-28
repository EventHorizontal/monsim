use super::game_mechanics::{BattlerUID, MoveUID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    None,
    Move {
        move_uid: MoveUID,
        target_uid: BattlerUID,
    },
}

impl ActionChoice {
    pub(crate) fn chooser(&self) -> BattlerUID {
        match self {
            ActionChoice::None => unreachable!(),
            ActionChoice::Move {
                move_uid,
                target_uid: _,
            } => move_uid.battler_uid,
        }
    }

    pub(crate) fn target(&self) -> BattlerUID {
        match self {
            ActionChoice::None => unreachable!(),
            ActionChoice::Move {
                move_uid: _,
                target_uid,
            } => *target_uid,
        }
    }
}

// TODO: If/when we support double battles, this needs to take 1-2 choices per team.
pub struct UserInput {
    pub ally_choices: ActionChoice,
    pub opponent_choices: ActionChoice,
}

impl UserInput {
    pub fn choices(&self) -> Vec<ActionChoice> {
        vec![self.ally_choices, self.opponent_choices]
    }
}
