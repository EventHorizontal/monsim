use std::{sync::mpsc, time::{Duration, Instant}, thread, io::Stdout};

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode}, event::{self, Event, KeyCode, KeyEventKind, KeyEvent}, execute};
use tui::{backend::CrosstermBackend, Terminal, terminal::CompletedFrame, widgets::{ListState, ListItem, Paragraph, Block, Borders, Wrap, List}, layout::{Layout, Direction, Constraint, Rect, Alignment}, Frame, text::{Span, Spans}, style::{Style, Color, Modifier}};

use crate::sim::{BattleSimulator, Battle, ChosenActions, EMPTY_LINE, SimState, AvailableActions, ActionChoice, BattlerTeam, MessageBuffer, TeamID, TeamAvailableActions, utils::{NOTHING, Nothing, not}};

#[derive(Debug, Clone)]
pub struct App<'a> {
	state: AppState,
	currently_selected_widget: SelectableWidget,
	message_log_ui_state: MessageLogUiState,
	ally_ui_state: TeamUiState<'a>,
	opponent_ui_state: TeamUiState<'a>,
	switch_out_state: Option<SwitchOutState<'a>>, 
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
	Initialising,
	Processing(ProcessingState),
	PromptSwitchOut(TeamID),
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
	name: TeamID,
	active_battler_status: String,
	team_roster_status: String,
	list_items: Vec<ListItem<'a>>,	
	list_state: ListState,
	selected: Option<SelectableOptions>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectableWidget {
	AllyChoices,
	MessageLog,
	OpponentChoices,
	AllyRoster,
	OpponentRoster,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectableOptions {
	Move(usize),
	SwitchOut,
}

#[derive(Debug, Clone)]
pub struct SwitchOutState<'a> {
	team_id: TeamID,
	list_of_choices: List<'a>,
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

	app.transition_to(AppState::Processing(AwaitingUserInput(battle_sim.battle.available_actions())));
	render_interface(&mut terminal, &mut app, &battle_sim.battle.message_buffer)?;

	'app: loop {
		match app.state {
			AppState::Initialising => unreachable!("The app never transitions back to the initialising state."),
			AppState::Processing(ref processing_state) => {
				match processing_state.clone() {
					ProcessingState::AwaitingUserInput(available_actions) => {
						process_awaiting_input_state(&mut terminal, &mut app, &mut battle_sim, &receiver, available_actions)?;
					},
					ProcessingState::Simulating(chosen_actions) => {
						let result = battle_sim.simulate_turn(chosen_actions.clone());
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
			AppState::PromptSwitchOut(team_id) => { process_switch_out(team_id, &mut terminal, &mut app, &battle_sim.battle, &receiver)? }, 
			AppState::Exiting => { break 'app; }
		}
		render_interface(&mut terminal, &mut app, &battle_sim.battle.message_buffer)?;
	}

	println!("monsim_tui exited successfully");
	Ok(NOTHING)
}

fn process_switch_out(
	team_id: TeamID, 
	terminal: &mut Terminal<CrosstermBackend<Stdout>>,
	app: &mut App,
	battle: &Battle,
	receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
) -> AppResult<Nothing> {
	let team = match team_id {
		TeamID::Allies => { battle.ally_team.inner() },
		TeamID::Opponents => { battle.opponent_team.inner() },
	};

	use KeyEventKind::Release;
	match receiver.recv()? {
		TuiEvent::Input(event) => {
			match (event.code, event.kind) {
				(KeyCode::Up, Release) => {
					// Nothing yet...
				},
				(KeyCode::Down, Release) => {
					// Nothing yet...
				},
				(KeyCode::Esc, Release) =>  {
					terminate(terminal, app)?; 
					return Ok(NOTHING);
				}
				_ => {}
			}
		},
		TuiEvent::Tick => {},
	}

	let list_of_choices = team.battlers()
		.iter()
		.filter_map(|battler| {
			if not!(battle.is_battler_on_field(battler.uid)) {
				Some(ListItem::new(battler.monster.nickname))
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	let list_of_choices = List::new(list_of_choices);

	app.switch_out_state = Some(SwitchOutState { 
		team_id, 
		list_of_choices, 
		list_state: new_list_state(),
	});

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

fn process_awaiting_input_state<'a>(
	terminal: &mut Terminal<CrosstermBackend<Stdout>>,
	mut app: &mut App<'a>,
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
				terminate(terminal, &mut app)?; 
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
					(KeyCode::Left, Release) => { {
						app.currently_selected_widget.shift_left()
					}; }
					(KeyCode::Right, Release) => { {
						app.currently_selected_widget.shift_right()
					}; },
					(KeyCode::Enter, Release) => {
						if app.currently_selected_widget == SelectableWidget::AllyChoices {
							app.state = AppState::PromptSwitchOut(TeamID::Allies);
						} else if app.currently_selected_widget == SelectableWidget::OpponentChoices {
							app.state = AppState::PromptSwitchOut(TeamID::Opponents);
						}
					}
					(KeyCode::Tab, Release) => { 
						if let (Some(selected_ally_choice_index), Some(selected_opponent_choice_index)) =
							(app.ally_ui_state.list_state.selected(), app.opponent_ui_state.list_state.selected())
						{
							// TODO: Think about how to move the expects to the implementation of TeamAvailableActions
							let chosen_actions = [
								available_actions.ally_team_available_actions[selected_ally_choice_index].expect("actions should be valid at this point"),
								available_actions.opponent_team_available_actions[selected_opponent_choice_index].expect("actions should be valid at this point"),
							];
							app.transition_to(AppState::Processing(Simulating(chosen_actions))); 
						} else {
							battle_sim.battle.push_message(&"Simulator: Invalid choices, could not simulate turn.");
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

fn terminate(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> AppResult<Nothing> {
	disable_raw_mode()?;
	execute!(std::io::stdout(), LeaveAlternateScreen)?;
	terminal.show_cursor()?;
	#[cfg(feature="debug")]
	remove_debug_log_file()?;
	app.transition_to(AppState::Exiting);
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
		
		// TODO: Could we get away with caching the panels?
		let ally_panel_chunks = divide_panel_into_chunks(chunks[0]);
		let ally_stats_widget = create_stats_panel_widget(&app.ally_ui_state); 
		let ally_choice_menu_widget = create_choice_menu_widget(&app.ally_ui_state, app.currently_selected_widget);
		let ally_team_roster_widget = create_roster_widget(&app.ally_ui_state, app.currently_selected_widget);

		let message_log_widget = create_message_log_widget(message_buffer, app.message_log_ui_state.message_log_scroll_idx, app.currently_selected_widget);
		
		let opponent_panel_chunks = divide_panel_into_chunks(chunks[2]);
		let opponent_stats_widget = create_stats_panel_widget(&app.opponent_ui_state); 
		let opponent_choice_menu_widget = create_choice_menu_widget(&app.opponent_ui_state, app.currently_selected_widget);
		let opponent_team_roster_widget = create_roster_widget(&app.opponent_ui_state, app.currently_selected_widget);

		if let Some(ref mut switch_out_state) = &mut app.switch_out_state {
			
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

			let switch_out_widget = switch_out_state.list_of_choices
				.clone()
				.block(
					Block::default()
						.title(Span::styled(format![" Switch out with? "], Style::default().fg(Color::Yellow)))
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

fn create_stats_panel_widget<'a>(team_ui_state: &'a TeamUiState) -> Paragraph<'a> {
	let team_name = team_ui_state.name;
	Paragraph::new(team_ui_state.active_battler_status.as_str())
		.block(Block::default().title(format![" {team_name} Active Monster "]).borders(Borders::ALL))
		.alignment(Alignment::Left)
		.wrap(Wrap { trim: true })
}

fn create_roster_widget<'a>(team_ui_state: &'a TeamUiState, currently_selected_list: SelectableWidget) -> Paragraph<'a> {
	let team_name = team_ui_state.name;
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

fn create_choice_menu_widget<'a>(team_ui_state: &'a TeamUiState, currently_selected_list: SelectableWidget) -> List<'a> {
	let team_name = team_ui_state.name;
	let team_choices_list = match team_name {
		TeamID::Allies => SelectableWidget::AllyChoices,
		TeamID::Opponents => SelectableWidget::OpponentChoices,
	};
	List::new(team_ui_state.list_items.clone())
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
fn create_message_log_widget<'a>(message_buffer: &'a MessageBuffer, message_log_scroll_idx: usize, currently_selected_list: SelectableWidget) -> Paragraph<'a> {
	let text = message_buffer
		.iter()
		.enumerate()
		.filter_map(|(idx, element)| {
			if idx >= message_log_scroll_idx { 
				if element.contains("Turn") {
					Some(Spans::from(Span::styled(element, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))))
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
			ally_ui_state: TeamUiState::new(&mut battle.ally_team.inner(), TeamID::Allies, ally_team_available_actions),
			opponent_ui_state: TeamUiState::new(&mut battle.opponent_team.inner(), TeamID::Opponents, opponent_team_available_actions),
			switch_out_state: None,
		}
	}

	fn transition_to(&mut self, destination_state: AppState) {
		self.state = destination_state;
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

		// We want to scroll to the end of the last turn, which is also the beginning of the next turn
		self.message_log_ui_state.message_log_scroll_idx = self.message_log_ui_state.last_message_buffer_length;
		self.message_log_ui_state.message_log_last_scrollable_line_idx = self.message_log_ui_state.message_log_scroll_idx;
		self.message_log_ui_state.last_message_buffer_length = battle.message_buffer.len();

		self.ally_ui_state = TeamUiState {
			active_battler_status: BattlerTeam::battler_status_as_string(battle.ally_team.active_battler()),
			team_roster_status: battle.ally_team.to_string(),
			..self.ally_ui_state.clone()
		};

		TeamUiState::regenerate_list(&battle.ally_team.inner(), &mut self.ally_ui_state.list_items, ally_team_available_actions);

		self.opponent_ui_state = TeamUiState {
			active_battler_status: BattlerTeam::battler_status_as_string(battle.opponent_team.active_battler()),
			team_roster_status: battle.opponent_team.to_string(),
			..self.opponent_ui_state.clone()
		};

		TeamUiState::regenerate_list(&battle.opponent_team.inner(), &mut self.opponent_ui_state.list_items, opponent_team_available_actions);
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
	fn new(team: &mut BattlerTeam, name: TeamID, available_actions: TeamAvailableActions) -> TeamUiState<'a> {
		let mut list_items = Vec::with_capacity(5);
		Self::regenerate_list(&team, &mut list_items, available_actions);
		TeamUiState {
			name,
			active_battler_status: BattlerTeam::battler_status_as_string(team.active_battler()),
			team_roster_status: team.to_string(),
			list_items,
			list_state: new_list_state(),
			selected: None,
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
				ActionChoice::Move { move_uid, target_uid: _ } => {
					for battler in team.battlers() {
						if battler.uid == move_uid.battler_uid {
							let move_ = battler.moveset[move_uid.move_number as usize];
							list_items.push(ListItem::new(move_.species.name));
						}
					}
				},
				ActionChoice::SwitchOut { active_battler_uid: _, benched_battler_uid: _ } => {
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
