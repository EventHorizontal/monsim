mod dsl_syntax;

use dsl_syntax::{ExprBattle, ExprBattlerTeam, GameMechanicType, path_to_ident};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::parse_macro_input;

/// This macro parses the following custom syntax:
/// ```
/// {
///     AllyTeam {
///         let MonsterNameHere: Monster = OptionalNameStr {
///                 MoveNameHere: Move,
///                 //...up to 3 more
///                 AbilityNameHere: Ability,
///                 ItemNameHere: Item, //(Not Implemented yet)
///             },
///         //...up to 5 more
///     },
///    OpponentTeam {
///         let MonsterNameHere: Monster = OptionalNameStr {
///                 MoveNameHere: Move,
///                 //...up to 3 more
///                 AbilityNameHere: Ability,
///                 ItemNameHere: Item, //(Not Implemented yet)
///             },
///         //...up to 5 more
///     }
/// }
/// ```
/// and produces a `battle::Battle`.
#[proc_macro]
pub fn build_battle(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the expression ________________________________________________________________
    let context_expr = parse_macro_input!(input as ExprBattle);

    // Construct the streams of Tokens_______________________________________________________
    
    let ExprBattle { 
        ally_expr_battler_team, 
        opponent_expr_battler_team 
    } = context_expr;

    let ally_team_type = ally_expr_battler_team.team_type.clone();
    let opponent_team_type = opponent_expr_battler_team.team_type.clone();
    
    let ally_battlers_vec = battler_team_to_tokens(
        ally_expr_battler_team
    );
    let opponent_battlers_vec = battler_team_to_tokens(
        opponent_expr_battler_team
    );
    
    let output = quote!(
        { 
            monsim::sim::Battle::new(
                monsim::sim::AllyBattlerTeam(#ally_team_type::new(#ally_battlers_vec)),
                monsim::sim::OpponentBattlerTeam(#opponent_team_type::new(#opponent_battlers_vec)),
            )
        }
    );
    
    // Return the final stream of Tokens ______________________________________________________
    output.into()
}

fn battler_team_to_tokens<'a>(
    expr_battler_team: ExprBattlerTeam
) -> TokenStream {
    let team_name_ident = expr_battler_team.team_path;
    let sim_ident = quote!(monsim::sim);
    let move_mod = quote!(#sim_ident::move_);
    let mut comma_separated_battlers = quote!();
    
    // Iterate through monsters
    for (index, monster) in expr_battler_team.monster_fields.into_iter().enumerate() {
        let monster_species = monster.monster_instance_path.clone();
        let monster_nickname = monster.nickname_literal.unwrap_or(Literal::string(
            path_to_ident(&monster.monster_instance_path).to_string().as_str()
        ));
        let mut ability_type_path = None;
        let mut ability_species = quote!();
        let mut moves_vec_delimited = quote!();
        
        // Iterate through game_mechanics on monster
        for game_mechanic_expr in monster.fields.iter() {
            match game_mechanic_expr.game_mechanic_type {
                GameMechanicType::Move => {
                    let move_ident = path_to_ident(&game_mechanic_expr.game_mechanic_instance_path);
                    let move_type_path = game_mechanic_expr.game_mechanic_type_path.clone();
                    // Add to the moves array
                    moves_vec_delimited = quote!(
                        #moves_vec_delimited #move_type_path::new(#move_ident),
                    );
                },
                GameMechanicType::Ability => {
                    let ability_ident = path_to_ident(&game_mechanic_expr.game_mechanic_instance_path);
                    ability_species = quote!(#ability_ident);
                    ability_type_path = Some(game_mechanic_expr.game_mechanic_type_path.clone());
                },
                GameMechanicType::Item => todo!("Items have not been implemented yet in the engine."),
            }
        }

        moves_vec_delimited = quote!(vec![#moves_vec_delimited]);
        let monster_number = quote!(BattlerNumber::from(#index));
        let monster_type_path = monster.monster_type_path;
        let ability_type_path = ability_type_path.expect("Every monster must have an ability.");
        
        comma_separated_battlers = quote!(
            #comma_separated_battlers 
            #sim_ident::Battler::new(
                #sim_ident::BattlerUID { team_id: #sim_ident::TeamID::#team_name_ident, battler_number: #sim_ident::#monster_number },
                #monster_type_path::new(#monster_species, #monster_nickname),
                #move_mod::MoveSet::new(#moves_vec_delimited),
                #ability_type_path::new(#ability_species),
            ),        
        );
    }

    quote!(vec![#comma_separated_battlers])
}

