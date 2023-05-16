use std::{
    io::Stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
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
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, ListState},
    Terminal,
};

const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: u64 = 20;
type MonsimIOResult = Result<(), Box<dyn std::error::Error>>;

pub enum AppState {
    AwaitingUserInput {
        action_choices: AvailableActions,
    },
    // InputReceived { chosen_actions: UserChoice },
    Simulating {
        user_input: UserInput,
    },
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
    let mut app_state = AppState::AwaitingUserInput {
        action_choices: battle.context.generate_action_choices(),
    };

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

    let mut overall_action_choices = None;
    'app: loop {
        match app_state {
            AppState::AwaitingUserInput { ref action_choices } => {
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
                                _ => {}
                            };
                            overall_action_choices = Some(action_choices.clone());
                        }
                        _ => {}
                    },
                    TUIEvent::Tick => {}
                }
            }
            AppState::Simulating { ref user_input } => {
                let _result = battle.simulate_turn(user_input); // <- This is the main use of the monsim library
                app_state = AppState::AwaitingUserInput {
                    action_choices: battle.context.generate_action_choices(),
                }
            }
        }

        render(&mut terminal, &battle.context, &overall_action_choices)?;
    }

    battle.context.message_buffer.clear();
    println!("The Battle ended with no errors.\n");
    Ok(())
}

fn render<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    battle_context: &BattleContext,
    action_choices: &Option<AvailableActions>,
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
        let items = {
            match action_choices {
                Some(choices) => choices
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
                None => vec![],
            }
        };
        let mut ally_list_state = ListState::default();
        ally_list_state.select(Some(2));
        let ally_widget = List::new(items)
            .block(Block::default().title(" Ally Team Choices ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(ally_widget, chunks[0], &mut ally_list_state);


        // Message log widget
        let text = battle_context
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
        let items = {
            match action_choices {
                Some(choices) => choices
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
                None => vec![],
            }
        };
        let mut opponent_list_state = ListState::default();
        opponent_list_state.select(Some(0));
        let opponent_widget = List::new(items)
            .block(Block::default().title(" Opponent Team Choices ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        frame.render_stateful_widget(opponent_widget, chunks[2], &mut opponent_list_state);
    })
}
enum TUIEvent<I> {
    Input(I),
    Tick,
}
