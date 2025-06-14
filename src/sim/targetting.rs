use std::fmt::Display;

use crate::TeamID;

/**
I am thinking of the Battlefield as divided into two "zones", like a tabletop 
card game, one is the **Bench** zone where the Monsters not participating in 
the battle are, and the **Field** zone where the Monsters currently participating
in the battle are. Monsters in the Field zone have a value indicating which tile
they are standing on.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardPosition {
    Bench,
    Field(FieldPosition),
}

impl Display for BoardPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardPosition::Bench => write!(f, "Benched"),
            BoardPosition::Field(field_position) => write!(f, "{}", field_position),
        }
    }
}

impl BoardPosition {
    pub fn field_position(&self) -> Option<FieldPosition> {
        match self {
            BoardPosition::Bench => None,
            BoardPosition::Field(field_position) => Some(*field_position),
        }
    }  
}

/**
The way I thought about the positions, the "left" and "right" are with respect to the
the ally team, so `AllyLeft` and `OpponentLeft` are facing each other. I know _technically_
it "should" be that `OpponentLeft` faces `AllyRight` but I think it gets difficult
to wrap your head around that every time.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPosition {
    AllySideLeft,
    AllySideCentre,
    AllySideRight,
    /// Opponent's left Monster as seen from the POV of the "player", i.e. the AllySide trainer
    OpponentSideLeft,
    OpponentSideCentre,
    /// Opponent's right Monster as seen from the POV of the "player", i.e. the AllySide trainer
    OpponentSideRight,
}

impl Display for FieldPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FieldPosition {
    /// Returns a vector of the positions adjacent to the position this method is called on, 
    /// including allies and opponents, but not including self.
    pub fn adjacent_positions(&self) -> Vec<FieldPosition> {
        let (x, y) = self.to_coords();
        let mut positions = Vec::with_capacity(8);
        let compass_directions = [(-1,1), (-1, 0), (-1, -1), (0, 1), (0, -1), (1, 1), (1, 0), (1, -1)];
        for (dx, dy) in compass_directions {
            let (x, y) = (x + dx, y + dy);
            let maybe_position = FieldPosition::from_coords((x, y));
            if let Some(position) = maybe_position {
                positions.push(position);
            } 
        }
        positions
    }

    pub fn is_adjacent_to(&self, position_to_compare: FieldPosition) -> bool {
        self.adjacent_positions().contains(&position_to_compare)
    } 

    fn from_coords(value: (i8, i8)) -> Option<FieldPosition> {
        match value {
            (0, 0) => Some(FieldPosition::AllySideLeft),
            (1, 0) => Some(FieldPosition::AllySideCentre),
            (2, 0) => Some(FieldPosition::AllySideRight),
            (0, 1) => Some(FieldPosition::OpponentSideLeft),
            (1, 1) => Some(FieldPosition::OpponentSideCentre),
            (2, 1) => Some(FieldPosition::OpponentSideRight),
            _ => None,
        }
    }

    fn to_coords(&self) -> (i8, i8) {
        match self {
            FieldPosition::AllySideLeft => (0, 0),
            FieldPosition::AllySideCentre => (1, 0),
            FieldPosition::AllySideRight => (2, 0),
            FieldPosition::OpponentSideLeft => (0, 1),
            FieldPosition::OpponentSideCentre => (1, 1),
            FieldPosition::OpponentSideRight => (2, 1),
        }
    }
    
    pub(crate) fn is_on_the_opposite_side_of(&self, other_position: FieldPosition) -> bool {
        let self_side = self.to_coords().1; // The second element tells us which side the position is on.
        let other_position_side = other_position.to_coords().1;
        self_side != other_position_side
    }
    
    pub(crate) fn is_on_the_same_side_as(&self, other_position: FieldPosition) -> bool {
        let self_side = self.to_coords().1; // The second element tells us which side the position is on.
        let other_position_side = other_position.to_coords().1;
        self_side == other_position_side
    }
    
    pub(crate) fn side(&self) -> TeamID {
        match self {
            FieldPosition::AllySideLeft | FieldPosition::AllySideCentre | FieldPosition::AllySideRight => TeamID::Allies,
            FieldPosition::OpponentSideLeft | FieldPosition::OpponentSideCentre | FieldPosition::OpponentSideRight => TeamID::Opponents,
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TargetFlags: u8 {
        const _           = 0b1111_1111;
        
        const ANY         = 0b0000_0000;
        const ALL         = 0b0000_0001;
        
        const ADJACENT    = 0b0000_0010;
        const NONADJACENT = 0b0000_0100;
        
        const SELF        = 0b0000_1000;
        const ALLIES      = 0b0001_0000;
        const OPPONENTS   = 0b0010_0000;
    }
}