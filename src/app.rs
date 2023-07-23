use std::{sync::mpsc, time::{Duration, Instant}, thread, io::Stdout, ops::ControlFlow};

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode}, event::{self, Event, KeyCode, KeyEventKind, KeyEvent}, execute};
use tui::{backend::CrosstermBackend, Terminal, terminal::CompletedFrame, widgets::{ListState, ListItem, Paragraph, Block, Borders, Wrap, List}, layout::{Layout, Direction, Constraint, Rect, Alignment}, Frame, text::{Span, Spans}, style::{Style, Color, Modifier}};

use crate::sim::{BattleSimulator, Battle, ChosenActions, EMPTY_LINE, SimState, AvailableActions, ActionChoice, BattlerTeam, MessageBuffer, TeamID, TeamAvailableActions, utils::{NOTHING, Nothing, not}, EnumeratedActionChoice, BattlerUID};

mod update;
mod render;

use update::update_app_state;
use render::render_interface;

pub type AppResult<T> = Result<T, BoxedError>;
type BoxedError = Box<dyn std::error::Error>;

/// `main` function for app.rs
pub fn run(mut battle: Battle) -> AppResult<Nothing> {
    
    let mut app = AppInstance::new(&mut battle);
    let mut simulator = BattleSimulator::new(battle);
    let mut terminal = create_configured_terminal()?;
    let (sender, receiver) = mpsc::channel();
    create_io_thread(sender);
    
    // Render interface once before the update loop starts.
    render_interface(&mut app, &mut terminal, &simulator.battle.message_buffer)?;
    'app: loop {
        if update_app_state(&mut app, &mut terminal, &mut simulator, &receiver)?.is_break() { break 'app };
        render_interface(&mut app, &mut terminal, &simulator.battle.message_buffer)?;
    }

    println!("monsim_tui exited successfully");
    Ok(NOTHING)
}

#[derive(Debug, Clone)]
pub struct AppInstance<'a> {
    state: AppState<'a>,
    currently_selected_widget: SelectableWidget,
    message_log_ui_state: MessageLogUiState,
    ally_panel_ui_state: TeamUiState<'a>,
    opponent_panel_ui_state: TeamUiState<'a>,
}

#[derive(Debug, Clone)]
enum AppState<'a> {
    Processing(ProcessingState),
    PromptSwitchOut(SwitchOutState<'a>),
    Exiting,
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

impl<'a> AppInstance<'a> {
    pub fn new(battle: &mut Battle) -> AppInstance<'a> {
        let AvailableActions { ally_team_available_actions, opponent_team_available_actions } = battle.available_actions();
        AppInstance {
            state: AppState::Processing(ProcessingState::AwaitingUserInput(battle.available_actions())),
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

    fn regenerate_ui_data(&mut self, battle: &mut Battle, available_actions: AvailableActions) {
        
        let AvailableActions {
            ally_team_available_actions,
            opponent_team_available_actions,
        } = available_actions;

        AppInstance::snap_message_log_scroll_index_to_turn_end(&mut self.message_log_ui_state, battle);
        
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

    fn confirm_selection(team_ui_state: &mut TeamUiState, team_available_actions: TeamAvailableActions) {
        if let Some(selected_index) = team_ui_state.list_state.selected() {
            team_ui_state.selected_action = team_available_actions[selected_index]; 
        }
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
