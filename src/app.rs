mod ui;
use ui::Ui;

use std::{error::Error, io::Stdout, sync::mpsc, thread, time::{Duration, Instant}};

use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use monsim_utils::{ArrayOfOptionals, Nothing, NOTHING};
use tui::{backend::CrosstermBackend, Terminal};

use crate::sim::{AvailableActions, Battle, BattleSimulator, MonsterUID, ChosenActionsForTurn, FullySpecifiedAction, PartiallySpecifiedAction, PerTeam, TeamID, EMPTY_LINE};

pub type AppResult<S> = Result<S, Box<dyn Error>>;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
enum AppState {
    AcceptingInput(InputMode),
    Simulating(ChosenActionsForTurn),
    Terminating,
}

impl AppState {
    fn transition(&mut self, optional_new_app_state: Option<AppState>) {
        if let Some(new_app_state) = optional_new_app_state { 
            *self = new_app_state 
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum InputMode {
    MidBattle(AvailableActions),
    SwitcheePrompt { 
        switcher_uid: MonsterUID,
        possible_switchee_uids: ArrayOfOptionals<MonsterUID, 5>,
        highlight_cursor: usize,
    },
    PostBattle,
}

/// The main function for the application
pub fn run(mut battle: Battle) -> AppResult<Nothing> {
    let (sender, receiver) = mpsc::channel();
    spawn_input_capturing_thread(sender);
    
    let available_actions = battle.available_actions();
    let mut current_app_state = AppState::AcceptingInput(
        InputMode::MidBattle(available_actions)
    );
    
    let mut terminal = acquire_terminal()?;
    let mut ui = Ui::new(&battle);
    ui.render(&mut terminal, &battle, &current_app_state)?;

    // Will store action related information until they can be built.
    let mut chosen_actions_for_turn = PerTeam::new(None, None);

    'main: loop {
        match &mut current_app_state {
            
            AppState::AcceptingInput(input_mode) => {
                // The app information only updates when input is received from the io thread. This may change in the future.
                if let Some(pressed_key) = get_pressed_key(&receiver)? {
                    let optional_new_app_state = update_from_input(
                        &mut ui,
                        &mut battle,
                        input_mode, 
                        &mut chosen_actions_for_turn,
                        pressed_key
                    );
                    ui.update_team_status_panels(&battle);
                    current_app_state.transition(optional_new_app_state);
                }
            },

            AppState::Simulating(chosen_actions) => {
                let _turn_result = BattleSimulator::simulate_turn(&mut battle, *chosen_actions);
                #[cfg(feature = "debug")]
                match _turn_result {
                    Ok(_) => battle.push_messages_to_log(
                        &[
                            "Simulator: The turn was calculated successfully.", 
                            EMPTY_LINE
                        ]
                    ),
                    Err(error) => battle.push_message_to_log(&format!["Simulator: {:?}", error]),
                };
                
                // TODO: Investigate whether updating the message log seperately is worth the possible syncing issues
                ui.update_message_log(battle.message_log.len());
                ui.update_team_status_panels(&battle);
                
                for team_id in [TeamID::Allies, TeamID::Opponents] {
                    if let FullySpecifiedAction::SwitchOut { .. } = chosen_actions[team_id] {
                        ui.reset_team_choice_menu(team_id);
                        chosen_actions_for_turn[team_id] = None;
                    }
                    
                }
                
                if battle.is_finished {
                    current_app_state.transition(Some(AppState::AcceptingInput(InputMode::PostBattle)));
                } else {
                    current_app_state.transition(Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_actions()))))
                }
            },
            
            AppState::Terminating => {
                release_terminal(&mut terminal)?;
                break 'main;
            },
        }
        ui.render(&mut terminal, &battle, &current_app_state)?;
    }

    println!("monsim_tui exited successfully");
    Ok(NOTHING)
}

/// Updates the `Ui` according to the input received and optionally returns a new `AppState` to transition to.
fn update_from_input(
    ui: &mut Ui,
    battle: &mut Battle,
    current_input_mode: &mut InputMode,
    chosen_actions_for_turn: &mut PerTeam<Option<FullySpecifiedAction>>,
    pressed_key: KeyCode,
) -> Option<AppState> {
    match current_input_mode {
        InputMode::MidBattle(available_actions) => {
            match pressed_key {
                KeyCode::Esc => { Some(AppState::Terminating) },

                KeyCode::Up => { ui.scroll_current_widget_up(); None },
                KeyCode::Down => { ui.scroll_current_widget_down(); None },
                KeyCode::Left => { ui.select_left_widget(); None },
                KeyCode::Right => { ui.select_right_widget(); None },
                KeyCode::Enter => {
                    let maybe_selected_menu_item = ui.select_currently_hightlighted_menu_item();
                    if let Some((selected_menu_item_index, team_id)) = maybe_selected_menu_item {
                        let available_actions_for_team = available_actions[team_id];
                        let selected_action = available_actions_for_team.get_by_index(selected_menu_item_index);
                        match selected_action {
                            PartiallySpecifiedAction::Move { move_uid, target_uid, .. } => {
                                chosen_actions_for_turn[team_id] = Some(FullySpecifiedAction::Move { move_uid, target_uid })
                            },
                            PartiallySpecifiedAction::SwitchOut { switcher_uid, possible_switchee_uids } => {
                                // Update the switchee list when the switch option is selected.
                                return Some(AppState::AcceptingInput(InputMode::SwitcheePrompt {
                                    switcher_uid,
                                    possible_switchee_uids,
                                    highlight_cursor: 0,
                                }));
                            },
                        }
                    }
                    None
                },
                KeyCode::Tab => { 
                    if let (Some(ally_team_action), Some(opponent_team_action)) = (chosen_actions_for_turn[TeamID::Allies], chosen_actions_for_turn[TeamID::Opponents]) {
                        Some(AppState::Simulating(PerTeam::new(ally_team_action, opponent_team_action)))
                    } else {
                        battle.push_messages_to_log(
                            &[
                                &"Simulator: Actions were not chosen... please select something before activating the simulation.",
                                &"---",
                                &EMPTY_LINE
                            ]
                        );
                        ui.snap_message_log_cursor_to_beginning_of_last_message();
                        None
                    }
                 },
                 _ => None
            }
        },

        InputMode::SwitcheePrompt { switcher_uid, possible_switchee_uids, highlight_cursor} => {
            match pressed_key {
                KeyCode::Esc => {
                    Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_actions())))
                },

                KeyCode::Up => {
                    let list_length = possible_switchee_uids.iter().flatten().count();
                    Ui::scroll_up_wrapped(highlight_cursor, list_length);
                    None
                },
                KeyCode::Down => { 
                    let list_length = possible_switchee_uids.iter().flatten().count();
                    Ui::scroll_down_wrapped(highlight_cursor, list_length); 
                    None 
                },
                KeyCode::Enter => {
                    if let Some(switchee_uid) = possible_switchee_uids[*highlight_cursor] {
                        chosen_actions_for_turn[switcher_uid.team_id] = Some(FullySpecifiedAction::SwitchOut { switcher_uid: *switcher_uid, switchee_uid });
                        Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_actions())))
                    } else {
                        None
                    }
                }
                _ => None
            }
        },

        InputMode::PostBattle => {
            match pressed_key {
                KeyCode::Esc => { Some(AppState::Terminating) },

                KeyCode::Up => { ui.scroll_message_log_up(); None },
                KeyCode::Down => { ui.scroll_message_log_down(); None },
                _ => None
            }
        },
    }
}

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

fn acquire_terminal() -> AppResult<TuiTerminal> {
    // Raw mode allows us to avoid requiring enter presses to get input
    enable_raw_mode().expect("Enabling raw mode should always work.");
    
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    Ok(terminal)
}

enum TuiEvent<I> {
    Input(I),
    Tick,
}

const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: Duration = Duration::from_millis(20);

fn spawn_input_capturing_thread(sender: mpsc::Sender<TuiEvent<KeyEvent>>) {
    let time_out_duration = TUI_INPUT_POLL_TIMEOUT_MILLISECONDS;
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = time_out_duration.checked_sub(last_tick.elapsed()).unwrap_or(Duration::from_secs(0));

            if event::poll(timeout).expect("Polling should be OK") {
                if let Event::Key(key) = event::read().expect("The poll should always be successful") {
                    sender.send(TuiEvent::Input(key)).expect("The event should always be sendable.");
                }
            }

            // Nothing happened, send a tick event
            if last_tick.elapsed() >= time_out_duration && sender.send(TuiEvent::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });
}

fn get_pressed_key(receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>) -> AppResult<Option<KeyCode>> {
    Ok(match receiver.recv()? {
        TuiEvent::Input(key_event) => {
            // Reminder: Linux does not have the Release and Repeat Flags enabled by default
            // As such I'm going to avoid using the extra flags, hopefully we don't get weird behaviour.
            if key_event.kind == KeyEventKind::Press { Some(key_event.code) } else { None }
        },
        TuiEvent::Tick => None,
    })
}

fn release_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> AppResult<Nothing> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    #[cfg(feature="debug")]
    crate::debug::remove_debug_log_file()?;
    Ok(NOTHING)
}