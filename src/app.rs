//FIXME: Broken

use std::{sync::mpsc, time::{Duration, Instant}, thread};

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode}, event::{self, Event, KeyCode, KeyEventKind, KeyEvent}, execute};
use tui::{backend::CrosstermBackend, Terminal, terminal::CompletedFrame, widgets::{ListState, ListItem, Paragraph, Block, Borders, Wrap, List}, layout::{Layout, Direction, Constraint, Rect, Alignment}, Frame, text::{Span, Spans}, style::{Style, Color, Modifier}};

use crate::{sim::{BattleSimulator, NOTHING, Battle, Nothing, ChosenActions, EMPTY_LINE, SimState, AvailableActions, ActionChoice, BattlerTeam, MessageBuffer}, not};

#[derive(Debug, Clone)]
pub struct App<'a> {
	state: AppState,
	selected_list_idx: usize,
	message_log_scroll_idx: usize,
	ally_ui_state: TeamUiState<'a>,
	opponent_ui_state: TeamUiState<'a>,
	message_buffer: MessageBuffer
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
	Initialising,
	Processing(ProcessingState),
	Exiting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessingState {
	AwaitingUserInput(AvailableActions),
	Simulating(ChosenActions),
}

#[derive(Debug, Clone)]
pub struct TeamUiState<'a> {
	name: &'a str,
	active_battler_status: String,
	team_roster_status: String,
	list_items: Vec<ListItem<'a>>,	
	list_state: ListState,
}

impl<'a> TeamUiState<'a> {
    fn new(team: &mut BattlerTeam, name: &'a str, action_choices: Vec<ActionChoice>) -> TeamUiState<'a> {
        let mut list_items = Vec::with_capacity(5);
		regenerate_list(&team, &mut list_items, action_choices);
		TeamUiState {
			name,
            active_battler_status: BattlerTeam::battler_status_as_string(team.active_battler()),
            team_roster_status: team.to_string(),
            list_items,
            list_state: {
                let mut list = ListState::default();
                list.select(Some(0));
                list
            },
        }
    }

	fn scroll_selection_up(&mut self) {
		if let Some(selected_index) = self.list_state.selected() {
			let new_index = (selected_index + self.list_length() - 1) % self.list_length();
			self.list_state.select(Some(new_index));
		} else {
			unreachable!();
			// self.list_state.select(Some(0));
		}
	}
	
	fn scroll_selection_down(&mut self) {
		if let Some(selected_index) = self.list_state.selected() {
			let new_index = (selected_index + 1) % self.list_length();
			self.list_state.select(Some(new_index));
		} else {
			unreachable!();
			// self.list_state.select(Some(0));
		}
	}
	
	#[inline(always)]
	fn list_length(&self) -> usize {
		self.list_items.len()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScrollableList {
	MessageLog,
	AllyChoices,
	OpponentChoices,
}

impl<'a> App<'a> {
    pub fn new(battle: &mut Battle) -> App<'a> {
		let AvailableActions { ally_team_choices, opponent_team_choices } = battle.generate_available_actions();
		App {
			state: AppState::Initialising,
			selected_list_idx: 1,
			ally_ui_state: TeamUiState::new(&mut battle.ally_team.inner(), "Ally", ally_team_choices),
			opponent_ui_state: TeamUiState::new(&mut battle.opponent_team.inner(), "Opponent", opponent_team_choices),
			message_log_scroll_idx: 0,
    		message_buffer: vec![],
		}
	}

    fn transition_to(&mut self, destination_state: AppState) {
        self.state = destination_state;
    } 

	fn scroll_message_log_up(&mut self) {
		self.message_log_scroll_idx = self.message_log_scroll_idx.saturating_sub(1);
	}

	fn scroll_message_log_down(&mut self, message_log_length: usize) {
		self.message_log_scroll_idx = (self.message_log_scroll_idx + 1)
			.min(message_log_length);
	}

	fn regenerate_ui_data(&mut self, battle: &mut Battle) {
		
		let AvailableActions {
			ally_team_choices,
			opponent_team_choices,
		} = battle.generate_available_actions();

		self.message_buffer.extend(battle.message_buffer.clone().into_iter());
		battle.message_buffer.clear();

		self.ally_ui_state = TeamUiState {
			active_battler_status: BattlerTeam::battler_status_as_string(battle.ally_team.active_battler()),
			team_roster_status: battle.ally_team.to_string(),
			..self.ally_ui_state.clone()
		};

		regenerate_list(&battle.ally_team.inner(), &mut self.ally_ui_state.list_items, ally_team_choices);

		self.opponent_ui_state = TeamUiState {
			active_battler_status: BattlerTeam::battler_status_as_string(battle.opponent_team.active_battler()),
			team_roster_status: battle.opponent_team.to_string(),
			..self.opponent_ui_state.clone()
		};

		regenerate_list(&battle.opponent_team.inner(), &mut self.opponent_ui_state.list_items, opponent_team_choices);
	}
}

fn regenerate_list(team: &BattlerTeam, list_items: &mut Vec<ListItem>, action_choices: Vec<ActionChoice>) {
	list_items.clear();
	for choice in action_choices.iter() {
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

impl From<usize> for ScrollableList {
    fn from(value: usize) -> Self {
		match value {
			0 => ScrollableList::AllyChoices,
			1 => ScrollableList::MessageLog,
			2 => ScrollableList::OpponentChoices,
			_ => panic!("index for scrollable list selector out of bounds. Expected an index between 0 and 2, found {value}")
		}
    }
}

enum TuiEvent<I> {
    Input(I),
    Tick,
}

const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: Duration = Duration::from_millis(20);

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

	app.transition_to(AppState::Processing(AwaitingUserInput(battle_sim.available_actions())));
	render_interface(&mut terminal, &mut app)?;

	'app: loop {
		match app.state {
			AppState::Initialising => unreachable!("The app never transitions back to the initialising state."),
			AppState::Processing(ref processing_state) => match processing_state {
				ProcessingState::AwaitingUserInput(available_actions) => {
					app = process_input(&mut terminal, app.clone(), &mut battle_sim, &receiver, available_actions.clone())?;
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
					
					let available_actions = battle_sim.available_actions();
					app.state = AppState::Processing(AwaitingUserInput(available_actions));
					app.regenerate_ui_data(&mut battle_sim.battle);
				},
			}
			AppState::Exiting => { break 'app; } 
		}
		if app.state == AppState::Exiting { break 'app };
		render_interface(&mut terminal, &mut app)?;
	}

	println!("monsim_tui exited successfully");
	Ok(NOTHING)
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

#[must_use]
fn process_input<'a>(
	terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
	mut app: App<'a>,
	battle_sim: &mut BattleSimulator,
	receiver: &mpsc::Receiver<TuiEvent<KeyEvent>>,
	available_actions: AvailableActions,
) -> AppResult<App<'a>> {
	match receiver.recv()? {
		TuiEvent::Input(event) => {
			use KeyEventKind::Release;
			use ProcessingState::Simulating;
			
			let was_escape_key_released = (KeyCode::Esc, Release) == (event.code, event.kind);
			if was_escape_key_released { 
				terminate(terminal, &mut app)?; 
				return Ok(app) 
			}

			if not!(battle_sim.sim_state == SimState::BattleFinished) {
				match (event.code, event.kind) {
					(KeyCode::Up, Release) => {
						match ScrollableList::from(app.selected_list_idx) {
							ScrollableList::MessageLog => { 
								app.scroll_message_log_up();
							},
							ScrollableList::AllyChoices => { app.ally_ui_state.scroll_selection_up() },
							ScrollableList::OpponentChoices => { app.opponent_ui_state.scroll_selection_up(); },
						}
					},
					(KeyCode::Down, Release) => {
						match ScrollableList::from(app.selected_list_idx) {
							ScrollableList::MessageLog => { 
								let message_log_length = app.message_buffer.len(); 
								app.scroll_message_log_down(message_log_length);
							},
							ScrollableList::AllyChoices => { app.ally_ui_state.scroll_selection_down() },
							ScrollableList::OpponentChoices => { app.opponent_ui_state.scroll_selection_down(); },
						}
					},
					(KeyCode::Left, Release) => {
						app.selected_list_idx = (app.selected_list_idx + 2) % 3; // Moves list cursor to the left.
					}
					(KeyCode::Right, Release) => {
						app.selected_list_idx = (app.selected_list_idx + 1) % 3; // Moves list cursor tot the right.
					},
					(KeyCode::Tab, Release) => { 
						if let (Some(selected_ally_choice_index), Some(selected_opponent_choice_index)) =
							(app.ally_ui_state.list_state.selected(), app.opponent_ui_state.list_state.selected())
						{
							let chosen_actions = [
								available_actions.ally_team_choices[selected_ally_choice_index],
								available_actions.opponent_team_choices[selected_opponent_choice_index],
							];
							app.transition_to(AppState::Processing(Simulating(chosen_actions))); 
						}
					},
					_ => {},
				}
			} else { // Battle is finished
				match (event.code, event.kind) {
					(KeyCode::Up, Release) => {
						app.scroll_message_log_up();
					},
					(KeyCode::Down, Release) => {
						let message_log_length = battle_sim.battle.message_buffer.len(); 
						app.scroll_message_log_down(message_log_length);
					},
					_ => {},
				}	
			}
		},
		TuiEvent::Tick => NOTHING,
	}
	Ok(app)
}

fn terminate(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App<'_>) -> AppResult<Nothing> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
	#[cfg(feature = "debug")]
	std::fs::remove_file("debug_output.txt")?;
    app.transition_to(AppState::Exiting);
    Ok(NOTHING)
}

fn render_interface<'a>(
	terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>, 
	app: &mut App
) -> std::io::Result<CompletedFrame<'a>> {
	let terminal_height = terminal.size()?.height as usize;
	
	terminal.draw(|frame| {
		let chunks = divide_screen_into_chunks(frame);
		
		let ally_panel_chunks = divide_panel_into_chunks(chunks[0]);
		let ally_stats_widget = create_stats_panel_widget(&app.ally_ui_state); 
		let ally_choice_menu_widget = create_choice_menu_widget(&app.ally_ui_state, app.selected_list_idx);
		let ally_team_roster_widget = create_roster_widget(&app.ally_ui_state);

		// Clamping ensures the bottom of the text can never scroll above the bottom of the screen.
		app.message_log_scroll_idx = app.message_log_scroll_idx.min(app.message_buffer.len().saturating_sub(terminal_height - 4));
		let message_log_widget = create_message_log_scroll_widget(&app.message_buffer, app.message_log_scroll_idx, app.selected_list_idx);
		
		let opponent_panel_chunks = divide_panel_into_chunks(chunks[2]);
		let opponent_stats_widget = create_stats_panel_widget(&app.opponent_ui_state); 
		let opponent_choice_menu_widget = create_choice_menu_widget(&app.opponent_ui_state, app.selected_list_idx);
		let opponent_team_roster_widget = create_roster_widget(&app.opponent_ui_state);

		frame.render_widget(ally_stats_widget, ally_panel_chunks[0]);
		let mut ally_list_state = app.ally_ui_state.list_state.clone();
		frame.render_stateful_widget(ally_choice_menu_widget, ally_panel_chunks[1], &mut ally_list_state); 
		frame.render_widget(ally_team_roster_widget, ally_panel_chunks[2]);
		
		frame.render_widget(message_log_widget, chunks[1]);
		
		frame.render_widget(opponent_stats_widget, opponent_panel_chunks[0]);
		let mut opponent_list_state = app.opponent_ui_state.list_state.clone();
		frame.render_stateful_widget(opponent_choice_menu_widget, opponent_panel_chunks[1], &mut opponent_list_state);
		frame.render_widget(opponent_team_roster_widget, opponent_panel_chunks[2]);
	})
}

fn create_message_log_scroll_widget<'a>(message_buffer: &'a MessageBuffer, message_log_scroll_idx: usize, selected_list_idx: usize) -> Paragraph<'a> {
	let text = message_buffer
		.iter()
		.enumerate()
		.filter_map(|(idx, element)| {
			if idx >= message_log_scroll_idx {
				Some(Spans::from(Span::raw(element)))
			} else {
				None
			}
		})
		.collect::<Vec<_>>();
    Paragraph::new(text)
		.block(
			Block::default()
				.title(if ScrollableList::from(selected_list_idx) == ScrollableList::MessageLog {
					Span::styled(" Message Log ", Style::default().fg(Color::Yellow))
				} else {
					Span::raw(" Message Log ")
				})
				.borders(Borders::ALL),
		)
		.alignment(Alignment::Center)
		.wrap(Wrap { trim: true })
}

fn create_roster_widget<'a>(team_ui_state: &'a TeamUiState) -> Paragraph<'a> {
    let team_name = team_ui_state.name;
	Paragraph::new(team_ui_state.team_roster_status.as_str())
		    .block(
			    Block::default()
				    .title({
						// TODO: Make this widget selectable
					    // if ScrollableLists::from(selected_list_idx) == ScrollableLists::AllyTeamStatus {
						//     Span::styled(" Ally Team Status ", Style::default().fg(Color::Yellow))
					    // } else {
						    Span::raw(format![" {team_name} Team Status "])
					    // }
				    })
				    .borders(Borders::ALL),
		    )
		    .alignment(Alignment::Left)
		    .wrap(Wrap { trim: true })
}

fn create_choice_menu_widget<'a>(team_ui_state: &'a TeamUiState, selected_list_idx: usize) -> List<'a> {
	let team_name = team_ui_state.name;
	let expected_list = if team_name == "Ally" { ScrollableList::AllyChoices } else { ScrollableList::OpponentChoices };
	
	List::new(team_ui_state.list_items.clone())
		.block(
			Block::default()
				.title(if ScrollableList::from(selected_list_idx) == expected_list {
					Span::styled(format![" {team_name} Monster Choices "], Style::default().fg(Color::Yellow))
				} else {
					Span::raw(format![" {team_name} Monster Choices "])
				})
				.borders(Borders::ALL),
		)
		.highlight_style(Style::default().add_modifier(Modifier::ITALIC))
		.highlight_symbol(">>")
}

fn create_stats_panel_widget<'a>(team_ui_state: &'a TeamUiState) -> Paragraph<'a> {
    let team_name = team_ui_state.name;
	Paragraph::new(team_ui_state.active_battler_status.as_str())
		.block(Block::default().title(format![" {team_name} Active Monster "]).borders(Borders::ALL))
		.alignment(Alignment::Left)
		.wrap(Wrap { trim: true })
}

fn divide_panel_into_chunks(chunk: Rect) -> Vec<Rect> {
    Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Percentage(33), Constraint::Length(8), Constraint::Length(6)].as_ref())
		.split(chunk)
}

fn divide_screen_into_chunks(frame: &mut Frame<CrosstermBackend<std::io::Stdout>>) -> Vec<Rect> {
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
