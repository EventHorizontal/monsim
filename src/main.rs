use std::{
    io::Stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use monsim::*;
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
type MonsimIOResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Debug)]
pub enum AppMode {
    AwaitingUserInput { available_actions: AvailableActions },
    Simulating { chosen_actions: ChosenActions },
}

#[derive(Debug)]
pub struct AppState<'a> {
    app_mode: AppMode,
    ally_list_items: Vec<ListItem<'a>>,
    ally_list_state: ListState,
    opponent_list_items: Vec<ListItem<'a>>,
    opponent_list_state: ListState,
    message_buffer: MessageBuffer,
}

impl<'a> AppState<'a> {
    fn new(battle_context: &mut BattleContext) -> Self {
        let mut state = Self {
            app_mode: AppMode::AwaitingUserInput {
                available_actions: battle_context.generate_action_choices(),
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
        };
        state.build_list_items(battle_context);
        state
    }

    fn build_list_items(&mut self, battle_context: &BattleContext) {
        let available_actions = battle_context.generate_action_choices();
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

    fn ally_list_items_length(&self) -> usize {
        self.ally_list_items.len()
    }

    fn opponent_list_items_length(&self) -> usize {
        self.opponent_list_items.len()
    }
}

fn main() -> MonsimIOResult {
    let mut battle = Battle::new(bcontext!(
        {
            AllyTeam {
                mon Torchic "Ruby" {
                    mov Ember,
                    mov Scratch,
                    mov Growl,
                    mov Bubble,
                    abl FlashFire,
                },
                mon Mudkip "Sapphire" {
                    mov Tackle,
                    mov Bubble,
                    abl FlashFire,
                },
                mon Torchic "Emerald" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            },
            OpponentTeam {
                mon Drifloon "Cheerio" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            }
        }
    ));

    // Initialise the Program
    let mut app_state = AppState::new(&mut battle.context);

    // Raw mode allows to not require enter presses to get
    enable_raw_mode().expect("can run in raw mode");

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
                if let Event::Key(key) = event::read().expect(
                    "We should always be able to read the events after the poll is successful",
                ) {
                    sender.send(TUIEvent::Input(key)).expect("can send events");
                }
            }

            // Nothing happened, send a tick event
            if last_tick.elapsed() >= time_out_duration {
                if let Ok(_) = sender.send(TUIEvent::Tick) {
                    last_tick = Instant::now();
                }
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
        match app_state.app_mode {
            AppMode::AwaitingUserInput {
                ref available_actions,
            } => {
                // Do appropriate thing based on input receieved
                match receiver.recv()? {
                    TUIEvent::Input(event) => match event {
                        // Quit
                        KeyEvent {
                            code,
                            modifiers: _,
                            kind: KeyEventKind::Release,
                            state: _,
                        } => {
                            match code {
                                KeyCode::Esc => {
                                    disable_raw_mode()?;
                                    execute!(std::io::stdout(), LeaveAlternateScreen)?;
                                    terminal.show_cursor()?;
                                    break 'app;
                                }
                                KeyCode::Up => {
                                    if let Some(selected_index) =
                                        app_state.opponent_list_state.selected()
                                    {
                                        let opponent_list_items_length =
                                            app_state.opponent_list_items_length();
                                        app_state.opponent_list_state.select(Some(
                                            (selected_index + opponent_list_items_length - 1)
                                                % opponent_list_items_length,
                                        ));
                                    }
                                }
                                KeyCode::Down => {
                                    if let Some(selected_index) =
                                        app_state.opponent_list_state.selected()
                                    {
                                        let opponent_list_items_length =
                                            app_state.opponent_list_items_length();
                                        app_state.opponent_list_state.select(Some(
                                            (selected_index + 1) % opponent_list_items_length,
                                        ))
                                    }
                                }
                                KeyCode::Tab => {
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
                                KeyCode::Char('w') => {
                                    if let Some(selected_index) =
                                        app_state.ally_list_state.selected()
                                    {
                                        let ally_list_length = app_state.ally_list_items_length();
                                        app_state.ally_list_state.select(Some(
                                            (selected_index + ally_list_length - 1)
                                                % ally_list_length,
                                        ));
                                    }
                                }
                                KeyCode::Char('s') => {
                                    if let Some(selected_index) =
                                        app_state.ally_list_state.selected()
                                    {
                                        let ally_list_length = app_state.ally_list_items_length();
                                        app_state
                                            .ally_list_state
                                            .select(Some((selected_index + 1) % ally_list_length))
                                    }
                                }
                                _ => {}
                            };
                        }
                        _ => {}
                    },
                    TUIEvent::Tick => {}
                }
            }
            AppMode::Simulating { chosen_actions } => {
                let result = battle.simulate_turn(chosen_actions); // <- This is the main use of the monsim library
                app_state.app_mode = AppMode::AwaitingUserInput {
                    available_actions: battle.context.generate_action_choices(),
                };
                app_state.message_buffer = battle.context.message_buffer.clone();
                battle.context.message_buffer.clear();
            }
        }

        render(&mut terminal, &mut app_state)?;
    }

    println!("The Battle ended with no errors.\n");
    Ok(())
}

fn render<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    app_state: &mut AppState,
) -> std::io::Result<CompletedFrame<'a>> {
    terminal.draw(|frame| {
        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(frame.size());

        // Ally Monster Stats Widget
        let mut ally_list_state = ListState::default();
        ally_list_state.select(Some(2));
        let ally_widget = List::new(app_state.ally_list_items.clone())
            .block(
                Block::default()
                    .title(" Ally Team Choices ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(ally_widget, chunks[0], &mut app_state.ally_list_state);

        // Message Log Widget
        let text = app_state
            .message_buffer
            .iter()
            .map(|element| Spans::from(Span::raw(element)))
            .collect::<Vec<_>>();
        let paragraph_widget = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Message Log ")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph_widget, chunks[1]);

        // Opponent Monster Stats Widget
        let opponent_widget = List::new(app_state.opponent_list_items.clone())
            .block(
                Block::default()
                    .title(" Opponent Team Choices ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(
            opponent_widget,
            chunks[2],
            &mut app_state.opponent_list_state,
        );
    })
}
enum TUIEvent<I> {
    Input(I),
    Tick,
}
