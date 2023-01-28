mod customsyntax;

use customsyntax::{BattleStateExpr, EffectType, MonsterExpr};

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
pub fn bcontext(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    
    // Parse the expression ________________________________________________________________
    let context_expr = parse_macro_input!(input as BattleStateExpr);

    // Construct the streams of Tokens_______________________________________________________
    
    let BattleStateExpr { 
        ally_team_fields, opponent_team_fields 
    } = context_expr;
    
    let ally_monsters = construct_streams_per_team(
        quote!(monsim),
        ally_team_fields, 
        quote!(Ally)
    );
    let opponent_monsters = construct_streams_per_team(
        quote!(monsim),
        opponent_team_fields, 
        quote!(Opponent),
    );

    let entities = quote!(monsim::game_mechanics);
    let output = quote!( 
        BattleContext::new(
            #entities::MonsterTeam::new([#ally_monsters]),
            #entities::MonsterTeam::new([#opponent_monsters]),
        )
    );
    
    // Return the final stream of Tokens ______________________________________________________
    output.into()
}

#[proc_macro]
pub fn bcontext_internal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    
    // Parse the expression ________________________________________________________________
    let context_expr = parse_macro_input!(input as BattleStateExpr);

    // Construct the streams of Tokens_______________________________________________________
    
    let BattleStateExpr { 
        ally_team_fields, opponent_team_fields 
    } = context_expr;
    
    let ally_monsters = construct_streams_per_team(
        quote!(crate),
        ally_team_fields, 
        quote!(Ally)
    );
    let opponent_monsters = construct_streams_per_team(
        quote!(crate),
        opponent_team_fields, 
        quote!(Opponent),
    );
    
    let entities = quote!(crate::game_mechanics);
    let output = quote!( 
        BattleContext::new(
            #entities::MonsterTeam::new([#ally_monsters]),
            #entities::MonsterTeam::new([#opponent_monsters]),
        )
    );
    
    // Return the final stream of Tokens ______________________________________________________
    output.into()
}

fn construct_streams_per_team<'a>(
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

    let monster_count = team_fields.len();
    let monster_iterator = team_fields.into_iter();

    let mut monsters = quote!();

    // Iterate through monsters
    for (index, monster) in monster_iterator.enumerate() {
        let monster_species = monster.monster_ident.clone();
        let monster_nickname = monster.nickname_ident.unwrap_or(Literal::string(&monster.monster_ident.to_string()));
        let mut ability_species = quote!();
        let mut moves_array = quote!();
        // Iterate through efects on monster
        for effect_expr in monster.fields.iter() {
            match effect_expr.effect_type {
                EffectType::Move => {
                    let move_ident = effect_expr.effect_ident.clone();
                    // Add to the moves array
                    moves_array = quote!(
                        #moves_array Some(#move_mod::Move::new(#move_dex_mod::#move_ident)),
                    );
                },
                EffectType::Ability => {
                    let ability_ident = effect_expr.effect_ident.clone();
                    ability_species = quote!(#ability_ident);
                },
                EffectType::Item => todo!(),
            }
        }
        // Fill in the rest of the MoveSet with None
        for _ in monster.move_count..4 {
            moves_array = quote!(#moves_array None,);
        }
        // Delimit the moves array with [] after we add all the elements
        moves_array = quote!([#moves_array]);
        // Add to the monsters array
        let monster_number = map_usize_to_monster_number_ident(index);
        let is_first_monster = index == 0;
        monsters = quote!(
            #monsters 
            Some(#game_mechanics::Battler::new(
                #game_mechanics::BattlerUID { team_id: #game_mechanics::TeamID::#team_name_ident, battler_number: #monster_mod::#monster_number },
                #is_first_monster,
                #monster_mod::Monster::new(#monster_dex_mod::#monster_species, #monster_nickname),
                #move_mod::MoveSet::new(#moves_array),
                #ability_mod::Ability::new(#ability_dex_mod::#ability_species),
            )),        
        );
    }

    // Fill the rest of the MonsterTeam with None
    for _ in monster_count..6 {
        monsters = quote!(#monsters None,)
    }

    // Return the `MonsterTeam` TokenStream
    monsters
}

#[inline(always)]
fn map_usize_to_monster_number_ident(number: usize) -> TokenStream {
    match number {
        0 => quote!(BattlerNumber::First),
        1 => quote!(BattlerNumber::Second),
        2 => quote!(BattlerNumber::Third),
        3 => quote!(BattlerNumber::Fourth),
        4 => quote!(BattlerNumber::Fifth),
        5 => quote!(BattlerNumber::Sixth),
        _ => panic!("You cannot have more than 6 Monsters.")
    }
}
