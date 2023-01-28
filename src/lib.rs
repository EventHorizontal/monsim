pub mod battle_sim;

#[cfg(test)]
mod public_api {

    use crate::battle_sim::*;

    #[test]
    fn test_example_battle() {
        let mut battle = Battle::new(bcontext!(
            {
                AllyTeam {
                    mon Torchic "Ruby" {
                        mov Ember,
                        mov Scratch,
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

        assert_eq!(battle.simulate(), Ok(()));
    }
}
