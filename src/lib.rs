pub mod battle;

#[cfg(test)]
mod tests {
    use crate::battle::Battle;
    pub use battle_state_macro;
    use battle_state_macro::battle_state;
    
    #[test]
    fn run_demo_battle() {
        let (battle_state, _, _) = battle_state!(
            {
                AllyTeam {
                    mon Shroomish Shroomba {
                        mov Tackle
                    }
                },
                OpponentTeam {
                    mon Trapinch Trap {
                        mov Tackle
                    }
                }
            }
        );
        let mut battle = Battle::new(battle_state);
        battle.run_sim();
    }
}