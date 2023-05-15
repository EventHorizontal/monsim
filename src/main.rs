use std::{thread, time::{Duration, Instant}, sync::mpsc};

use monsim::*;
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, execute};
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, Paragraph, Wrap}, style::{Style, Color, Modifier}, layout::{Alignment}, text::{Spans, Span}};


const TUI_INPUT_POLL_TIMEOUT_MILLISECONDS: u64 = 20;
type MonsimIOResult = Result<(), Box<dyn std::error::Error>>; 

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

    loop {

        // Do appropriate thing based on input receieved
        match receiver.recv()? {
            TUIEvent::Input(event) => match event {
                // Quit
                KeyEvent { code, modifiers: _, kind: KeyEventKind::Release, state: _ } => {
                    if code == KeyCode::Esc {
                        disable_raw_mode()?;
                        execute!(std::io::stdout(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;
                        break;
                    // Keep simulating turns until the battle is finished.
                    } else if code == KeyCode::Char('n') && battle.context.state != BattleState::Finished {
                        battle.context.message_buffer.clear();
                        let user_input = UserInput::receive_input(&battle.context);
                        _ = battle.simulate_turn(user_input);
                    }
                },
                _ => {},
            },
            TUIEvent::Tick => {},
        }

        // Draw the result of the current turn to the terminal
        terminal.draw( |frame| {    
            let text = battle.context.message_buffer.iter().map(|element| { Spans::from(Span::raw(element))}).collect::<Vec<_>>();
            let paragraph_widget = Paragraph::new(text)
                .block(Block::default()
                    .title(Span::styled(" Monsim TUI ", Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)))
                    .style(Style::default().fg(Color::Blue))
                    .borders(Borders::ALL)
                )
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            frame.render_widget(paragraph_widget, frame.size());
        })?;
    }
    
    battle.context.message_buffer.clear();
    println!("The Battle ended with no errors.\n");
    Ok(())
}

enum TUIEvent<I> {
    Input(I),
    Tick,
}