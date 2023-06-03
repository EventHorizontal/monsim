pub mod battle_context;
pub mod game_mechanics;
pub mod global_constants;
pub mod choice;

mod action;
mod event;
mod ordering;
pub mod prng;

pub use action::*;
pub use battle_context::*;
pub use event::{EventHandler, EventHandlerFilters, EventHandlerSet, ActivationOrder, InBattleEvent, event_dex, TargetFlags, DEFAULT_HANDLERS};
pub use game_mechanics::*;
pub use global_constants::*;
pub use choice::*;
use prng::Prng;

pub use battle_context::BattleContext;
pub use battle_context_macro::battle_context;

type TurnOutcome = Result<(), SimError>;

#[derive(Debug)]
pub struct Battle {
    pub ctx: BattleContext,
    pub prng: Prng,
    pub turn_number: u8,
}

impl Battle {
    pub fn new(ctx: BattleContext) -> Self {
        Battle {
            ctx,
            prng: Prng::new(prng::seed_from_time_now()),
            turn_number: 0,
        }
    }

    pub fn simulate_turn(&mut self, mut chosen_actions: ChosenActions) -> TurnOutcome {
        match self.turn_number.checked_add(1) {
            Some(turn_number) => self.turn_number = turn_number,
            None => {
                return Err(SimError::InvalidStateError(String::from(
                    "Turn limit exceeded (Limit = 255 turns)",
                )))
            }
        };

        self.ctx
            .push_messages(&[&format!["Turn {}", self.turn_number], &EMPTY_LINE]);

        ordering::sort_by_activation_order(
            &mut self.prng,
            &mut chosen_actions,
            &mut |choice| self.ctx.choice_activation_order(choice),
        );

        let mut result = Ok(());
        for chosen_action in chosen_actions.into_iter() {
            self.ctx.current_action = Some(chosen_action);
            result = match chosen_action {
                ActionChoice::Move {
                    move_uid,
                    target_uid,
                } => match self.ctx.move_(move_uid).category() {
                    MoveCategory::Physical | MoveCategory::Special => PrimaryAction::damaging_move(
                        &mut self.ctx,
                        &mut self.prng,
                        move_uid,
                        target_uid,
                    ),
                    MoveCategory::Status => PrimaryAction::status_move(
                        &mut self.ctx,
                        &mut self.prng,
                        move_uid,
                        target_uid,
                    ),
                },
            };
            // Check if any monster fainted due to the last action.
            if let Some(battler) = self.ctx.battlers().find(|it| it.fainted()) {
                self.ctx
                    .push_message(&format!["{} fainted!", battler.monster.nickname]);
                self.ctx.sim_state = SimState::Finished;
                break;
            };
            self.ctx.push_message(&EMPTY_LINE);
        }

        result
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimError {
    InvalidStateError(String),
    InputError(String),
}
