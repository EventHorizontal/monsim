use std::{
    io::Stdout,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use crate::sim::*;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::CompletedFrame,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};

const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: u64 = 20;
type MonsimIoResult = Result<bool, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
enum AppMode {
    AwaitingUserInput { available_actions: AvailableActions },
    Simulating { chosen_actions: ChosenActions },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScrollableWidgets {
    AllyTeamStatus,
    OpponentTeamStatus,
    MessageLog,
    AllyTeamChoices,
    OpponentTeamChoices,
}

#[derive(Debug, Clone)]
pub struct AppState<'a> {
    app_mode: AppMode,
    ally_list_items: Vec<ListItem<'a>>,
    ally_list_state: ListState,
    ally_active_battler_string: String,
    ally_team_string: String,
    opponent_list_items: Vec<ListItem<'a>>,
    opponent_list_state: ListState,
    opponent_active_battler_string: String,
    opponent_team_string: String,
    message_buffer: MessageBuffer,
    selected_list: ScrollableWidgets,
    message_log_scroll_idx: usize,
    is_battle_ongoing: bool,
}

impl<'a> AppState<'a> {
    fn new(battle: &mut Battle) -> Self {
        let mut state = Self {
            app_mode: AppMode::AwaitingUserInput {
                available_actions: battle.generate_available_actions(),
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
            ally_active_battler_string: BattlerTeam::battler_status_as_string(
                battle.ally_team.active_battler(),
            ),
            ally_team_string: battle.ally_team.to_string(),
            opponent_active_battler_string: BattlerTeam::battler_status_as_string(
                battle.opponent_team.active_battler(),
            ),
            opponent_team_string: battle.opponent_team.to_string(),
        };
        state.build_list_items(battle);
        state
    }

    fn build_list_items(&mut self, battle_context: &Battle) {
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

    fn update_battle_related_state(&mut self, battle: &mut Battle) {
        *self = Self {
            message_buffer: battle.message_buffer.clone(),
            is_battle_ongoing: battle.sim_state != SimState::BattleFinished,
            ally_active_battler_string: BattlerTeam::battler_status_as_string(
                battle.ally_team.active_battler(),
            ),
            ally_team_string: battle.ally_team.to_string(),
            opponent_active_battler_string: BattlerTeam::battler_status_as_string(
                battle.opponent_team.active_battler(),
            ),
            opponent_team_string: battle.opponent_team.to_string(),
            ..self.clone()
        }
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

pub type MonsimResult = Result<(), Box<dyn std::error::Error>>;

pub fn run(mut battle: BattleSimulator) -> MonsimResult {
    extern crate self as monsim;

    // Initialise the Program
    let mut app_state = AppState::new(&mut battle.battle);

    // Raw mode allows to not require enter presses to get input
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
        let should_exit =
            update_state_from_input(&mut terminal, &mut app_state, &mut battle, &receiver)?;
        if should_exit {
            break 'app;
        }
        render_interface(&mut terminal, &mut app_state)?;
    }

    println!("monsim_tui exited successfully");
    Ok(())
}

fn update_state_from_input(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app_state: &mut AppState,
    battle: &mut BattleSimulator,
    receiver: &Receiver<TuiEvent<KeyEvent>>,
) -> MonsimIoResult {
    let mut result: MonsimIoResult = Ok(false);
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
                                #[cfg(feature = "debug")]
                                {
                                    let removal_result = std::fs::remove_file("debug_output.txt");
                                    if let Err(e) = removal_result {
                                        match e.kind() {
                                            std::io::ErrorKind::NotFound => (),
                                            _ => {
                                                return Err(Box::new(e));
                                            }
                                        }
                                    }
                                }
                                result = Ok(true);
                            }
                            (KeyCode::Up, KeyEventKind::Release) => match app_state.selected_list {
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
                                            (selected_index + opponent_list_items_length - 1)
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
                            },
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
                                                (selected_index + 1) % opponent_list_items_length,
                                            ))
                                        } else {
                                            app_state.opponent_list_state.select(Some(0));
                                        }
                                    }
                                    ScrollableWidgets::MessageLog => {
                                        app_state.message_log_scroll_idx =
                                            (app_state.message_log_scroll_idx + 1)
                                                .min(battle.battle.message_buffer.len());
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
                                #[cfg(feature = "debug")]
                                {
                                    let removal_result = std::fs::remove_file("debug_output.txt");
                                    if let Err(e) = removal_result {
                                        match e.kind() {
                                            std::io::ErrorKind::NotFound => (),
                                            _ => { return Err(Box::new(e)); },
                                        }
                                    }
                                }
                                result = Ok(true);
                            }
                            (KeyCode::Up, KeyEventKind::Release) => {
                                app_state.message_log_scroll_idx =
                                    app_state.message_log_scroll_idx.saturating_sub(1);
                            }
                            (KeyCode::Down, KeyEventKind::Release) => {
                                app_state.message_log_scroll_idx =
                                    (app_state.message_log_scroll_idx + 1)
                                        .min(battle.battle.message_buffer.len());
                            }
                            _ => (),
                        }
                    }
                }
                TuiEvent::Tick => {}
            };
        }
        AppMode::Simulating { ref chosen_actions } => {
            let result = battle.simulate_turn(chosen_actions.clone()); // <- This is the main use of the monsim library
            match result {
                Ok(_) => {
                    battle
                        .battle
                        .push_message(&"(The turn was calculated successfully.)");
                }
                Err(error) => battle.battle.message_buffer.push(format!["{:?}", error]),
            }
            if battle.battle.sim_state == SimState::BattleFinished {
                battle
                    .battle
                    .push_messages(&[&EMPTY_LINE, &"The battle ended."]);
            }
            battle.battle.push_messages(&[&"---", &EMPTY_LINE]);
            app_state.app_mode = AppMode::AwaitingUserInput {
                available_actions: battle.battle.generate_available_actions(),
            };
            app_state.update_battle_related_state(&mut battle.battle);
        }
    }
    result
}

pub fn render_interface<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    app_state: &mut AppState,
) -> std::io::Result<CompletedFrame<'a>> {
    let terminal_height = terminal.size()?.height as usize;

    let longest_message_length =
        app_state.message_buffer.iter().fold(
            0usize,
            |acc, x| {
                if x.len() > acc {
                    x.len()
                } else {
                    acc
                }
            },
        );

    terminal.draw(|frame| {
        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Min(longest_message_length as u16 + 2),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(frame.size());

        // Divide Chunks on Ally Panels
        let ally_panel_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Length(6),
                    Constraint::Length(6),
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        // Ally Active Monster Status Widget
        let ally_stats_widget = Paragraph::new(app_state.ally_active_battler_string.as_str())
            .block(
                Block::default()
                    .title(" Ally Active Monster ")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(ally_stats_widget, ally_panel_chunks[0]);

        // Ally Choice List Menu
        let mut ally_list_state = ListState::default();
        ally_list_state.select(Some(2));
        let ally_choice_menu_widget = List::new(app_state.ally_list_items.clone())
            .block(
                Block::default()
                    .title(
                        if app_state.selected_list == ScrollableWidgets::AllyTeamChoices {
                            Span::styled(
                                " Ally Monster Choices ",
                                Style::default().fg(Color::Yellow),
                            )
                        } else {
                            Span::raw(" Ally Monster Choices ")
                        },
                    )
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(
            ally_choice_menu_widget,
            ally_panel_chunks[1],
            &mut app_state.ally_list_state,
        );

        // Opponent Team Roster Widget
        let ally_team_status_widget = Paragraph::new(app_state.ally_team_string.as_str())
            .block(
                Block::default()
                    .title({
                        if app_state.selected_list == ScrollableWidgets::AllyTeamStatus {
                            Span::styled(" Ally Team Status ", Style::default().fg(Color::Yellow))
                        } else {
                            Span::raw(" Ally Team Status ")
                        }
                    })
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(ally_team_status_widget, ally_panel_chunks[2]);

        // Message Log Widget
        // This clamps the scrolling such that the last line of the text never rises above the bottom of the screen, as would be expected from a scrollable text window.
        app_state.message_log_scroll_idx = app_state.message_log_scroll_idx.min(
            app_state
                .message_buffer
                .len()
                .saturating_sub(terminal_height - 4),
        );
        let text = app_state
            .message_buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, element)| {
                if idx >= app_state.message_log_scroll_idx {
                    Some(Spans::from(Span::raw(element)))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let paragraph_widget = Paragraph::new(text)
            .block(
                Block::default()
                    .title(
                        if app_state.selected_list == ScrollableWidgets::MessageLog {
                            Span::styled(" Message Log ", Style::default().fg(Color::Yellow))
                        } else {
                            Span::raw(" Message Log ")
                        },
                    )
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph_widget, chunks[1]);

        // Divide Chunks on Opponent Panel
        let opponent_panel_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Length(6),
                    Constraint::Length(6),
                ]
                .as_ref(),
            )
            .split(chunks[2]);

        // Opponent Active Monster Status Widget
        let opponent_stats_widget =
            Paragraph::new(app_state.opponent_active_battler_string.as_str())
                .block(
                    Block::default()
                        .title(" Opponent Active Monster ")
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
        frame.render_widget(opponent_stats_widget, opponent_panel_chunks[0]);

        // Opponent Choice List Menu
        let opponent_choice_menu_widget = List::new(app_state.opponent_list_items.clone())
            .block(
                Block::default()
                    .title(
                        if app_state.selected_list == ScrollableWidgets::OpponentTeamChoices {
                            Span::styled(
                                " Opponent Monster Choices ",
                                Style::default().fg(Color::Yellow),
                            )
                        } else {
                            Span::raw(" Opponent Monster Choices ")
                        },
                    )
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(
            opponent_choice_menu_widget,
            opponent_panel_chunks[1],
            &mut app_state.opponent_list_state,
        );

        // Opponent Team Roster Widget
        let opponent_team_status_widget = Paragraph::new(app_state.opponent_team_string.as_str())
            .block(
                Block::default()
                    .title(
                        if app_state.selected_list == ScrollableWidgets::OpponentTeamStatus {
                            Span::styled(
                                " Opponent Team Status ",
                                Style::default().fg(Color::Yellow),
                            )
                        } else {
                            Span::raw(" Opponent Team Status ")
                        },
                    )
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(opponent_team_status_widget, opponent_panel_chunks[2]);
    })
}
