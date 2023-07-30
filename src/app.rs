use std::{sync::mpsc, time::{Duration, Instant}, thread, io::Stdout};

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode}, event::{self, Event, KeyCode, KeyEventKind, KeyEvent}, execute};
use tui::{backend::CrosstermBackend, Terminal, terminal::CompletedFrame, widgets::{ListState, ListItem, Paragraph, Block, Borders, Wrap, List}, layout::{Layout, Direction, Constraint, Rect, Alignment}, Frame, text::{Span, Spans}, style::{Style, Color, Modifier}};

use crate::sim::{BattleSimulator, Battle, ChosenActionsForTurn, EMPTY_LINE, AvailableActions, ChosenAction, BattlerTeam, MessageBuffer, TeamID, TeamAvailableActions, utils::{NOTHING, Nothing}, BattlerUID, ChoosableAction, EnumeratedChosenAction};

mod render;
use render::render_interface;

pub type AppResult<T> = Result<T, BoxedError>;
type BoxedError = Box<dyn std::error::Error>;

/// `main` function for app.rs
pub fn run(mut battle: Battle) -> AppResult<Nothing> {
    
    let mut ui = Ui::new(&mut battle);
    let mut simulator = BattleSimulator::new(battle);
    let mut terminal = create_configured_terminal()?;
    let (sender, receiver) = mpsc::channel();
    create_io_thread(sender);
    
    let available_actions = simulator.generate_available_actions();
    let mut app_state = AppState::Processing(ProcessingState::FreeInput(available_actions));
    
    // Render interface once before the update loop starts.
    render_interface(&mut ui, &mut app_state, &mut terminal, &simulator.battle.message_buffer)?;
    'update_and_render: loop {
        match app_state {
            AppState::Processing(ref processing_state) => {
                match processing_state.clone() {
                    ProcessingState::FreeInput(available_actions) => {
                        if let Some(key) = get_key_released(&receiver)? {
                            ui.update_from_free_input(&mut simulator.battle, &available_actions, key).map_or(NOTHING, |new_state| {
                                app_state = new_state;
                            });
                        };
                    },
                    ProcessingState::Simulation(chosen_actions) => {
                        match simulator.simulate_turn(chosen_actions) {
                            Ok(_) => simulator.battle.push_message(&"Simulator: The turn was calculated successfully."),
                            Err(error) => simulator.battle.push_message(&format!["Simulator: {:?}", error]),
                        };
                        let available_actions = simulator.generate_available_actions();
                        ui.refresh(&mut simulator.battle, available_actions);
    
                        app_state = AppState::Processing(ProcessingState::FreeInput(available_actions))
                    },
                    ProcessingState::BattleFinished => {
                        if let Some(key) = get_key_released(&receiver)? {
                            ui.update_from_post_battle_input(&mut simulator.battle, key).map_or(NOTHING, |new_state| {
                                app_state = new_state;
                            });
                        }
                    }
                }
            }
            AppState::PromptSwitchOut(ref mut switch_out_state) => { 
                if let Some(key) = get_key_released(&receiver)? {
                    let team_id = switch_out_state.team_id;
                    ui.update_switch_out_state(switch_out_state, &mut simulator.battle, team_id, key)?.map_or(NOTHING, |new_state| {
                        app_state = new_state;
                    }); 
                }
            }, 
            AppState::Terminating => { 
                terminate(&mut terminal)?;
                break 'update_and_render; 
            }
        }
        render_interface(&mut ui, &mut app_state, &mut terminal, &simulator.battle.message_buffer)?;
    }

    println!("monsim_tui exited successfully");
    Ok(NOTHING)
}

fn get_key_released(receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>) -> AppResult<Option<KeyCode>> {
    Ok(match receiver.recv()? {
        TuiEvent::Input(key_event) => {
            if key_event.kind == KeyEventKind::Release { Some(key_event.code) } else { None }
        },
        TuiEvent::Tick => None,
    })
}

#[derive(Debug, Clone)]
pub struct Ui<'a> {
    currently_selected_widget: SelectableWidget,
    message_log_ui_state: MessageLogUiState,
    ally_panel_ui_state: TeamUiState<'a>,
    opponent_panel_ui_state: TeamUiState<'a>,
}

#[derive(Debug, Clone)]
enum AppState<'a> {
    Processing(ProcessingState),
    PromptSwitchOut(SwitchOutState<'a>),
    Terminating,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessingState {
    FreeInput(AvailableActions),
    Simulation(ChosenActionsForTurn),
    BattleFinished,
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
    selected_action: Option<EnumeratedChosenAction>,
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

fn new_list_state() -> ListState {
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    list_state
}

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

fn create_configured_terminal() -> AppResult<TuiTerminal> {
    // Raw mode allows us to not require enter presses to get input
    enable_raw_mode().expect("Enabling raw mode should always work.");

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    Ok(terminal)
}

fn create_io_thread(sender: mpsc::Sender<TuiEvent<KeyEvent>>) {
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

fn terminate(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> AppResult<Nothing> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    #[cfg(feature="debug")]
    remove_debug_log_file()?;
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

impl<'a> Ui<'a> {
    pub fn new(battle: &mut Battle) -> Ui<'a> {
        let AvailableActions { ally_team_available_actions, opponent_team_available_actions } = battle.available_actions();
        Ui {
            currently_selected_widget: SelectableWidget::MessageLog,
            message_log_ui_state: MessageLogUiState::new(),
            ally_panel_ui_state: TeamUiState::new(
                battle,
                TeamID::Allies,
                ally_team_available_actions
            ),
            opponent_panel_ui_state: TeamUiState::new(
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

    fn refresh(&mut self, battle: &mut Battle, available_actions: AvailableActions) {
        
        let AvailableActions {
            ally_team_available_actions,
            opponent_team_available_actions,
        } = available_actions;

        Ui::snap_message_log_scroll_index_to_turn_end(&mut self.message_log_ui_state, battle);
        
        let ally_active_battler = battle.active_battlers_on_team(TeamID::Allies).0;
        
        self.ally_panel_ui_state = TeamUiState {
            active_battler_status: BattlerTeam::battler_status_as_string(ally_active_battler),
            team_roster_status: battle.ally_team.to_string(),
            ..self.ally_panel_ui_state.clone()
        };

        TeamUiState::regenerate_list(&battle.ally_team, &mut self.ally_panel_ui_state.list_items, ally_team_available_actions);
        
        let opponent_active_battler = battle.active_battlers_on_team(TeamID::Opponents).0;
        
        self.opponent_panel_ui_state = TeamUiState {
            active_battler_status: BattlerTeam::battler_status_as_string(opponent_active_battler),
            team_roster_status: battle.opponent_team.to_string(),
            ..self.opponent_panel_ui_state.clone()
        };
        
        TeamUiState::regenerate_list(&battle.opponent_team, &mut self.opponent_panel_ui_state.list_items, opponent_team_available_actions);
    }
    
    fn snap_message_log_scroll_index_to_turn_end(message_log_ui_state: &mut MessageLogUiState, battle: &mut Battle) {
        message_log_ui_state.message_log_scroll_idx = message_log_ui_state.last_message_buffer_length;
        message_log_ui_state.message_log_last_scrollable_line_idx = message_log_ui_state.message_log_scroll_idx;
        message_log_ui_state.last_message_buffer_length = battle.message_buffer.len();
    }

    #[must_use]
    fn update_switch_out_state(
        &mut self,
        switch_out_state: &mut SwitchOutState,
        battle: &mut Battle,
        team_id: TeamID,
        input_key: KeyCode,
    ) -> AppResult<Option<AppState<'a>>> {

        let team_ui_state = match team_id {
            TeamID::Allies => &mut self.ally_panel_ui_state,
            TeamID::Opponents => &mut self.opponent_panel_ui_state,
        };

        let message_log_ui_state = &mut self.message_log_ui_state;

        match input_key {
            KeyCode::Up => {
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
            KeyCode::Down => {
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
            KeyCode::Enter => {
                match switch_out_state.list_state.selected() {
                    Some(index) => {
                        let benched_battler_uid = switch_out_state.list_of_choices[index].0;
                        team_ui_state.selected_action = Some((team_ui_state.list_items.len() - 1, ChosenAction::SwitchOut { 
                            switcher_uid: switch_out_state.switching_battler, 
                            switchee_uid: benched_battler_uid,
                        }));
                    },
                    None => {
                        battle.push_messages(
                            &[
                                &"Simulator: Switchee was not chosen. Please select a battler to switch to before activating the simulation.",
                                &"---",
                                &EMPTY_LINE
                            ]
                        );
                        Ui::snap_message_log_scroll_index_to_turn_end(message_log_ui_state, battle);
                    },
                }
                return Ok(Some(AppState::Processing(ProcessingState::FreeInput(battle.available_actions()))));
            }
            KeyCode::Esc =>  { return Ok(Some(AppState::Terminating)) },
            _ => NOTHING
        }
        Ok(None)
    }

    #[must_use]
    fn update_from_free_input(
        &mut self, 
        battle: &mut Battle, 
        available_actions: &AvailableActions, 
        input_key: KeyCode
    ) -> Option<AppState<'a>> {
        
        if input_key == KeyCode::Esc {
            return Some(AppState::Terminating);
        }

        match input_key {
            KeyCode::Up => {
                match self.currently_selected_widget {
                    SelectableWidget::MessageLog => { 
                        self.scroll_message_log_up();
                    },
                    SelectableWidget::AllyChoices => { self.ally_panel_ui_state.scroll_selection_up() },
                    SelectableWidget::OpponentChoices => { self.opponent_panel_ui_state.scroll_selection_up(); },
                    SelectableWidget::AllyRoster => unreachable!(),
                    SelectableWidget::OpponentRoster => unreachable!(),
                }
            },
            KeyCode::Down => {
                match self.currently_selected_widget {
                    SelectableWidget::MessageLog => { 
                        let message_log_length = battle.message_buffer.len(); 
                        self.scroll_message_log_down(message_log_length);
                    },
                    SelectableWidget::AllyChoices => { self.ally_panel_ui_state.scroll_selection_down() },
                    SelectableWidget::OpponentChoices => { self.opponent_panel_ui_state.scroll_selection_down(); },
                    SelectableWidget::AllyRoster => unreachable!(),
                    SelectableWidget::OpponentRoster => unreachable!(),
                }
            },
            KeyCode::Left => {
                self.currently_selected_widget.shift_left()
            }
            KeyCode::Right => {
                self.currently_selected_widget.shift_right()
            },
            KeyCode::Enter => {
                match self.currently_selected_widget {
                    SelectableWidget::AllyChoices => { 
                        Ui::update_ui_selection_state(
                            &mut self.ally_panel_ui_state, 
                            battle,
                            available_actions.ally_team_available_actions,
                        );
                    },
                    SelectableWidget::MessageLog => NOTHING,
                    SelectableWidget::OpponentChoices => {
                        Ui::update_ui_selection_state(
                            &mut self.opponent_panel_ui_state, 
                            battle,
                            available_actions.opponent_team_available_actions,
                        );
                    },
                    SelectableWidget::AllyRoster => todo!(),
                    SelectableWidget::OpponentRoster => todo!(),
                }
            }
            KeyCode::Tab => { 
                if let (Some(ally_selected_action), Some(opponent_selected_action)) =
                    (self.ally_panel_ui_state.selected_action, self.opponent_panel_ui_state.selected_action)
                {
                    let chosen_actions = [
                        ally_selected_action,
                        opponent_selected_action,
                    ];
                    return Some(AppState::Processing(ProcessingState::Simulation(chosen_actions)));
                } else {
                    battle.push_messages(
                        &[
                            &"Simulator: Actions were not chosen... please select something before activating the simulation.",
                            &"---",
                            &EMPTY_LINE
                        ]
                    );
                    Ui::snap_message_log_scroll_index_to_turn_end(&mut self.message_log_ui_state, battle);                  
                }
            },
            _ => NOTHING,
        }

        None
    }

    fn update_ui_selection_state(
        team_ui_state: &mut TeamUiState, 
        battle: &mut Battle, 
        team_available_actions: TeamAvailableActions, 
    ) -> Option<AppState<'a>> {
        let switch_action_index = team_available_actions.switch_out_action_index();
        let switch_selected = team_ui_state.list_state.selected() == switch_action_index;
        let team_id = team_ui_state.team_id;
                                    
        let list_of_choices = battle.switch_partners_on_team(team_id)
            .iter()
            .map(|battler| {
                (battler.uid, battler.monster.nickname, battler.monster.species.name)
            })
            .collect::<Vec<_>>();

        let switch_out_state = SwitchOutState {
            switching_battler: battle.active_battlers_on_team(team_id).0.uid,
            team_id, 
            list_of_choices, 
            list_state: new_list_state(),
        };

        if switch_selected {
            Some(AppState::PromptSwitchOut(switch_out_state))
        } else {
            if let Some(selected_index) = team_ui_state.list_state.selected() {
                team_ui_state.selected_action = team_available_actions[selected_index].map( |(idx, choosable_action)| {
                    match choosable_action {
                        ChoosableAction::Move(move_uid) => (idx, ChosenAction::Move { move_uid, target_uid: todo!() }),
                        ChoosableAction::SwitchOut { switcher_uid } => (idx, ChosenAction::SwitchOut { switcher_uid, switchee_uid: todo!() }),
                    }
                }); 
            };
            None
        }
    }

    #[must_use]
    fn update_from_post_battle_input(
        &mut self, 
        battle: &mut Battle,
        input_key: KeyCode
    ) -> Option<AppState<'a>> {
        
        match input_key {
            KeyCode::Up => { self.scroll_message_log_up(); },
            KeyCode::Down => { 
                let message_log_length = battle.message_buffer.len(); 
                self.scroll_message_log_down(message_log_length); 
            },
            _ => NOTHING,
        }
        
        None	
    }
}

impl<'a> PartialEq for AppState<'a> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
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
                (_, ChoosableAction::Move(move_uid)) => {
                    for battler in team.battlers() {
                        if battler.uid == move_uid.battler_uid {
                            let move_ = battler.moveset[move_uid.move_number as usize];
                            list_items.push(ListItem::new(move_.species.name));
                        }
                    }
                },
                (_, ChoosableAction::SwitchOut { switcher_uid: _ }) => {
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
