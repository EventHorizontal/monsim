mod ui;
use ui::Ui;

/*
TODO: We will probably be abandoning this TUI, either just using the CLI or replacing it with 
the not-deprecated `ratutui` crate. This is just a message to indicate that and that this 
module has been rotting for a while, since I turned it off to improve build times. So expect to 
have to rewrite some or all of this later.
*/

use std::{error::Error, io::Stdout, sync::mpsc, thread, time::{Duration, Instant}};

use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use monsim_utils::{MaxSizedVec, Nothing, NOTHING};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{sim::{BattleSimulator, BattleState, FullySpecifiedChoice, MonsterID, PartiallySpecifiedChoice, PerTeam, TeamID, EMPTY_LINE}, ActivationOrder, AvailableChoicesForTeam};

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
        active_monster_id: MonsterID,
        switchable_benched_monster_ids: MaxSizedVec<MonsterID, 5>,
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
                
                for team_id in [TeamID::Allies, TeamID::Opponents] {
                    if let FullySpecifiedChoice::SwitchOut { .. } = choices[team_id] {
                        ui.clear_choice_menu_selection_for_team(team_id);
                        choices_for_turn[team_id] = None;
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
                    let activation_order = { if let FullySpecifiedChoice::SwitchOut { activation_order, .. } = choices[fainted_battler.id.team_id] { activation_order } else { unreachable!() } };
                    // FIXME: We cannot handle multiple simultaneous fainted battlers with this logic
                    current_app_state.transition(Some(AppState::AcceptingInput(InputMode::SwitcheePrompt { 
                        is_between_turn_switch: true,
                        active_monster_id: fainted_battler.id,
                        switchable_benched_monster_ids: battle.switchable_benched_monster_ids(fainted_battler.id.team_id),
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
                    if let Some((selected_menu_item_index, team_id)) = maybe_selected_menu_item {
                        let available_choices_for_team = available_choices[team_id];
                        let selected_choice = available_choices_for_team[selected_menu_item_index];
                        match selected_choice {
                            PartiallySpecifiedChoice::Move { attacker_id, move_id, target_id, activation_order, .. } => {
                                choices_for_turn[team_id] = Some(FullySpecifiedChoice::Move { attacker_id, move_id, target_id, activation_order })
                            },
                            PartiallySpecifiedChoice::SwitchOut { active_monster_id, switchable_benched_monster_ids, activation_order, .. } => {
                                // Update the switchee list when the switch option is selected.
                                return Some(AppState::AcceptingInput(InputMode::SwitcheePrompt {
                                    is_between_turn_switch: false,
                                    active_monster_id,
                                    switchable_benched_monster_ids,
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

        InputMode::SwitcheePrompt { is_between_turn_switch, active_monster_id, switchable_benched_monster_ids, highlight_cursor, activation_order} => {
            match pressed_key {
                KeyCode::Esc => {
                    Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_choices())))
                },

                KeyCode::Up => {
                    let list_length = switchable_benched_monster_ids.iter().count();
                    Ui::scroll_up_wrapped(highlight_cursor, list_length);
                    None
                },
                KeyCode::Down => { 
                    let list_length = switchable_benched_monster_ids.iter().count();
                    Ui::scroll_down_wrapped(highlight_cursor, list_length); 
                    None 
                },
                KeyCode::Enter => {
                    let benched_monster_id = switchable_benched_monster_ids[*highlight_cursor]; 
                    // HACK: no formal way to switch out between turns.
                    if *is_between_turn_switch {
                        let _ = BattleSimulator::switch_out_between_turns(battle, *active_monster_id, benched_monster_id);
                        ui.clear_choice_menu_selection_for_team(active_monster_id.team_id);
                        // HACK: This fixes the issue of targetting the previous fainted foe until we have a more robust targetting system
                        ui.clear_choice_menu_selection_for_team(active_monster_id.team_id.other());
                        ui.update_team_status_panels(battle);
                        ui.update_message_log(battle.message_log.len());

                        *choices_for_turn = PerTeam::both(None);
                    } else {
                        choices_for_turn[active_monster_id.team_id] = Some(FullySpecifiedChoice::SwitchOut { active_monster_id: *active_monster_id, benched_monster_id, activation_order: *activation_order });
                    }
                    Some(AppState::AcceptingInput(InputMode::MidBattle(battle.available_choices())))
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