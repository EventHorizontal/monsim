mod syntax;

use proc_macro::TokenStream;
use syntax::{path_to_ident, ExprBattle, ExprEventHandlerDeck, ExprMonsterTeam, GameMechanicType};
use proc_macro2::{Ident, Literal};
use quote::quote;
use syn::{parse_macro_input, ExprTuple, ExprType, Pat};

// BattleState generation macro ------

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
/// and produces a `battle::BattleState`.
#[proc_macro]
pub fn battle_state(input: TokenStream) -> TokenStream {
    // Parse the expression ________________________________________________________________
    let context_expr = parse_macro_input!(input as ExprBattle);

    // Construct the streams of Tokens_______________________________________________________
    
    let ExprBattle { 
        ally_expr_monster_team, 
        opponent_expr_monster_team 
    } = context_expr;

    let ally_team_type = ally_expr_monster_team.team_type.clone();
    let opponent_team_type = opponent_expr_monster_team.team_type.clone();
    
    let ally_monsters_vec = monster_team_to_tokens(
        ally_expr_monster_team
    );
    let opponent_monsters_vec = monster_team_to_tokens(
        opponent_expr_monster_team
    );
    
    let output = quote!(
        { 
            monsim::sim::BattleState::new(
                monsim::sim::PerTeam::new(monsim::sim::Ally(#ally_team_type::new(#ally_monsters_vec, TeamUID::Allies)),
                monsim::sim::Opponent(#opponent_team_type::new(#opponent_monsters_vec, TeamUID::Opponents)))
            )
        }
    );
    
    // Return the final stream of Tokens ______________________________________________________
    output.into()
}

fn monster_team_to_tokens<'a>(
    expr_monster_team: ExprMonsterTeam
) -> proc_macro2::TokenStream {
    let team_name_ident = expr_monster_team.team_path;
    let sim_ident = quote!(monsim::sim);
    let move_mod = quote!(#sim_ident::move_);
    let mut comma_separated_monsters = quote!();
    
    // Iterate through monsters
    for (index, monster) in expr_monster_team.monster_fields.into_iter().enumerate() {
        let monster_species = monster.monster_instance_path.clone();
        let monster_nickname = monster.nickname_literal;
        let monster_nickname = if monster_nickname.is_some() { quote!(Some(#monster_nickname))} else { quote!(None) };
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
        let monster_number = quote!(MonsterNumber::from(#index));
        let ability_type_path = ability_type_path.expect("Every monster must have an ability.");
        
        comma_separated_monsters = quote!(
            #comma_separated_monsters 
            #sim_ident::Monster::new(
                #sim_ident::MonsterUID { team_uid: #sim_ident::TeamUID::#team_name_ident, monster_number: #sim_ident::#monster_number },
                #monster_species, 
                #monster_nickname,
                #move_mod::MoveSet::new(#moves_vec_delimited),
                #ability_type_path::new(#ability_species),
            ),        
        );
    }

    quote!(vec![#comma_separated_monsters])
}

// Event system macros ------

// Generates the struct `CollectionType`, the default constant and the `TraitName` trait plus 
/// implementations for each event, when given a list of event identifiers. The syntax for this 
/// is as follows
/// ```
/// pub struct CollectionType {
/// match event {
///         /// Possible documentation for event_1
///         #[context(<ContextType>)]
///         event_name_1 => <EventReturnType>,
///         ...
///         /// Possible documentation for event_n
///         #[context(<ContextType>)]
///         event_name_n => <EventReturnType>,
///     }
/// }
/// pub const CONSTANT_NAME;
/// pub trait TraitName;  
/// ```
#[proc_macro]
pub fn generate_events(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    
    let event_handler_type = quote!(EventHandler);
    
    let ExprEventHandlerDeck { 
        doc_comment, 
        first_pub_keyword, 
        struct_keyword, 
        struct_name, 
        match_expr, 
        const_keyword, 
        default_handler_constant_name, 
        default_handler_value, 
        second_pub_keyword, 
        trait_keyword, 
        trait_name 
    }: ExprEventHandlerDeck = parse_macro_input!(input);

    let mut fields = quote!();
    let mut fields_for_constant = quote!();
    let mut events = quote!();
    let mut macro_fields = quote!();

    for expression in match_expr.arms {
        let mut comments = quote!();
        let mut maybe_context_type = None;
        for attr in expression.attrs {
            let attribute_name = attr.path.get_ident().expect("There should be an ident").to_string();
            if attribute_name == "doc" {
                comments = quote!(
                    #comments
                    #attr
                );
            } else if attribute_name == "context" {
                let type_token = attr.parse_args::<ExprTuple>();
                match type_token {
                    Ok(type_token) => maybe_context_type = Some(quote!(#type_token)),
                    Err(_) => {
                        let type_token = attr.parse_args::<Ident>().expect("Context must be a type or `None`");
                        if type_token.to_string() == "None" {
                            maybe_context_type = Some(quote!(()));
                        } else {
                            maybe_context_type = Some(quote!(#type_token));
                        }
                    },
                }
            } else {
                panic!("Only doc comment and `context` attributes are allowed in this macro.")
            }
        }
        let context_type = match maybe_context_type {
            Some(tokens) => tokens,
            None => panic!("A context must be specified for each field."),
        };
        let handler_ident = expression.pat;
        let handler_return_type = *expression.body;
        
        let pat_ident = match handler_ident {
            Pat::Ident( ref pat_ident) => pat_ident.clone(),
            _ => panic!("Error: Expected handler_ident to be an identifier."),
        };
        let trait_name_string_in_pascal_case = to_pascal_case(pat_ident.clone().ident.to_string());
        let event_trait_literal = Literal::string(&trait_name_string_in_pascal_case);
        let event_name_ident_in_pascal_case = Ident::new(
            &trait_name_string_in_pascal_case,
            pat_ident.ident.span(),
        );
            fields = quote!( 
                #fields
                #comments
                pub #handler_ident: Option<#event_handler_type<#handler_return_type, #context_type>>,
            );
            fields_for_constant = quote!(
                #fields_for_constant
                #handler_ident: #default_handler_value,
            );
            events = quote!(
                #events

                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub struct #event_name_ident_in_pascal_case;

                impl #trait_name for #event_name_ident_in_pascal_case {
                    type EventReturnType = #handler_return_type;
                    type ContextType = #context_type;
                    fn corresponding_handler(&self, event_handler_deck: &#struct_name) -> Option<#event_handler_type<Self::EventReturnType, Self::ContextType>> {
                        event_handler_deck.#handler_ident
                    }
        
                    fn name(&self) -> &'static str {
                        #event_trait_literal
                    }
                }
            );

            macro_fields = quote!(
                #macro_fields
                (stringify![#event_name_ident_in_pascal_case]) => { #handler_ident }
            )
    }

    let output_token_stream = quote!(
        #doc_comment
        #[derive(Debug, Clone, Copy)]
        #first_pub_keyword #struct_keyword #struct_name {
            #fields
        }

        #const_keyword #default_handler_constant_name: #struct_name = #struct_name {
            #fields_for_constant
        };

        #second_pub_keyword #trait_keyword #trait_name: Clone + Copy {
            type EventReturnType: Sized + Clone + Copy;
            type ContextType: Sized + Clone + Copy;

            fn corresponding_handler(
                &self,
                event_handler_deck: &#struct_name,
            ) -> Option<#event_handler_type<Self::EventReturnType, Self::ContextType>>;

            fn name(&self) -> &'static str;
        }

        #[macro_export]
        macro_rules! corresponding_handler {
            ($x: expr) => {
                match stringify![$x] {
                    #macro_fields
                }
            }            
        }

        pub mod event_dex {
            use super::*;

            #events
        }
    );
    output_token_stream.into()
}

fn to_pascal_case(input_string: String) -> String {
    let mut output_string = String::new();
    let mut previous_char = None;
    for char in input_string.chars() {
        if let Some(previous_char) = previous_char {
            if previous_char == '_' {
                output_string.push(char.to_ascii_uppercase());
            } else {
                output_string.push(char);
            }
        } else {
            output_string.push(char.to_ascii_uppercase());
        }
        previous_char = Some(char);
    }
    output_string.replace("_", "")
}

/// A small macro that allows one to annotate a type. Meant to be used for ambiguous returns.
#[proc_macro]
pub fn returns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ExprType);
    let type_ = input.ty;
    quote!(#type_).into()
}