mod ui;
use ui::Ui;

use std::{error::Error, io::Stdout, sync::mpsc, thread, time::{Duration, Instant}};

use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use monsim_utils::{ArrayOfOptionals, Nothing, NOTHING};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{sim::{BattleSimulator, BattleState, FullySpecifiedChoice, MonsterUID, PartiallySpecifiedChoice, PerTeam, TeamUID, EMPTY_LINE}, ActivationOrder, AvailableChoicesForTeam};

pub type TuiResult<S> = Result<S, Box<dyn Error>>;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
enum AppState {
    AcceptingInput(InputMode),
    Simulating(PerTeam<FullySpecifiedChoice>),
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
    MidBattle(PerTeam<AvailableChoicesForTeam>),
    SwitcheePrompt {
        is_between_turn_switch: bool,
        active_monster_uid: MonsterUID,
        switchable_benched_monster_uids: ArrayOfOptionals<MonsterUID, 5>,
        activation_order: ActivationOrder,
        highlight_cursor: usize,
    },
    PostBattle,
}

/// The main function for the application
pub fn run(mut battle: BattleState) -> TuiResult<Nothing> {
    let (sender, receiver) = mpsc::channel();
    spawn_input_capturing_thread(sender);
    
    let available_choices = battle.available_choices();
    let mut current_app_state = AppState::AcceptingInput(
        InputMode::MidBattle(available_choices)
    );
    
    let mut terminal = acquire_terminal()?;
    let mut ui = Ui::new(&battle);
    ui.render(&mut terminal, &battle, &current_app_state)?;

    // Will store choice related information until they can be built.
    let mut choices_for_turn = PerTeam::both(None);

    'main: loop {
        match &mut current_app_state {
            
            AppState::AcceptingInput(input_mode) => {
                // The app information only updates when input is received from the io thread. This may change in the future.
                if let Some(pressed_key) = get_pressed_key(&receiver)? {
                    let optional_new_app_state = update_from_input(
                        &mut ui,
                        &mut battle,
                        input_mode, 
                        &mut choices_for_turn,
                        pressed_key
                    );
                    ui.update_team_status_panels(&battle);
                    current_app_state.transition(optional_new_app_state);
                }
            },

            AppState::Simulating(choices) => {
                let _turn_result = BattleSimulator::simulate_turn(&mut battle, *choices);
                #[cfg(feature = "debug")]
                match _turn_result {
                    Ok(_) => battle.message_log.extend(
                        &[
                            "Simulator: The turn was calculated successfully.", 
                            EMPTY_LINE
                        ]
                    ),
                    Err(error) => battle.message_log.push(format!["Simulator: {:?}", error]),
                };
                
                // TODO: Investigate whether updating the message log seperately is worth the possible syncing issues
                ui.update_message_log(battle.message_log.len());
                ui.update_team_status_panels(&battle);
                
                for team_uid in [TeamUID::Allies, TeamUID::Opponents] {
                    if let FullySpecifiedChoice::SwitchOut { .. } = choices[team_uid] {
                        ui.clear_choice_menu_selection_for_team(team_uid);
                        choices_for_turn[team_uid] = None;
                    }
                }
                
                // If a Monster has fainted, we need to switch it out
                let maybe_fainted_active_battler = battle.active_monsters()
                    .into_iter()
                    .find(|monster| { monster.is_fainted });
                if battle.is_finished {
                    current_app_state.transition(Some(AppState::AcceptingInput(InputMode::PostBattle)));
                } else if let Some(fainted_battler) = maybe_fainted_active_battler {
                    // HACK: Quickly cobbled together a way to get the ActivationOrder here, but not sure
                    // if this is guaranteed to give the right one, especially if there are 
                    // multiple monsters.
                    let activation_order = { if let FullySpecifiedChoice::SwitchOut { activation_order, .. } = choices[fainted_battler.uid.team_uid] { activation_order } else { unreachable!() } };
                    // FIXME: We cannot handle multiple simultaneous fainted battlers with this logic
                    current_app_state.transition(Some(AppState::AcceptingInput(InputMode::SwitcheePrompt { 
                        is_between_turn_switch: true,
                        active_monster_uid: fainted_battler.uid,
                        switchable_benched_monster_uids: battle.switchable_benched_monster_uids(fainted_battler.uid.team_uid),
                        activation_order,
                        highlight_cursor: 0 
                    })));
                } else {
                    current_app_state.transition(Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_choices()))))
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
    battle: &mut BattleState,
    current_input_mode: &mut InputMode,
    choices_for_turn: &mut PerTeam<Option<FullySpecifiedChoice>>,
    pressed_key: KeyCode,
) -> Option<AppState> {
    match current_input_mode {
        InputMode::MidBattle(available_choices) => {
            match pressed_key {
                KeyCode::Esc => { Some(AppState::Terminating) },

                KeyCode::Up => { ui.scroll_current_widget_up(); None },
                KeyCode::Down => { ui.scroll_current_widget_down(); None },
                KeyCode::Left => { ui.select_left_widget(); None },
                KeyCode::Right => { ui.select_right_widget(); None },
                KeyCode::Enter => {
                    let maybe_selected_menu_item = ui.select_currently_hightlighted_menu_item();
                    if let Some((selected_menu_item_index, team_uid)) = maybe_selected_menu_item {
                        let available_choices_for_team = available_choices[team_uid];
                        let selected_choice = available_choices_for_team.get_by_index(selected_menu_item_index);
                        match selected_choice {
                            PartiallySpecifiedChoice::Move { move_uid, target_uid, activation_order, .. } => {
                                choices_for_turn[team_uid] = Some(FullySpecifiedChoice::Move { move_uid, target_uid, activation_order })
                            },
                            PartiallySpecifiedChoice::SwitchOut { active_monster_uid, switchable_benched_monster_uids, activation_order, .. } => {
                                // Update the switchee list when the switch option is selected.
                                return Some(AppState::AcceptingInput(InputMode::SwitcheePrompt {
                                    is_between_turn_switch: false,
                                    active_monster_uid,
                                    switchable_benched_monster_uids,
                                    activation_order,
                                    highlight_cursor: 0,
                                }));
                            },
                        }
                    }
                    None
                },
                KeyCode::Tab => { 

                    if let (Some(ally_team_choice), Some(opponent_team_choice)) = choices_for_turn.to_option_pair() {
                        Some(AppState::Simulating(PerTeam::new(ally_team_choice, opponent_team_choice)))
                    } else {
                        battle.message_log.extend(
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

        InputMode::SwitcheePrompt { is_between_turn_switch, active_monster_uid, switchable_benched_monster_uids, highlight_cursor, activation_order} => {
            match pressed_key {
                KeyCode::Esc => {
                    Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_choices())))
                },

                KeyCode::Up => {
                    let list_length = switchable_benched_monster_uids.iter().flatten().count();
                    Ui::scroll_up_wrapped(highlight_cursor, list_length);
                    None
                },
                KeyCode::Down => { 
                    let list_length = switchable_benched_monster_uids.iter().flatten().count();
                    Ui::scroll_down_wrapped(highlight_cursor, list_length); 
                    None 
                },
                KeyCode::Enter => {
                    if let Some(benched_monster_uid) = switchable_benched_monster_uids[*highlight_cursor] {
                        // HACK: cleaner/more systematic way to do this?
                        if *is_between_turn_switch {
                            let _ = BattleSimulator::switch_out_between_turns(battle, *active_monster_uid, benched_monster_uid);
                            ui.clear_choice_menu_selection_for_team(active_monster_uid.team_uid);
                            // HACK: This fixes the issue of targetting the previous fainted foe until we have a more robust targetting system
                            ui.clear_choice_menu_selection_for_team(active_monster_uid.team_uid.other());
                            ui.update_team_status_panels(battle);
                            ui.update_message_log(battle.message_log.len());

                            *choices_for_turn = PerTeam::both(None);
                        } else {
                            choices_for_turn[active_monster_uid.team_uid] = Some(FullySpecifiedChoice::SwitchOut { active_monster_uid: *active_monster_uid, benched_monster_uid, activation_order: *activation_order });
                        }
                        Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_choices())))
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

fn acquire_terminal() -> TuiResult<TuiTerminal> {
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

fn get_pressed_key(receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>) -> TuiResult<Option<KeyCode>> {
    Ok(match receiver.recv()? {
        TuiEvent::Input(key_event) => {
            // Reminder: Linux does not have the Release and Repeat Flags enabled by default
            // As such I'm going to avoid using the extra flags, hopefully we don't get weird behaviour.
            if key_event.kind == KeyEventKind::Press { Some(key_event.code) } else { None }
        },
        TuiEvent::Tick => None,
    })
}

fn release_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> TuiResult<Nothing> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    #[cfg(feature="debug")]
    crate::debug::remove_debug_log_file()?;
    Ok(NOTHING)
}