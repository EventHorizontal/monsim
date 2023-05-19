use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use monsim::*;
use tui::{
    backend::CrosstermBackend,
    widgets::{ListItem, ListState},
    Terminal,
};
mod render;
use render::render;

const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: u64 = 20;
type MonsimIOResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Debug)]
enum AppMode {
    AwaitingUserInput { available_actions: AvailableActions },
    Simulating { chosen_actions: ChosenActions },
}

#[derive(Debug, PartialEq, Eq)]
enum ScrollableWidgets {
    AllyTeamStatus,
    OpponentTeamStatus,
    MessageLog,
    AllyTeamChoices,
    OpponentTeamChoices,
}

#[derive(Debug)]
pub struct AppState<'a> {
    app_mode: AppMode,
    ally_list_items: Vec<ListItem<'a>>,
    ally_list_state: ListState,
    opponent_list_items: Vec<ListItem<'a>>,
    opponent_list_state: ListState,
    message_buffer: MessageBuffer,
    selected_list: ScrollableWidgets,
    message_log_scroll_idx: usize,
    is_battle_ongoing: bool,
}

impl<'a> AppState<'a> {
    fn new(battle_context: &mut BattleContext) -> Self {
        let mut state = Self {
            app_mode: AppMode::AwaitingUserInput {
                available_actions: battle_context.generate_available_actions(),
            },
            ally_list_items: Vec::with_capacity(4),
            ally_list_state: {
                let mut list = ListState::default();
                list.select(Some(0));
                list
            },
            opponent_list_items: Vec::with_capacity(4),
            opponent_list_state: {
                let mut list = ListState::default();
                list.select(Some(0));
                list
            },
            message_buffer: Vec::with_capacity(CONTEXT_MESSAGE_BUFFER_SIZE),
            selected_list: ScrollableWidgets::MessageLog,
            message_log_scroll_idx: 0,
            is_battle_ongoing: true,
        };
        state.build_list_items(battle_context);
        state
    }

    fn build_list_items(&mut self, battle_context: &BattleContext) {
        let available_actions = battle_context.generate_available_actions();
        (self.ally_list_items, self.opponent_list_items) = {
            (
                available_actions
                    .ally_team_choices
                    .iter()
                    .map(|choice| {
                        ListItem::new(
                            battle_context
                                .move_({
                                    let ActionChoice::Move {
                                        move_uid,
                                        target_uid: _,
                                    } = choice;
                                    *move_uid
                                })
                                .species
                                .name,
                        )
                    })
                    .collect(),
                available_actions
                    .opponent_team_choices
                    .iter()
                    .map(|choice| {
                        ListItem::new(
                            battle_context
                                .move_({
                                    let ActionChoice::Move {
                                        move_uid,
                                        target_uid: _,
                                    } = choice;
                                    *move_uid
                                })
                                .species
                                .name,
                        )
                    })
                    .collect(),
            )
        };
    }

    fn ally_list_length(&self) -> usize {
        self.ally_list_items.len()
    }

    fn opponent_list_length(&self) -> usize {
        self.opponent_list_items.len()
    }
}

enum TuiEvent<I> {
    Input(I),
    Tick,
}

fn main() -> MonsimIOResult {
    let mut battle = Battle::new(bcontext!(
        {
            AllyTeam {
                mon Torchic "Ruby" {
                    mov Ember,
                    mov Scratch,
                    mov Growl,
                    mov Bubble,
                    abl FlashFire,
                },
                mon Mudkip "Sapphire" {
                    mov Tackle,
                    mov Bubble,
                    abl FlashFire,
                },
                mon Torchic "Emerald" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            },
            OpponentTeam {
                mon Drifloon "Cheerio" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            }
        }
    ));

    // Initialise the Program
    let mut app_state = AppState::new(&mut battle.context);

    // Raw mode allows to not require enter presses to get
    enable_raw_mode().expect("Raw mode should always enableable.");

    // Construct an mpsc channel to communicate between main thread and io thread
    let (sender, receiver) = mpsc::channel();
    let time_out_duration = Duration::from_millis(TUI_INPUT_POLL_TIMEOUT_MILLISECONDS);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = time_out_duration
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("Polling should be OK") {
                if let Event::Key(key) =
                    event::read().expect("The poll should always be successful")
                {
                    sender
                        .send(TuiEvent::Input(key))
                        .expect("The event should always be sendable.");
                }
            }

            // Nothing happened, send a tick event
            if last_tick.elapsed() >= time_out_duration && sender.send(TuiEvent::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    // Create a new TUI terminal with the CrossTerm backend and stdout.
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    'app: loop {
        match app_state.app_mode {
            AppMode::AwaitingUserInput {
                ref available_actions,
            } => {
                // Do appropriate thing based on input receieved
                match receiver.recv()? {
                    TuiEvent::Input(event) => {
                        if app_state.is_battle_ongoing {
                            match (event.code, event.kind) {
                                (KeyCode::Esc, KeyEventKind::Release) => {
                                    disable_raw_mode()?;
                                    execute!(std::io::stdout(), LeaveAlternateScreen)?;
                                    terminal.show_cursor()?;
                                    break 'app;
                                }
                                (KeyCode::Up, KeyEventKind::Release) => {
                                    match app_state.selected_list {
                                        ScrollableWidgets::AllyTeamChoices => {
                                            if let Some(selected_index) =
                                                app_state.ally_list_state.selected()
                                            {
                                                let ally_list_length = app_state.ally_list_length();
                                                app_state.ally_list_state.select(Some(
                                                    (selected_index + ally_list_length - 1)
                                                        % ally_list_length,
                                                ));
                                            } else {
                                                app_state.ally_list_state.select(Some(0));
                                            }
                                        }
                                        ScrollableWidgets::OpponentTeamChoices => {
                                            if let Some(selected_index) =
                                                app_state.opponent_list_state.selected()
                                            {
                                                let opponent_list_items_length =
                                                    app_state.opponent_list_length();
                                                app_state.opponent_list_state.select(Some(
                                                    (selected_index + opponent_list_items_length
                                                        - 1)
                                                        % opponent_list_items_length,
                                                ));
                                            } else {
                                                app_state.opponent_list_state.select(Some(0));
                                            }
                                        }
                                        ScrollableWidgets::MessageLog => {
                                            app_state.message_log_scroll_idx =
                                                app_state.message_log_scroll_idx.saturating_sub(1);
                                        }
                                        ScrollableWidgets::AllyTeamStatus => todo!(),
                                        ScrollableWidgets::OpponentTeamStatus => todo!(),
                                    }
                                }
                                (KeyCode::Down, KeyEventKind::Release) => {
                                    match app_state.selected_list {
                                        ScrollableWidgets::AllyTeamChoices => {
                                            if let Some(selected_index) =
                                                app_state.ally_list_state.selected()
                                            {
                                                let ally_list_length = app_state.ally_list_length();
                                                app_state.ally_list_state.select(Some(
                                                    (selected_index + 1) % ally_list_length,
                                                ))
                                            } else {
                                                app_state.ally_list_state.select(Some(0));
                                            }
                                        }
                                        ScrollableWidgets::OpponentTeamChoices => {
                                            if let Some(selected_index) =
                                                app_state.opponent_list_state.selected()
                                            {
                                                let opponent_list_items_length =
                                                    app_state.opponent_list_length();
                                                app_state.opponent_list_state.select(Some(
                                                    (selected_index + 1)
                                                        % opponent_list_items_length,
                                                ))
                                            } else {
                                                app_state.opponent_list_state.select(Some(0));
                                            }
                                        }
                                        ScrollableWidgets::MessageLog => {
                                            app_state.message_log_scroll_idx =
                                                (app_state.message_log_scroll_idx + 1)
                                                    .min(battle.context.message_buffer.len());
                                        }
                                        ScrollableWidgets::AllyTeamStatus => todo!(),
                                        ScrollableWidgets::OpponentTeamStatus => todo!(),
                                    }
                                }
                                (KeyCode::Left, KeyEventKind::Release) => {
                                    match app_state.selected_list {
                                        ScrollableWidgets::AllyTeamStatus => todo!(),
                                        ScrollableWidgets::OpponentTeamStatus => todo!(),
                                        ScrollableWidgets::MessageLog => {
                                            app_state.selected_list =
                                                ScrollableWidgets::AllyTeamChoices;
                                        }
                                        ScrollableWidgets::AllyTeamChoices => {
                                            app_state.selected_list =
                                                ScrollableWidgets::OpponentTeamChoices;
                                            app_state.ally_list_state.select(None);
                                        }
                                        ScrollableWidgets::OpponentTeamChoices => {
                                            app_state.selected_list = ScrollableWidgets::MessageLog;
                                        }
                                    }
                                }
                                (KeyCode::Right, KeyEventKind::Release) => {
                                    match app_state.selected_list {
                                        ScrollableWidgets::AllyTeamStatus => todo!(),
                                        ScrollableWidgets::OpponentTeamStatus => todo!(),
                                        ScrollableWidgets::MessageLog => {
                                            app_state.selected_list =
                                                ScrollableWidgets::OpponentTeamChoices;
                                        }
                                        ScrollableWidgets::AllyTeamChoices => {
                                            app_state.selected_list = ScrollableWidgets::MessageLog;
                                        }
                                        ScrollableWidgets::OpponentTeamChoices => {
                                            app_state.selected_list =
                                                ScrollableWidgets::AllyTeamChoices;
                                        }
                                    }
                                }
                                (KeyCode::Tab, KeyEventKind::Release) => {
                                    if let (
                                        Some(selected_ally_choice_index),
                                        Some(selected_opponent_choice_index),
                                    ) = (
                                        app_state.ally_list_state.selected(),
                                        app_state.opponent_list_state.selected(),
                                    ) {
                                        let chosen_actions = vec![
                                            available_actions.ally_team_choices
                                                [selected_ally_choice_index],
                                            available_actions.opponent_team_choices
                                                [selected_opponent_choice_index],
                                        ];
                                        app_state.app_mode = AppMode::Simulating { chosen_actions };
                                    }
                                }
                                _ => (),
                            }
                        } else {
                            match (event.code, event.kind) {
                                (KeyCode::Esc, KeyEventKind::Release) => {
                                    disable_raw_mode()?;
                                    execute!(std::io::stdout(), LeaveAlternateScreen)?;
                                    terminal.show_cursor()?;
                                    break 'app;
                                }
                                (KeyCode::Up, KeyEventKind::Release) => {
                                    app_state.message_log_scroll_idx =
                                        app_state.message_log_scroll_idx.saturating_sub(1);
                                }
                                (KeyCode::Down, KeyEventKind::Release) => {
                                    app_state.message_log_scroll_idx =
                                        (app_state.message_log_scroll_idx + 1)
                                            .min(battle.context.message_buffer.len());
                                }
                                _ => (),
                            }
                        }
                    }
                    TuiEvent::Tick => {}
                };
            }
            AppMode::Simulating { chosen_actions } => {
                let result = battle.simulate_turn(chosen_actions); // <- This is the main use of the monsim library
                match result {
                    Ok(_) => {
                        battle
                            .context
                            .message_buffer
                            .push(String::from("(The turn was calculated successfully.)"));
                    }
                    Err(error) => battle.context.message_buffer.push(format!["{:?}", error]),
                }
                if battle.context.state == BattleState::Finished {
                    app_state.is_battle_ongoing = false;
                    battle
                        .context
                        .message_buffer
                        .push(String::from("The battle ended."));
                }
                battle
                    .context
                    .message_buffer
                    .extend([String::from("---"), String::from(EMPTY_LINE)].into_iter());
                app_state.app_mode = AppMode::AwaitingUserInput {
                    available_actions: battle.context.generate_available_actions(),
                };
                app_state.message_buffer = battle.context.message_buffer.clone();
            }
        }

        //TODO: Move the context usage inside app state
        render(&mut terminal, &mut app_state, &battle.context)?;
    }

    println!("monsim_tui exited successfully");
    Ok(())
}
