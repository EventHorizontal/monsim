mod dsl_syntax;

use dsl_syntax::{BattleStateExpr, EffectType, MonsterExpr};

use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma};

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
    let context_expr = parse_macro_input!(input as BattleStateExpr);

    // Construct the streams of Tokens_______________________________________________________
    
    let BattleStateExpr { 
        ally_team_fields, opponent_team_fields 
    } = context_expr;
    
    let ally_monsters_vec = construct_team_token_stream(
        ally_team_fields, 
        quote!(Ally)
    );
    let opponent_monsters_vec = construct_team_token_stream(
        opponent_team_fields, 
        quote!(Opponent),
    );
    
    let entities = quote!(monsim::sim);
    let output = quote!({ 
            Battle::new(
                #entities::AllyBattlerTeam(#entities::BattlerTeam::new(#ally_monsters_vec)),
                #entities::OpponentBattlerTeam(#entities::BattlerTeam::new(#opponent_monsters_vec)),
            )
        }
    );
    
    // Return the final stream of Tokens ______________________________________________________
    output.into()
}

fn construct_team_token_stream<'a>(
    team_fields: Punctuated<MonsterExpr, Comma>,
    team_name_ident: TokenStream,
) -> TokenStream {

    let sim_ident = quote!(monsim::sim);
    
    let monster_mod = quote!(#sim_ident::monster);
    
    let move_mod = quote!(#sim_ident::move_);
    
    let ability_mod = quote!(#sim_ident::ability);

    let monster_iterator = team_fields.into_iter();

    let mut monsters = quote!();

    // Iterate through monsters
    for (index, monster) in monster_iterator.enumerate() {
        let monster_species = monster.monster_path.clone();
        let monster_nickname = monster.nickname_ident.unwrap_or(Literal::string(&format!("{}", 
        &monster.monster_path
            .path
            .segments
            .last()
            .expect("There should be at least one segment in the path to the monster.")
            .ident
            .to_string()
        )));
        let mut ability_species = quote!();
        let mut moves_vec = quote!();
        // Iterate through efects on monster
        for effect_expr in monster.fields.iter() {
            match effect_expr.effect_type {
                EffectType::Move => {
                    let move_ident = effect_expr
                        .effect_path
                        .path.segments
                        .last()
                        .expect("There should be at least one segment in the path to the move.")
                        .clone();
                    // Add to the moves array
                    moves_vec = quote!(
                        #moves_vec #move_mod::Move::new(#move_ident),
                    );
                },
                EffectType::Ability => {
                    let ability_ident = effect_expr.effect_path
                        .path.segments
                        .last()
                        .expect("There should be at least one segment in the path to the ability.")
                        .clone();
                    ability_species = quote!(#ability_ident);
                },
                EffectType::Item => todo!(),
            }
        }

        // Delimit the moves array with [] after we add all the elements
        moves_vec = quote!(vec![#moves_vec]);
        // Add to the monsters array
        let monster_number = quote!(BattlerNumber::from(#index));
        let is_first_monster = index == 0;
        monsters = quote!(
            #monsters 
            #sim_ident::Battler::new(
                #sim_ident::BattlerUID { team_id: #sim_ident::TeamID::#team_name_ident, battler_number: #monster_mod::#monster_number },
                #is_first_monster,
                #monster_mod::Monster::new(#monster_species, #monster_nickname),
                #move_mod::MoveSet::new(#moves_vec),
                #ability_mod::Ability::new(#ability_species),
            ),        
        );
    }

    // Return the `BattlerTeam` TokenStream
    quote!(vec![#monsters])
}
