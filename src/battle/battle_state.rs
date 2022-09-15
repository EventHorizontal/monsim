use rand::{rngs::ThreadRng, Rng};
use super::entities::{BattlerID, Entity, EventHandlerSet, Monster, Move, MoveID};

#[derive(Clone, Debug)]
pub struct BattleState {
    pub prng: ThreadRng,
    monsters: Vec<Monster>,
    moves: Vec<Move>,
}

impl BattleState {
    pub fn new(monsters: Vec<Monster>, moves: Vec<Move>) -> Self {
        BattleState {
            prng: ThreadRng::default(),
            monsters,
            moves,
        }
    }

    pub fn get_event_handlers<R>(
        &self,
        field: fn(EventHandlerSet) -> Option<fn() -> R>,
    ) -> Vec<fn() -> R> {
        self.entities()
            .iter()
            .filter(|it| { it.active() } )
            .map(|it| it.get_event_handlers().clone())
            .map(field)
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn entities(&self) -> Vec<&dyn Entity> {
        let mut entities = vec![];
        for monster in self.monsters.iter() {
            entities.append(&mut vec![monster as &dyn Entity]);
        }
        for _move in self.moves.iter() {
            entities.append(&mut vec![_move as &dyn Entity]);
        }
        entities
    }

    pub fn monster(&self, battler_id: BattlerID) -> &Monster {
        self.monsters
            .iter()
            .find(|it| it.battler_id == battler_id)
            .expect(
                format!["E0003: Could not find a monster with BattlerID {battler_id}."]
                    .as_str(),
            )
    }

    pub fn monster_mut(&mut self, battler_id: BattlerID) -> &mut Monster {
        self.monsters
            .iter_mut()
            .find(|it| it.battler_id == battler_id)
            .expect(
                format!["E0005: Could not find a monster with BattlerID {battler_id}."]
                    .as_str(),
            )
    }

    pub fn move_(&self, battler_id: BattlerID, move_id: MoveID) -> &Move {
        self.moves
            .iter()
            .find(|it| it.battler_id == battler_id && it.move_id == move_id)
            .expect(format!["E0004: Could not find a move with BattlerID {battler_id} and MoveID {move_id:?}."].as_str())
    }

    pub fn move_mut(&mut self, battler_id: BattlerID, move_id: MoveID) -> &mut Move {
        self.moves
            .iter_mut()
            .find(|it| it.battler_id == battler_id && it.move_id == move_id)
            .expect(format!["E0004: Could not find a move with BattlerID {battler_id} and MoveID {move_id:?}."].as_str())
    }
}