use std::{sync::mpsc, time::{Duration, Instant}, thread, io::Stdout};

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode}, event::{self, Event, KeyCode, KeyEventKind, KeyEvent}, execute};
use tui::{backend::CrosstermBackend, Terminal, terminal::CompletedFrame, widgets::{ListState, ListItem, Paragraph, Block, Borders, Wrap, List}, layout::{Layout, Direction, Constraint, Rect, Alignment}, Frame, text::{Span, Spans}, style::{Style, Color, Modifier}};

use crate::sim::{BattleSimulator, Battle, ChosenActions, EMPTY_LINE, SimState, AvailableActions, ActionChoice, BattlerTeam, MessageBuffer, TeamID, TeamAvailableActions, utils::{NOTHING, Nothing, not}, EnumeratedActionChoice, BattlerUID};

//TODO: Reorganise later...

#[derive(Debug, Clone)]
pub struct App<'a> {
    state: AppState<'a>,
    currently_selected_widget: SelectableWidget,
    message_log_ui_state: MessageLogUiState,
    ally_ui_state: TeamUiState<'a>,
    opponent_ui_state: TeamUiState<'a>,
}

#[derive(Debug, Clone)]
pub enum AppState<'a> {
    Initialising,
    Processing(ProcessingState),
    PromptSwitchOut(SwitchOutState<'a>),
    Exiting,
}

impl<'a> PartialEq for AppState<'a> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessingState {
    AwaitingUserInput(AvailableActions),
    Simulating(ChosenActions),
}

#[derive(Debug, Clone)]
pub struct MessageLogUiState {
    message_log_scroll_idx: usize,
    message_log_last_scrollable_line_idx: usize,
    last_message_buffer_length: usize,
}

#[derive(Debug, Clone)]
pub struct TeamUiState<'a> {
    team_id: TeamID,
    active_battler_status: String,
    team_roster_status: String,
    list_items: Vec<ListItem<'a>>,	
    list_state: ListState,
    selected_action: Option<EnumeratedActionChoice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectableWidget {
    AllyChoices,
    MessageLog,
    OpponentChoices,
    AllyRoster,
    OpponentRoster,
}

#[derive(Debug, Clone)]
pub struct SwitchOutState<'a> {
    switching_battler: BattlerUID,
    team_id: TeamID,
    list_of_choices: Vec<(BattlerUID, &'a str, &'a str)>,
    list_state: ListState,
}

const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: Duration = Duration::from_millis(20);
enum TuiEvent<I> {
    Input(I),
    Tick,
}

type BoxedError = Box<dyn std::error::Error>;
pub type AppResult<T> = Result<T, BoxedError>;

pub fn run(mut battle_sim: BattleSimulator) -> AppResult<Nothing> {
    
    use ProcessingState::AwaitingUserInput;

    let mut app = App::new(&mut battle_sim.battle);

    // Raw mode allows us to not require enter presses to get input
    enable_raw_mode().expect("Enabling raw mode should always work.");

    let (sender, receiver) = mpsc::channel();
    create_io_thread(sender);

    // Create a new TUI terminal with the CrossTerm backend and stdout.
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    app.state = AppState::Processing(AwaitingUserInput(battle_sim.battle.available_actions()));
    
    render_interface(&mut terminal, &mut app, &battle_sim.battle.message_buffer)?;

    // TODO: bring all the app state changes up to here, using returns.
    'app: loop {
        match app.state {
            AppState::Initialising => unreachable!("The app never transitions back to the initialising state."),
            AppState::Processing(ref processing_state) => {
                match processing_state.clone() {
                    ProcessingState::AwaitingUserInput(available_actions) => {
                        process_awaiting_input_state(&mut terminal, &mut app, &mut battle_sim, &receiver, available_actions)?;
                    },
                    ProcessingState::Simulating(chosen_actions) => {
                        let result = battle_sim.simulate_turn(chosen_actions);
                        match result {
                            Ok(_) => {
                                battle_sim.battle.push_message(&"Simulator: The turn was calculated successfully.");
                            }
                            Err(error) => battle_sim.battle.push_message(&format!["Simulator: {:?}", error]),
                        }
                        
                        if battle_sim.sim_state == SimState::BattleFinished {
                            battle_sim.battle.push_messages(&[&EMPTY_LINE, &"The battle ended."]);
                        }
                        battle_sim.battle.push_messages(&[&"---", &EMPTY_LINE]);
                        
                        let available_actions = battle_sim.battle.available_actions();
                        app.state = AppState::Processing(AwaitingUserInput(available_actions));
                        app.regenerate_ui_data(&mut battle_sim.battle, available_actions);
                    },
                }
            }
            AppState::PromptSwitchOut(SwitchOutState { team_id, .. }) => { 
                // TODO: Think about a format for passing arguments to process functions like this.
                process_switch_out(
                    &mut app.state,
                    match team_id {
                        TeamID::Allies => &mut app.ally_ui_state,
                        TeamID::Opponents => &mut app.opponent_ui_state,
                    },
                    &mut app.message_log_ui_state,
                    &mut terminal,
                    &mut battle_sim.battle, 
                    &receiver
                )? 
            }, 
            AppState::Exiting => { break 'app; }
        }
        
        render_interface(&mut terminal, &mut app, &battle_sim.battle.message_buffer)?;
    }

    println!("monsim_tui exited successfully");
    Ok(NOTHING)
}

fn process_switch_out(
    app_state: &mut AppState,
    team_ui_state: &mut TeamUiState,
    message_log_ui_state:&mut MessageLogUiState,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    battle: &mut Battle,
    receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
) -> AppResult<Nothing> {

    let switch_out_state = match app_state {
        AppState::PromptSwitchOut(switch_out_state) => switch_out_state,
        _ => unreachable!("App is checked to be in the PromptSwitchOut state before calling this function.")
    };

    use KeyEventKind::Release;
    match receiver.recv()? {
        TuiEvent::Input(event) => {
            match (event.code, event.kind) {
                (KeyCode::Up, Release) => {
                    match switch_out_state.list_state.selected() {
                        Some(index) => {
                            let number_of_choices = switch_out_state.list_of_choices.len();
                            let new_index = (index + number_of_choices - 1) % number_of_choices;
                            switch_out_state.list_state.select(Some(new_index));
                        },
                        None => {
                            switch_out_state.list_state.select(Some(0));
                        },
                    }
                },
                (KeyCode::Down, Release) => {
                    match switch_out_state.list_state.selected() {
                        Some(index) => {
                            let number_of_choices = switch_out_state.list_of_choices.len();
                            let new_index = (index + 1) % number_of_choices;
                            switch_out_state.list_state.select(Some(new_index));
                        },
                        None => {
                            switch_out_state.list_state.select(Some(0));
                        },
                    }
                },
                (KeyCode::Enter, Release) => {
                    match switch_out_state.list_state.selected() {
                        Some(index) => {
                            let benched_battler_uid = Some(switch_out_state.list_of_choices[index].0);
                            team_ui_state.selected_action = Some((team_ui_state.list_items.len() - 1, ActionChoice::SwitchOut { 
                                active_battler_uid: switch_out_state.switching_battler, 
                                benched_battler_uid,
                            }));
                        },
                        None => {
                            battle.push_messages(
                                &[
                                    &"Simulator: Switch partner was not chosen... please select a battler to switch to before activating the simulation.",
                                    &"---",
                                    &EMPTY_LINE
                                ]
                            );
                            App::snap_message_log_scroll_index_to_turn_end(message_log_ui_state, battle);
                        },
                    }
                    *app_state = AppState::Processing(ProcessingState::AwaitingUserInput(battle.available_actions()));
                }
                (KeyCode::Esc, Release) =>  {
                    terminate(terminal, app_state)?; 
                    return Ok(NOTHING);
                }
                _ => NOTHING
            }
        },
        TuiEvent::Tick => NOTHING,
    }
    Ok(NOTHING)
}

fn new_list_state() -> ListState {
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    list_state
}

fn create_io_thread(sender: mpsc::Sender<TuiEvent<KeyEvent>>) {
    let time_out_duration = TUI_INPUT_POLL_TIMEOUT_MILLISECONDS;
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = time_out_duration.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));

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

fn process_awaiting_input_state(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: &mut App,
    battle_sim: &mut BattleSimulator,
    receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
    available_actions: AvailableActions,
) -> AppResult<Nothing> {
    match receiver.recv()? {
        TuiEvent::Input(event) => {
            use KeyEventKind::Release;
            use ProcessingState::Simulating;
            
            let was_escape_key_released = (KeyCode::Esc, Release) == (event.code, event.kind);
            if was_escape_key_released { 
                terminate(terminal, &mut app.state)?; 
                return Ok(NOTHING); 
            }
            
            let is_battle_finished = battle_sim.sim_state == SimState::BattleFinished;
            if not!(is_battle_finished) {
                match (event.code, event.kind) {
                    (KeyCode::Up, Release) => {
                        match app.currently_selected_widget {
                            SelectableWidget::MessageLog => { 
                                app.scroll_message_log_up();
                            },
                            SelectableWidget::AllyChoices => { app.ally_ui_state.scroll_selection_up() },
                            SelectableWidget::OpponentChoices => { app.opponent_ui_state.scroll_selection_up(); },
                            SelectableWidget::AllyRoster => { /* does nothing for now */ },
                            SelectableWidget::OpponentRoster => { /* does nothing for now */},
                        }
                    },
                    (KeyCode::Down, Release) => {
                        match app.currently_selected_widget {
                            SelectableWidget::MessageLog => { 
                                let message_log_length = battle_sim.battle.message_buffer.len(); 
                                app.scroll_message_log_down(message_log_length);
                            },
                            SelectableWidget::AllyChoices => { app.ally_ui_state.scroll_selection_down() },
                            SelectableWidget::OpponentChoices => { app.opponent_ui_state.scroll_selection_down(); },
                            SelectableWidget::AllyRoster => { /* does nothing for now */ },
                            SelectableWidget::OpponentRoster => { /* does nothing for now */},
                        }
                    },
                    (KeyCode::Left, Release) => {
                        app.currently_selected_widget.shift_left()
                    }
                    (KeyCode::Right, Release) => {
                        app.currently_selected_widget.shift_right()
                    },
                    (KeyCode::Enter, Release) => {
                        match app.currently_selected_widget {
                            SelectableWidget::AllyChoices => {
                                
                                process_choice_selection(
                                    available_actions.ally_team_available_actions,
                                    &mut app.ally_ui_state, 
                                    battle_sim,
                                    &mut app.state,
                                );
                            },
                            SelectableWidget::MessageLog => NOTHING,
                            SelectableWidget::OpponentChoices => {
                                
                                process_choice_selection(
                                    available_actions.opponent_team_available_actions,
                                    &mut app.opponent_ui_state, 
                                    battle_sim,
                                    &mut app.state,
                                );
                            },
                            SelectableWidget::AllyRoster => todo!(),
                            SelectableWidget::OpponentRoster => todo!(),
                        }
                    }
                    (KeyCode::Tab, Release) => { 
                        if let (Some(ally_selected_action), Some(opponent_selected_action)) =
                            (app.ally_ui_state.selected_action, app.opponent_ui_state.selected_action)
                        {
                            let chosen_actions = [
                                ally_selected_action,
                                opponent_selected_action,
                            ];
                            app.state = AppState::Processing(Simulating(chosen_actions));
                        } else {
                            battle_sim.battle.push_messages(
                                &[
                                    &"Simulator: Actions were not chosen... please select something before activating the simulation.",
                                    &"---",
                                    &EMPTY_LINE
                                ]
                            );
                            App::snap_message_log_scroll_index_to_turn_end(&mut app.message_log_ui_state, &mut battle_sim.battle);                  
                        }
                    },
                    _ => NOTHING,
                }
            } else { // Battle is finished
                match (event.code, event.kind) {
                    (KeyCode::Up, Release) => { app.scroll_message_log_up(); },
                    (KeyCode::Down, Release) => { 
                        let message_log_length = battle_sim.battle.message_buffer.len(); 
                        app.scroll_message_log_down(message_log_length); 
                    },
                    _ => NOTHING,
                }	
            }
        },
        TuiEvent::Tick => NOTHING,
    }
    Ok(NOTHING)
}

fn process_choice_selection(team_available_actions: TeamAvailableActions, team_ui_state: &mut TeamUiState, battle_sim: &mut BattleSimulator, app_state: &mut AppState) {
    let switch_action_index = team_available_actions.switch_out_action_index();
    let switch_selected = team_ui_state.list_state.selected() == switch_action_index;
    let team_id = team_ui_state.team_id;
                                
    let list_of_choices = battle_sim.battle.switch_partners_on_team(team_id)
        .iter()
        .map(|battler| {
            (battler.uid, battler.monster.nickname, battler.monster.species.name)
        })
        .collect::<Vec<_>>();

    let switch_out_state = SwitchOutState {
        switching_battler: battle_sim.battle.active_battlers_on_team(team_id).0.uid,
        team_id, 
        list_of_choices, 
        list_state: new_list_state(),
    };

    if switch_selected {
        *app_state = AppState::PromptSwitchOut(switch_out_state);
    } else {
        App::confirm_selection(team_ui_state, team_available_actions);
    }
}

fn terminate(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app_state: &mut AppState) -> AppResult<Nothing> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    #[cfg(feature="debug")]
    remove_debug_log_file()?;
    *app_state = AppState::Exiting;
    Ok(NOTHING)
}

#[cfg(feature="debug")]
fn remove_debug_log_file() -> Result<Nothing, BoxedError> {
    #[cfg(feature = "debug")]
    if let Err(e) = std::fs::remove_file("debug_output.txt") {
        if std::io::ErrorKind::NotFound != e.kind() {
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        } else {
            Ok(NOTHING)
        }
    } else {
        Ok(NOTHING)
    }
}

fn render_interface<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>, 
    app: &mut App,
    message_buffer: &MessageBuffer,
) -> std::io::Result<CompletedFrame<'a>> {

    terminal.draw(|frame| {
        let chunks = divide_screen_into_chunks(frame);
        
        let ally_panel_chunks = divide_panel_into_chunks(chunks[0]);
        let ally_stats_widget = construct_stats_panel_widget(&app.ally_ui_state); 
        let ally_choice_menu_widget = construct_choice_menu_widget(&app.ally_ui_state, app.currently_selected_widget);
        let ally_team_roster_widget = construct_roster_widget(&app.ally_ui_state, app.currently_selected_widget);

        let message_log_widget = construct_message_log_widget(message_buffer, &mut app.message_log_ui_state, app.currently_selected_widget);
        
        let opponent_panel_chunks = divide_panel_into_chunks(chunks[2]);
        let opponent_stats_widget = construct_stats_panel_widget(&app.opponent_ui_state); 
        let opponent_choice_menu_widget = construct_choice_menu_widget(&app.opponent_ui_state, app.currently_selected_widget);
        let opponent_team_roster_widget = construct_roster_widget(&app.opponent_ui_state, app.currently_selected_widget);

        if let AppState::PromptSwitchOut(ref mut switch_out_state) = &mut app.state {
            
            let message_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(80),
                    Constraint::Max(5),
                ]
                .as_ref(),
            )
            .split(chunks[1]);

            let list_of_choices = switch_out_state.list_of_choices
                .clone()
                .iter()
                .map(|(_, benched_teammate_nickname, benched_teammate_species_name)| { ListItem::new(format!["{} the {}", *benched_teammate_nickname, benched_teammate_species_name])})
                .collect::<Vec<_>>();

            let switch_out_widget = List::new(list_of_choices)
                .block(
                    Block::default()
                        .title(Span::styled(" Switch out with? ", Style::default().fg(Color::Yellow)))
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            frame.render_widget(message_log_widget, message_chunks[0]);
            frame.render_stateful_widget(switch_out_widget, message_chunks[1], &mut switch_out_state.list_state);
        } else {
            frame.render_widget(message_log_widget, chunks[1]);
        }

        frame.render_widget(ally_stats_widget, ally_panel_chunks[0]);
        let mut ally_list_state = app.ally_ui_state.list_state.clone();
        frame.render_stateful_widget(ally_choice_menu_widget, ally_panel_chunks[1], &mut ally_list_state); 
        frame.render_widget(ally_team_roster_widget, ally_panel_chunks[2]);
        
        
        frame.render_widget(opponent_stats_widget, opponent_panel_chunks[0]);
        let mut opponent_list_state = app.opponent_ui_state.list_state.clone();
        frame.render_stateful_widget(opponent_choice_menu_widget, opponent_panel_chunks[1], &mut opponent_list_state);
        frame.render_widget(opponent_team_roster_widget, opponent_panel_chunks[2]);
    })
}

fn divide_screen_into_chunks(frame: &mut Frame<CrosstermBackend<Stdout>>) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Min(25),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(frame.size())
}

fn divide_panel_into_chunks(chunk: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(33), Constraint::Length(8), Constraint::Length(6)].as_ref())
        .split(chunk)
}

fn construct_stats_panel_widget<'a>(team_ui_state: &'a TeamUiState) -> Paragraph<'a> {
    let team_name = team_ui_state.team_id;
    Paragraph::new(team_ui_state.active_battler_status.as_str())
        .block(Block::default().title(format![" {team_name} Active Monster "]).borders(Borders::ALL))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
}

fn construct_roster_widget<'a>(team_ui_state: &'a TeamUiState, currently_selected_list: SelectableWidget) -> Paragraph<'a> {
    let team_name = team_ui_state.team_id;
    let team_roster_list = match team_name {
        TeamID::Allies => SelectableWidget::AllyRoster,
        TeamID::Opponents => SelectableWidget::OpponentRoster,
    };
    Paragraph::new(team_ui_state.team_roster_status.as_str())
            .block(
                Block::default()
                    .title({
                        if currently_selected_list == team_roster_list {
                            Span::styled(format![" {team_name} Team Status "], Style::default().fg(Color::Yellow))
                        } else {
                            Span::raw(format![" {team_name} Team Status "])
                        }
                    })
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
}

fn construct_choice_menu_widget<'a>(team_ui_state: &'a TeamUiState, currently_selected_list: SelectableWidget) -> List<'a> {
    let team_name = team_ui_state.team_id;
    let team_choices_list = match team_name {
        TeamID::Allies => SelectableWidget::AllyChoices,
        TeamID::Opponents => SelectableWidget::OpponentChoices,
    };
    let currently_selected_choice = team_ui_state.selected_action.map(|it| { it.0 });
    let list_items = team_ui_state.list_items.iter().enumerate().map(|(i, list_item)| {
        if let Some(currently_selected_choice) = currently_selected_choice {
            if i == currently_selected_choice {
                list_item.clone().style(Style::default().fg(Color::Green))
            } else {
                list_item.clone()
            }
        } else {
            list_item.clone()
        }
    }).collect::<Vec<_>>();
    List::new(list_items)
        .block(
            Block::default()
                .title(if currently_selected_list == team_choices_list {
                    Span::styled(format![" {team_name} Monster Choices "], Style::default().fg(Color::Yellow))
                } else {
                    Span::raw(format![" {team_name} Monster Choices "])
                })
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
}

/// `message_log_scroll_index` is the index of  the first line of the message buffer to be rendered.
fn construct_message_log_widget<'a>(message_buffer: &'a MessageBuffer, message_log_ui_state:&'a mut MessageLogUiState, currently_selected_list: SelectableWidget) -> Paragraph<'a> {
    let text = message_buffer
        .iter()
        .enumerate()
        .filter_map(|(idx, element)| {
            if idx >= message_log_ui_state.message_log_scroll_idx { 
                if element.contains("Turn") {
                    Some(Spans::from(Span::styled(element, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))))
                } else {
                    Some(Spans::from(Span::raw(element)))
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(text)
        .block(
            Block::default()
                .title(if currently_selected_list == SelectableWidget::MessageLog {
                    Span::styled(" Message Log ", Style::default().fg(Color::Yellow))
                } else {
                    Span::raw(" Message Log ")
                })
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

impl<'a> App<'a> {
    pub fn new(battle: &mut Battle) -> App<'a> {
        let AvailableActions { ally_team_available_actions, opponent_team_available_actions } = battle.available_actions();
        App {
            state: AppState::Initialising,
            currently_selected_widget: SelectableWidget::MessageLog,
            message_log_ui_state: MessageLogUiState::new(),
            ally_ui_state: TeamUiState::new(
                battle,
                TeamID::Allies,
                ally_team_available_actions
            ),
            opponent_ui_state: TeamUiState::new(
                battle,
                TeamID::Opponents,
                opponent_team_available_actions
            ),
        }
    } 

    fn scroll_message_log_up(&mut self) {
        self.message_log_ui_state.message_log_scroll_idx = self.message_log_ui_state.message_log_scroll_idx.saturating_sub(1);
        self.message_log_ui_state.message_log_scroll_idx = self.message_log_ui_state.message_log_scroll_idx.min(self.message_log_ui_state.message_log_last_scrollable_line_idx);
    }

    fn scroll_message_log_down(&mut self, message_log_length: usize) {
        self.message_log_ui_state.message_log_scroll_idx = (self.message_log_ui_state.message_log_scroll_idx + 1)
            .min(message_log_length);
        self.message_log_ui_state.message_log_scroll_idx = self.message_log_ui_state.message_log_scroll_idx.min(self.message_log_ui_state.message_log_last_scrollable_line_idx);
    }

    fn regenerate_ui_data(&mut self, battle: &mut Battle, available_actions: AvailableActions) {
        
        let AvailableActions {
            ally_team_available_actions,
            opponent_team_available_actions,
        } = available_actions;

        App::snap_message_log_scroll_index_to_turn_end(&mut self.message_log_ui_state, battle);
        
        let ally_active_battler = battle.active_battlers_on_team(TeamID::Allies).0;
        
        self.ally_ui_state = TeamUiState {
            active_battler_status: BattlerTeam::battler_status_as_string(ally_active_battler),
            team_roster_status: battle.ally_team.to_string(),
            ..self.ally_ui_state.clone()
        };

        TeamUiState::regenerate_list(&battle.ally_team, &mut self.ally_ui_state.list_items, ally_team_available_actions);
        
        let opponent_active_battler = battle.active_battlers_on_team(TeamID::Opponents).0;
        
        self.opponent_ui_state = TeamUiState {
            active_battler_status: BattlerTeam::battler_status_as_string(opponent_active_battler),
            team_roster_status: battle.opponent_team.to_string(),
            ..self.opponent_ui_state.clone()
        };
        
        TeamUiState::regenerate_list(&battle.opponent_team, &mut self.opponent_ui_state.list_items, opponent_team_available_actions);
    }
    
    fn snap_message_log_scroll_index_to_turn_end(message_log_ui_state: &mut MessageLogUiState, battle: &mut Battle) {
        message_log_ui_state.message_log_scroll_idx = message_log_ui_state.last_message_buffer_length;
        message_log_ui_state.message_log_last_scrollable_line_idx = message_log_ui_state.message_log_scroll_idx;
        message_log_ui_state.last_message_buffer_length = battle.message_buffer.len();
    }

    fn confirm_selection(team_ui_state: &mut TeamUiState, team_available_actions: TeamAvailableActions) {
        if let Some(selected_index) = team_ui_state.list_state.selected() {
            team_ui_state.selected_action = team_available_actions[selected_index]; 
        }
    }
}

impl MessageLogUiState {
    fn new() -> Self {
        Self {
            message_log_scroll_idx: 0,
            message_log_last_scrollable_line_idx: 0,
            last_message_buffer_length: 0,
        }
    }
}

impl<'a> TeamUiState<'a> {
    fn new(battle: &mut Battle, team_id: TeamID, available_actions: TeamAvailableActions) -> TeamUiState<'a> {
        let (team, team_active_battler) = match team_id {
            TeamID::Allies => {
                (&battle.ally_team, battle.active_battlers_on_team(TeamID::Allies).0)
            },
            TeamID::Opponents => {
                (&battle.opponent_team, battle.active_battlers_on_team(TeamID::Opponents).0)
            },
        };
        
        let mut list_items = Vec::with_capacity(5);
        Self::regenerate_list(team, &mut list_items, available_actions);
        TeamUiState {
            team_id,
            active_battler_status: BattlerTeam::battler_status_as_string(team_active_battler),
            team_roster_status: team.to_string(),
            list_items,
            list_state: new_list_state(),
            selected_action: None,
        }
    }

    fn scroll_selection_up(&mut self) {
        let selected_index = self.list_state.selected().expect("We are not supposed to set this to None.");
            let new_index = (selected_index + self.len() - 1) % self.len();
            self.list_state.select(Some(new_index));
    }
    
    fn scroll_selection_down(&mut self) {
        let selected_index = self.list_state.selected().expect("We are not supposed to set this to None.");
        let new_index = (selected_index + 1) % self.len();
        self.list_state.select(Some(new_index));
    }
    
    #[inline(always)]
    fn len(&self) -> usize {
        self.list_items.len()
    }

    fn regenerate_list(team: &BattlerTeam, list_items: &mut Vec<ListItem>, available_actions: TeamAvailableActions) {
        list_items.clear();
        for choice in available_actions.into_iter() {
            match choice {
                (_, ActionChoice::Move { move_uid, target_uid: _ }) => {
                    for battler in team.battlers() {
                        if battler.uid == move_uid.battler_uid {
                            let move_ = battler.moveset[move_uid.move_number as usize];
                            list_items.push(ListItem::new(move_.species.name));
                        }
                    }
                },
                (_, ActionChoice::SwitchOut { active_battler_uid: _, benched_battler_uid: _ }) => {
                    list_items.push(ListItem::new("Switch Out"));
                },
            }
        }
    }
}

impl SelectableWidget {
    pub fn shift_right(&mut self) {
        match self {
            SelectableWidget::AllyChoices => { *self = SelectableWidget::MessageLog },
            SelectableWidget::MessageLog => { *self = SelectableWidget::OpponentChoices },
            SelectableWidget::OpponentChoices => { *self = SelectableWidget::AllyChoices },
            _ => NOTHING, // Shifting right does nothing.
        }
    }

    pub fn shift_left(&mut self) {
        match self {
            SelectableWidget::AllyChoices => { *self = SelectableWidget::OpponentChoices },
            SelectableWidget::MessageLog => { *self = SelectableWidget::AllyChoices },
            SelectableWidget::OpponentChoices => { *self = SelectableWidget::MessageLog },
            _ => NOTHING, // Shifting right does nothing.
        }
    }
}
