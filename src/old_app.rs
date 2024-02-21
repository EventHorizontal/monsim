use std::{sync::mpsc, time::{Duration, Instant}, thread, io::Stdout};

use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use tui::{backend::CrosstermBackend, Terminal, terminal::CompletedFrame, widgets::{ListState, ListItem, Paragraph, Block, Borders, Wrap, List}, layout::{Layout, Direction, Constraint, Rect, Alignment}, Frame, text::{Span, Spans}, style::{Style, Color, Modifier}};

use crate::sim::{utils::{Nothing, NOTHING}, AvailableActions, Battle, BattleSimulator, BattlerUID, PartialActionChoice, ActionChoice, ChosenActionsForTurn, MessageLog, AvailableActionsByTeam, TeamID, EMPTY_LINE};

mod render;
use render::render_interface;

pub type AppResult<T> = Result<T, BoxedError>;
type BoxedError = Box<dyn std::error::Error>;

/// `main` function for our application
pub fn run(mut battle: Battle) -> AppResult<Nothing> {
    
    let mut ui = Ui::new(&mut battle);
    let mut simulator = BattleSimulator::new(battle);
    let mut terminal = create_configured_terminal()?;
    let (sender, receiver) = mpsc::channel();
    create_io_thread(sender);
    
    let available_actions = simulator.available_actions();
    // TODO: I guess we might think about have a "renderables" to export to the UI.
    ui.update(
        available_actions, 
        simulator.battle.active_battlers_by_team(TeamID::Allies)
            .status_string(), 
        simulator.battle.active_battlers_by_team(TeamID::Opponents)
            .status_string(),
        simulator.battle.ally_team().to_string(),
        simulator.battle.opponent_team().to_string(),
        simulator.battle.message_buffer.len(),
    );
    let mut app_state = AppState::Processing(ProcessingState::ProcessingMidBattleInput(available_actions));
    
    // Render interface once before the update loop starts.
    render_interface(&mut ui, &mut app_state, &mut terminal, &simulator.battle.message_buffer)?;
    'update_and_render: loop {
        match app_state {
            AppState::Processing(ref processing_state) => {
                match processing_state.clone() {
                    ProcessingState::ProcessingMidBattleInput(available_actions) => {
                        if let Some(key) = get_key_pressed(&receiver)? {
                            let message_log_length = simulator.battle.message_buffer.len();
                            let maybe_new_app_state = ui.mid_battle_update(&mut simulator.battle, &available_actions, key, message_log_length);
                            app_state.transition_to(maybe_new_app_state);
                        };
                    },
                    ProcessingState::Simulating(chosen_actions) => {
                        match simulator.simulate_turn(chosen_actions) {
                            Ok(_) => simulator.battle.push_message(&"Simulator: The turn was calculated successfully."),
                            Err(error) => simulator.battle.push_message(&format!["Simulator: {:?}", error]),
                        };
                        let available_actions = simulator.available_actions();
                        let message_buffer_length = simulator.battle.message_buffer.len();
                        ui.update(
                            available_actions, 
                            simulator.battle.active_battlers_by_team(TeamID::Allies)
                                .status_string(), 
                            simulator.battle.active_battlers_by_team(TeamID::Opponents)
                                .status_string(),
                            simulator.battle.ally_team().to_string(),
                            simulator.battle.opponent_team().to_string(),
                            message_buffer_length,
                        );
                        match simulator.sim_state {
                            crate::sim::SimState::BattleOngoing => {
                                app_state = AppState::Processing(ProcessingState::ProcessingMidBattleInput(available_actions));
                            },
                            crate::sim::SimState::BattleFinished => {
                                app_state = AppState::Processing(ProcessingState::ProcessingPostBattleInput);
                            },
                        }
                    },
                    ProcessingState::ProcessingPostBattleInput => {
                        if let Some(key) = get_key_pressed(&receiver)? {
                            let maybe_new_app_state = ui.post_battle_update(&mut simulator.battle, key);
                            app_state.transition_to(maybe_new_app_state);
                        }
                    }
                }
            }
            AppState::PromptSwitchOut(ref mut switch_out_state) => { 
                if let Some(key) = get_key_pressed(&receiver)? {
                    let team_id = switch_out_state.team_id;
                    let maybe_new_app_state = ui.update_switch_out_state(switch_out_state, &mut simulator.battle, team_id, key)?;
                    app_state.transition_to(maybe_new_app_state);
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

fn get_key_pressed(receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>) -> AppResult<Option<KeyCode>> {
    Ok(match receiver.recv()? {
        TuiEvent::Input(key_event) => {
            // Reminder: Linux does not have the Release and Repeat Flags enabled by default
            // As such I'm going to avoid using the extra flags, hopefully we don't get weird behaviour.
            if key_event.kind == KeyEventKind::Press { Some(key_event.code) } else { None }
        },
        TuiEvent::Tick => None,
    })
}

#[derive(Debug, Clone)]
pub struct Ui<'a> {
    currently_selected_widget: SelectableWidget,
    message_log_panel: MessageLogUiState,
    ally_ui_panel: TeamUiState<'a>,
    opponent_ui_panel: TeamUiState<'a>,
}

#[derive(Debug, Clone)]
enum AppState<'a> {
    Processing(ProcessingState),
    PromptSwitchOut(ProposedSwitch<'a>),
    Terminating,
}

impl<'a> AppState<'a> {
    fn transition_to(&mut self, maybe_new_app_state: Option<AppState<'a>>) {
        if let Some(new_app_state) = maybe_new_app_state { *self = new_app_state };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessingState {
    ProcessingMidBattleInput(AvailableActions),
    Simulating(ChosenActionsForTurn),
    ProcessingPostBattleInput,
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
    selected_item_index: Option<usize>,
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
pub struct ProposedSwitch<'a> {
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
        Ui {
            currently_selected_widget: SelectableWidget::MessageLog,
            message_log_panel: MessageLogUiState::new(),
            ally_ui_panel: TeamUiState::new(
                battle,
                TeamID::Allies,
            ),
            opponent_ui_panel: TeamUiState::new(
                battle,
                TeamID::Opponents,
            ),
        }
    } 

    fn scroll_message_log_up(&mut self) {
        self.message_log_panel.message_log_scroll_idx = self.message_log_panel.message_log_scroll_idx.saturating_sub(1);
        self.message_log_panel.message_log_scroll_idx = self.message_log_panel.message_log_scroll_idx.min(self.message_log_panel.message_log_last_scrollable_line_idx);
    }

    fn scroll_message_log_down(&mut self, message_log_length: usize) {
        self.message_log_panel.message_log_scroll_idx = (self.message_log_panel.message_log_scroll_idx + 1)
            .min(message_log_length);
        self.message_log_panel.message_log_scroll_idx = self.message_log_panel.message_log_scroll_idx.min(self.message_log_panel.message_log_last_scrollable_line_idx);
    }

    fn update(&mut self, 
        available_actions: AvailableActions,
        ally_active_battler: String,
        opponent_active_battler: String,
        ally_benched_battlers: String,
        opponent_benched_battlers: String,
        message_buffer_length: usize,
    ) {
        
        Ui::snap_message_log_scroll_index_to_turn_end(&mut self.message_log_panel, message_buffer_length);
        
        let AvailableActions {
            ally_team_available_actions,
            opponent_team_available_actions,
        } = available_actions;

        self.ally_ui_panel.update_list(ally_team_available_actions);
        self.opponent_ui_panel.update_list(opponent_team_available_actions);
        
        self.ally_ui_panel = TeamUiState {
            active_battler_status: ally_active_battler,
            team_roster_status: ally_benched_battlers,
            ..self.ally_ui_panel.clone()
        };
        
        self.opponent_ui_panel = TeamUiState {
            active_battler_status: opponent_active_battler,
            team_roster_status: opponent_benched_battlers,
            ..self.opponent_ui_panel.clone()
        };
        
    }
    
    fn snap_message_log_scroll_index_to_turn_end(message_log_ui_state: &mut MessageLogUiState, message_buffer_length: usize) {
        message_log_ui_state.message_log_scroll_idx = message_log_ui_state.last_message_buffer_length;
        message_log_ui_state.message_log_last_scrollable_line_idx = message_log_ui_state.message_log_scroll_idx;
        message_log_ui_state.last_message_buffer_length = message_buffer_length;
    }

    #[must_use]
    fn update_switch_out_state(
        &mut self,
        switch_out_state: &mut ProposedSwitch,
        battle: &mut Battle,
        team_id: TeamID,
        input_key: KeyCode,
    ) -> AppResult<Option<AppState<'a>>> {

        let team_ui_state = match team_id {
            TeamID::Allies => &mut self.ally_ui_panel,
            TeamID::Opponents => &mut self.opponent_ui_panel,
        };

        let message_log_ui_state = &mut self.message_log_panel;

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
                        team_ui_state.selected_item_index = Some(index);
                    },
                    None => {
                        battle.push_messages_to_log(
                            &[
                                &"Simulator: Switchee was not chosen. Please select a battler to switch to before activating the simulation.",
                                &"---",
                                &EMPTY_LINE
                            ]
                        );
                        Ui::snap_message_log_scroll_index_to_turn_end(message_log_ui_state, battle.message_log.len());
                    },
                }
                return Ok(Some(AppState::Processing(ProcessingState::ProcessingMidBattleInput(battle.available_actions()))));
            }
            KeyCode::Esc =>  { return Ok(Some(AppState::Terminating)) },
            _ => NOTHING
        }
        Ok(None)
    }

    #[must_use]
    fn mid_battle_update(
        &mut self, 
        battle: &mut Battle, 
        available_actions: &AvailableActions, 
        input_key: KeyCode,
        message_log_length: usize,
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
                    SelectableWidget::AllyChoices => { self.ally_ui_panel.scroll_selection_up() },
                    SelectableWidget::OpponentChoices => { self.opponent_ui_panel.scroll_selection_up(); },
                    SelectableWidget::AllyRoster => unreachable!(),
                    SelectableWidget::OpponentRoster => unreachable!(),
                }
            },
            KeyCode::Down => {
                match self.currently_selected_widget {
                    SelectableWidget::MessageLog => { 
                        let message_log_length = battle.message_log.len(); 
                        self.scroll_message_log_down(message_log_length);
                    },
                    SelectableWidget::AllyChoices => { self.ally_ui_panel.scroll_selection_down() },
                    SelectableWidget::OpponentChoices => { self.opponent_ui_panel.scroll_selection_down(); },
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
                            &mut self.ally_ui_panel, 
                            battle,
                            available_actions.ally_team_available_actions,
                        );
                    },
                    SelectableWidget::MessageLog => NOTHING,
                    SelectableWidget::OpponentChoices => {
                        Ui::update_ui_selection_state(
                            &mut self.opponent_ui_panel, 
                            battle,
                            available_actions.opponent_team_available_actions,
                        );
                    },
                    SelectableWidget::AllyRoster => todo!(),
                    SelectableWidget::OpponentRoster => todo!(),
                }
            }
            KeyCode::Tab => { 
                if let (Some(ally_selected_item_index), Some(opponent_selected_item_index)) =
                    (self.ally_ui_panel.selected_item_index, self.opponent_ui_panel.selected_item_index)
                {
                    let fill_out_partial_action = |partial_action| {
                        match partial_action {
                            PartialActionChoice::Move { move_uid, target_uid, .. } => ActionChoice::Move { move_uid, target_uid },
                            PartialActionChoice::SwitchOut { switcher_uid } => todo!(),
                        }
                    };
                    let ally_partial_action = available_actions[TeamID::Allies][ally_selected_item_index].expect("Indices should be valid");
                    let opponent_partial_action = available_actions[TeamID::Opponents][opponent_selected_item_index].expect("Indices should be valid");
                    let chosen_actions = [
                        fill_out_partial_action(ally_partial_action),
                        fill_out_partial_action(opponent_partial_action)
                    ];
                    return Some(AppState::Processing(ProcessingState::Simulating(chosen_actions)));
                } else {
                    battle.push_messages_to_log(
                        &[
                            &"Simulator: Actions were not chosen... please select something before activating the simulation.",
                            &"---",
                            &EMPTY_LINE
                        ]
                    );
                    Ui::snap_message_log_scroll_index_to_turn_end(&mut self.message_log_panel, message_log_length);
                }
            },
            _ => NOTHING,
        }

        None
    }

    /// May update the `AppState`
    fn update_ui_selection_state(
        team_ui_state: &mut TeamUiState, 
        battle: &mut Battle, 
        team_available_actions: AvailableActionsByTeam, 
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

        let switch_out_state = ProposedSwitch {
            switching_battler: battle.active_battlers_by_team(team_id).uid,
            team_id, 
            list_of_choices, 
            list_state: new_list_state(),
        };

        if switch_selected {
            Some(AppState::PromptSwitchOut(switch_out_state))
        } else {
            team_ui_state.selected_item_index = team_ui_state.list_state.selected();
            None
        }
    }

    #[must_use]
    fn post_battle_update(
        &mut self, 
        battle: &mut Battle,
        input_key: KeyCode
    ) -> Option<AppState<'a>> {
        
        if input_key == KeyCode::Esc {
            return Some(AppState::Terminating);
        }

        match input_key {
            KeyCode::Up => { self.scroll_message_log_up(); },
            KeyCode::Down => { 
                let message_log_length = battle.message_log.len(); 
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
    fn new(battle: &mut Battle, team_id: TeamID) -> TeamUiState<'a> {
        let team = battle.team(team_id);
        let team_active_battler = battle.active_battlers_by_team(team_id);
        
        TeamUiState {
            team_id,
            active_battler_status: team_active_battler.status_string(),
            team_roster_status: team.to_string(),
            list_items: Vec::with_capacity(5),
            list_state: new_list_state(),
            selected_item_index: None,
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

    fn update_list(&mut self, available_actions_by_team: AvailableActionsByTeam) {
        self.list_items.clear();
        for choice in available_actions_by_team.into_iter() {
            match choice {
                PartialActionChoice::Move { display_text, .. } => {
                    self.list_items.push(ListItem::new(display_text));
                },
                PartialActionChoice::SwitchOut { switcher_uid: _ } => {
                    self.list_items.push(ListItem::new("Switch Out"));
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
