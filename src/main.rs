use std::{thread, time::{Duration, Instant}, sync::mpsc, io::Stdout};

use monsim::*;
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, execute};
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, Paragraph, Wrap}, style::{Style, Color}, layout::{Alignment, Layout, Direction, Constraint}, text::{Spans, Span}, terminal::CompletedFrame};


const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: u64 = 20;
type MonsimIOResult = Result<(), Box<dyn std::error::Error>>; 

pub enum AppState {
    AwaitingUserInput { action_choices: AvailableActionChoices },
    // InputReceived { chosen_actions: UserChoice },
    Simulating { user_input: UserInput }
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
    let mut app_state = AppState::AwaitingUserInput { action_choices: battle.context.generate_action_choices() };
    
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
                if let Event::Key(key) = event::read().expect("We should always be able to read the events after the poll is successful") {
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

        match app_state {
            AppState::AwaitingUserInput { ref action_choices } => {
                // Do appropriate thing based on input receieved
                match receiver.recv()? {
                    TUIEvent::Input(event) => match event {
                        // Quit
                        KeyEvent { code, modifiers: _, kind: KeyEventKind::Release, state: _ } => {
                            if code == KeyCode::Esc {
                                disable_raw_mode()?;
                                execute!(std::io::stdout(), LeaveAlternateScreen)?;
                                terminal.show_cursor()?;
                                break 'app;
                            }
                            // Keep simulating turns until the battle is finished.
                            render(&mut terminal, &battle.context.message_buffer, Some(action_choices))?;
                        },
                        _ => {},
                    },
                    TUIEvent::Tick => {},
                }
            },
            AppState::Simulating { ref user_input } => {
                let _result = battle.simulate_turn(user_input); // <- This is the main use of the monsim library
                app_state = AppState::AwaitingUserInput { action_choices: battle.context.generate_action_choices()}
            },
        }


        // Draw the result of the current turn to the terminal
        // render(&mut terminal, &battle.context.message_buffer)?;
    }
    
    battle.context.message_buffer.clear();
    println!("The Battle ended with no errors.\n");
    Ok(())
}

fn render<'a>(
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    message_buffer: &Vec<String>,
    action_choices: Option<&AvailableActionChoices>,
) -> std::io::Result<CompletedFrame<'a>> {
    terminal.draw( |frame| {    
        
        // Chunks
        let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33)
            ].as_ref()
        )
        .split(frame.size());
    
        let ally_text;
        let opponent_text;
        match action_choices {
            Some(choices) => {
                ally_text = format!["{:?}", choices.ally_team_choices];
                opponent_text = format!["{:?}",choices.opponent_team_choices];
            },
            None => {
                ally_text = String::new();
                opponent_text = String::new();
            },
        }

        // Ally Monster Stats Widget
        let ally_text = Span::raw(ally_text);
        let ally_widget = Paragraph::new(ally_text)
        .block(Block::default()
        .title(" Ally Team ")
        .style(Style::default().fg(Color::Blue))
        .borders(Borders::ALL)
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(ally_widget, chunks[0]);   


        // Message log widget
        let text = message_buffer
        .iter()
        .map(|element| { Spans::from(Span::raw(element))})
            .collect::<Vec<_>>();
        let paragraph_widget = Paragraph::new(text)
            .block(Block::default()
                .title(" Message Log ")
                .style(Style::default().fg(Color::Blue))
                .borders(Borders::ALL)
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph_widget, chunks[1]);   
        
        
        // Opponent Monster Stats Widget
        let opponent_text = Span::raw(opponent_text);
        let opponent_widget = Paragraph::new(opponent_text)
        .block(Block::default()
        .title(" Opponent Team ")
        .style(Style::default().fg(Color::Blue))
        .borders(Borders::ALL)
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(opponent_widget, chunks[2]);   
    })
    
}
enum TUIEvent<I> {
    Input(I),
    Tick,
}