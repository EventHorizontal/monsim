mod customsyntax;

use customsyntax::{BattleStateExpr, EffectType, MonsterExpr};

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Punct, Spacing};
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma};

/// This macro parses the following custom syntax:
/// ```
/// {
///     AllyTeam {
///         mon #MonsterNameHere {
///                 mov #MoveNameHere,
///                 //...up to 3 more
///                 abl #AbilityNameHere, //(Not Implemented yet)
///                 itm #ItemNameHere, //(Not Implemented yet)
///             },
///         //...up to 5 more
///     },
///    OpponentTeam {
///         mon #MonsterNameHere {
///                 mov #MoveNameHere,
///                 //...up to 3 more
///                 abl #AbilityNameHere, //(Not Implemented yet)
///                 itm #ItemNameHere, //(Not Implemented yet)
///             },
///         //...up to 5 more
///     }
/// }
/// ```
/// and produces a `battle::BattleState` struct and the final entity ID and effector ID that it
/// used up as a tuple. It is hardwired to start at the ID (0,0) and uses ID (0,0), ID (1,1) and
/// ID (2,2) for the Root, AllyTeam and OpponentTeam respectively.
#[proc_macro]
pub fn battle_state(input: TokenStream) -> TokenStream {
    // Parse the expression ________________________________________________________________
    let battle_state_expr = parse_macro_input!(input as BattleStateExpr);

    // Construct the streams of Tokens_______________________________________________________
    let mut entity_id = 1u8;
    let mut effector_id = 1u8;

    let mut monster_stream = quote! ( let mut monsters = vec! );

    let mut move_stream = quote! ( let mut moves = vec! );

    let (mut ally_monster_list, mut ally_move_list) =
        construct_streams_per_team(
            battle_state_expr.ally_team_fields,
            &mut entity_id,
            &mut effector_id,
            1,
        );

    let (opponent_monster_list, opponent_move_list) =
        construct_streams_per_team(
            battle_state_expr.opponent_team_fields,
            &mut entity_id,
            &mut effector_id,
            2,
        );

    // Concatenate streams of Tokens ______________________________________________________
    let mut output = quote!( use crate::battle; ); // init output stream
    concatenate(&mut ally_monster_list, opponent_monster_list);
    concatenate_delimited(&mut monster_stream, ally_monster_list, Delimiter::Bracket);
    concatenate(&mut output, monster_stream);
    output.append(Punct::new(';', Spacing::Alone)); // add a semicolon
    concatenate(&mut ally_move_list, opponent_move_list);
    concatenate_delimited(&mut move_stream, ally_move_list, Delimiter::Bracket);
    concatenate(&mut output, move_stream);
    output.append(Punct::new(';', Spacing::Alone)); // add a semicolon
    concatenate(
        &mut output,
        quote!(
            (
                battle::battle_state::BattleState::new( monsters, moves ),
                #entity_id,
                #effector_id
            )
        ),
    );
    let mut output2 = proc_macro2::TokenStream::new(); // init a second output stream
    concatenate_delimited(&mut output2, output, Delimiter::Brace);

    // Return the final stream of Tokens ______________________________________________________
    output2.into()
}

fn construct_streams_per_team<'a>(
    expr_tree: Punctuated<MonsterExpr, Comma>,
    entity_id: &mut u8,
    effector_id: &mut u8,
    _team_index: usize,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {

    let mut first = true;

    let monstertype = quote!(battle::entities::Monster);
    let monstersmod = quote!(battle::entities::monster_dex);
    let mut monster_list = proc_macro2::TokenStream::new();
    
    let movetype = quote!(battle::entities::Move);
    let movesmod = quote!(battle::entities::move_dex);
    let mut move_list = proc_macro2::TokenStream::new();

    for monster_expr in expr_tree.iter() {
        
        if first {
            first = false;
            let monster_name = monster_expr.monster_ident.clone();
            let monster_nickname = monster_expr.nickname_ident.clone();
            concatenate(
                &mut monster_list,
                quote!(#monstertype::from_species(
                    #monstersmod::#monster_name, 
                    #entity_id, 
                    stringify![#monster_nickname]).set_active(true),
                ),
            );
            *effector_id += 1;
            for effect_expr in monster_expr.fields.iter() {
                let effector_name = effect_expr.effect_ident.clone();
                match effect_expr.effect_type {
                    EffectType::Move => {
                        concatenate(
                            &mut move_list,
                            quote!(#movetype::from_species(
                                #movesmod::#effector_name, 
                                #entity_id, 
                                battle::entities::MoveID::First).set_active(true),
                            ),
                        );
                    }
                    EffectType::Ability => {
                        todo!()
                    }
                    EffectType::Item => {
                        todo!()
                    }
                }
                *effector_id += 1;
            }
            *entity_id += 1;
        } else {
            let monster_name = monster_expr.monster_ident.clone();
            let monster_nickname = monster_expr.nickname_ident.clone();
            concatenate(
                &mut monster_list,
                quote!(#monstertype::from_species(
                    #monstersmod::#monster_name, 
                    #entity_id, 
                    stringify![#monster_nickname]),
                ),
            );
            *effector_id += 1;
            for effect_expr in monster_expr.fields.iter() {
                let effector_name = effect_expr.effect_ident.clone();
                match effect_expr.effect_type {
                    EffectType::Move => {
                        concatenate(
                            &mut move_list,
                            quote!(#movetype::from_species(
                                #movesmod::#effector_name, 
                                #entity_id, 
                                monsim::battle::entities::MoveID::First),
                            ),
                        );
                    }
                    EffectType::Ability => {
                        todo!()
                    }
                    EffectType::Item => {
                        todo!()
                    }
                }
                *effector_id += 1;
            }
            *entity_id += 1;
        }
    }
    
    ( monster_list, move_list )
}

/// Concatenates `addend_stream` to the end of `base_stream`.
fn concatenate(
    base_stream: &mut proc_macro2::TokenStream,
    addend_stream: proc_macro2::TokenStream,
) {
    base_stream.extend(addend_stream.into_iter());
}

///Concatenates `addend_stream` to the end of `base_stream`, delimiting the `addend_stream` with `delimiter`.
fn concatenate_delimited(
    base_stream: &mut proc_macro2::TokenStream,
    addend_stream: proc_macro2::TokenStream,
    delimiter: Delimiter,
) {
    base_stream.append(Group::new(delimiter, addend_stream));
}
