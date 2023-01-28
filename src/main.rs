use monsim::*;
use tui;

fn main() {
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

    // Keep simulating turns until the battle is finished.
    while battle.context.state != BattleState::Finished {
        let user_input = UserInput::receive_input(&battle.context);
        let result = battle.simulate_turn(user_input);
        println!("{:?}\n", result);
        println!("\n-------------------------------------\n");
    }

    println!("The Battle ended with no errors.\n");
}
