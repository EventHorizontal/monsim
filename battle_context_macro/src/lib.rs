mod dsl_syntax;

use dsl_syntax::{BattleStateExpr, EffectType, MonsterExpr};

use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma};

/// This macro parses the following custom syntax:
/// ```
/// {
///     AllyTeam {
///         mon #MonsterNameHere {
///                 mov #MoveNameHere,
///                 //...up to 3 more
///                 abl #AbilityNameHere,
///                 itm #ItemNameHere, //(Not Implemented yet)
///             },
///         //...up to 5 more
///     },
///    OpponentTeam {
///         mon #MonsterNameHere {
///                 mov #MoveNameHere,
///                 //...up to 3 more
///                 abl #AbilityNameHere,
///                 itm #ItemNameHere, //(Not Implemented yet)
///             },
///         //...up to 5 more
///     }
/// }
/// ```
/// and produces a `battle::BattleContext`.
#[proc_macro]
pub fn battle_context(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the expression ________________________________________________________________
    let context_expr = parse_macro_input!(input as BattleStateExpr);

    // Construct the streams of Tokens_______________________________________________________
    
    let BattleStateExpr { 
        ally_team_fields, opponent_team_fields 
    } = context_expr;
    
    let ally_monsters_vec = construct_team_token_stream(
        quote!(monsim),
        ally_team_fields, 
        quote!(Ally)
    );
    let opponent_monsters_vec = construct_team_token_stream(
        quote!(monsim),
        opponent_team_fields, 
        quote!(Opponent),
    );
    
    let entities = quote!(monsim::game_mechanics);
    let output = quote!({ 
            extern crate self as monsim;
            BattleContext::new(
                monsim::game_mechanics::AllyBattlerTeam(#entities::BattlerTeam::new(#ally_monsters_vec)),
                monsim::game_mechanics::OpponentBattlerTeam(#entities::BattlerTeam::new(#opponent_monsters_vec)),
            )
        }
    );
    
    // Return the final stream of Tokens ______________________________________________________
    output.into()
}

fn construct_team_token_stream<'a>(
    package_ident: TokenStream,
    team_fields: Punctuated<MonsterExpr, Comma>,
    team_name_ident: TokenStream,
) -> TokenStream {

    let game_mechanics = quote!(#package_ident::game_mechanics);
    
    let monster_mod = quote!(#game_mechanics::monster);
    let monster_dex_mod = quote!(#game_mechanics::monster_dex);
    
    let move_mod = quote!(#game_mechanics::move_);
    let move_dex_mod = quote!(#game_mechanics::move_dex);
    
    let ability_mod = quote!(#game_mechanics::ability);
    let ability_dex_mod = quote!(#game_mechanics::ability_dex);

    let monster_iterator = team_fields.into_iter();

    let mut monsters = quote!();

    // Iterate through monsters
    for (index, monster) in monster_iterator.enumerate() {
        let monster_species = monster.monster_ident.clone();
        let monster_nickname = monster.nickname_ident.unwrap_or(Literal::string(&monster.monster_ident.to_string()));
        let mut ability_species = quote!();
        let mut moves_vec = quote!();
        // Iterate through efects on monster
        for effect_expr in monster.fields.iter() {
            match effect_expr.effect_type {
                EffectType::Move => {
                    let move_ident = effect_expr.effect_ident.clone();
                    // Add to the moves array
                    moves_vec = quote!(
                        #moves_vec #move_mod::Move::new(#move_dex_mod::#move_ident),
                    );
                },
                EffectType::Ability => {
                    let ability_ident = effect_expr.effect_ident.clone();
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
            #game_mechanics::Battler::new(
                #game_mechanics::BattlerUID { team_id: #game_mechanics::TeamID::#team_name_ident, battler_number: #monster_mod::#monster_number },
                #is_first_monster,
                #monster_mod::Monster::new(#monster_dex_mod::#monster_species, #monster_nickname),
                #move_mod::MoveSet::new(#moves_vec),
                #ability_mod::Ability::new(#ability_dex_mod::#ability_species),
            ),        
        );
    }

    // Return the `BattlerTeam` TokenStream
    quote!(vec![#monsters])
}
