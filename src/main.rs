use std::{thread, time::{Duration, Instant}, sync::mpsc};

use monsim::*;
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, execute};
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, BorderType}, style::{Style, Color}, layout::{Layout, Direction, Rect}};


const TUI_TICK_RATE_MILLISECONDS: u64 = 20;
type MonsimIOResult = Result<(), Box<dyn std::error::Error>>; 

fn main() -> MonsimIOResult {
    let _battle = Battle::new(bcontext!(
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
    let tick_rate = Duration::from_millis(TUI_TICK_RATE_MILLISECONDS);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let Event::Key(key) = event::read().expect("can read events") {
                    sender.send(TUIEvent::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
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
        terminal.draw( |frame| {
            let size  = frame.size();
                
            let background_widget = Block::default()
                .title(" MonsimTUI ")
                .style(Style::default().fg(Color::Black).bg(Color::White))
                .borders(Borders::ALL)
                .border_type(BorderType::Plain);
            frame.render_widget(background_widget, size);
        })?;

        match receiver.recv()? {
            TUIEvent::Input(event) => match event {
                // Quit
                KeyEvent { code, modifiers, kind: KeyEventKind::Release, state } => {
                    if code == KeyCode::Esc {
                        disable_raw_mode()?;
                        execute!(std::io::stdout(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;
                        break;
                    }
                },
                _ => {},
            },
            TUIEvent::Tick => {},
        }
    }
    
    // // Keep simulating turns until the battle is finished.
    // while battle.context.state != BattleState::Finished {
    //     let user_input = UserInput::receive_input(&battle.context);
    //     let result = battle.simulate_turn(user_input);
    //     println!("{:?}\n", result);
    //     println!("\n-------------------------------------\n");
    // }
    
    println!("The Battle ended with no errors.\n");
    Ok(())
}

enum TUIEvent<I> {
    Input(I),
    Tick,
}