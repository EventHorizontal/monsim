mod syntax;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, ExprTuple, Pat};

use syntax::battle_macro_syntax::{MonsterExpr, BattleExpr, MonsterTeamExpr};
use syntax::event_system_macro_syntax::ExprEventHandlerDeck;
use syntax::accessor_macro_syntax::ExprMechanicAccessor;

/// Shorthand for retrieving a `Monster` from a `Battle`. Currently requires a variable `battle` of type `Battle` to be in scope.
#[proc_macro]
pub fn monster(input: TokenStream) -> TokenStream {
    construct_accessor(input, quote!(monster))
}

/// Shorthand for retrieving a `Move` from a `Battle`. Currently requires a variable `battle` of type `Battle` to be in scope.
#[proc_macro]
pub fn move_(input: TokenStream) -> TokenStream {
    construct_accessor(input, quote!(move_))
}

/// Shorthand for retrieving an `Ability` from a `Battle`. Currently requires a variable `battle` of type `Battle` to be in scope.
#[proc_macro]
pub fn ability(input: TokenStream) -> TokenStream {
    construct_accessor(input, quote!(ability))
}

fn construct_accessor(input: TokenStream, accessor_name: TokenStream2) -> TokenStream {
    let ExprMechanicAccessor { is_mut, ident } = parse_macro_input!(input as ExprMechanicAccessor);
    let span = ident.span();
    if is_mut {
        // add "_mut" to the accessor
        let mut accessor = accessor_name.to_string();
        if accessor.ends_with("_") {
            accessor.push_str("mut");
        } else {
            accessor.push_str("_mut");
        }
        let suffixed_accessor = Ident::new(accessor.as_str(), span);
        quote!(battle.#suffixed_accessor(#ident)).into()
    } else {
        quote!(battle.#accessor_name(#ident)).into()
    }
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
pub fn generate_events(input: TokenStream) -> TokenStream {
    
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
            let attribute_name = attr.path().get_ident().expect("There should be an ident").to_string();
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

/// This macro parses the following syntax:
/// ```
/// team: Allies
/// {
///     *MonsterName*: "*OptionalMonsterNickname*" {
///         moveset: (*MoveName* { *optional_specifiers* }, ..0-3 more ),
///         ability: *AbilityName*
///     },
///     ..0-5 more
/// },
/// team: Opponents
/// {
///     *MonsterName*: "*OptionalMonsterNickname*" {
///         moveset: (*MoveName* { *optional_specifiers* }, ..0-3 more ),
///         ability: *AbilityName*
///     },
///     ..0-5 more
/// }
/// ```
/// and produces a `BattleState` with the given specifications.
#[proc_macro]
pub fn battle(input: TokenStream) -> TokenStream {
    let battle_expr = parse_macro_input!(input as BattleExpr);

    let (ally_team_expr, opponent_team_expr) = battle_expr.team_exprs();
    let get_team_tokens = |team_expr: MonsterTeamExpr, method_ident: TokenStream2| {
        let team_monster_tokens = team_expr.monster_exprs()
            .fold( quote!(), |mut tokens_so_far, monster_expr|  {
                    let MonsterExpr { monster_ident, maybe_nickname_literal, moveset_expr, ability_expr } = monster_expr;

                    let nickname_tokens = maybe_nickname_literal.map_or(quote!(), |nickname| { quote!(.with_nickname(#nickname)) });

                    let move_tokens = moveset_expr.move_exprs
                        .into_iter()
                        .fold(quote!(), |mut tokens_so_far, move_expr| {
                                let power_point_tokens = move_expr.maybe_power_points
                                    .clone()
                                    .map(|lit_int| {
                                        quote!(.with_power_points(#lit_int))
                                    });

                                tokens_so_far.extend(quote!(
                                    .add_move(
                                        Move::of_species(&#move_expr)
                                            #power_point_tokens
                                    )
                                )); 
                                tokens_so_far
                            }
                        );

                    let ability_tokens = quote!(.add_ability(&#ability_expr));

                    let monster_tokens = quote!(
                        .add_monster(Monster::of_species(&#monster_ident)
                            #nickname_tokens
                            #move_tokens
                            #ability_tokens
                        )
                    );
                    tokens_so_far.extend(monster_tokens);
                    tokens_so_far
                }
            );
        quote!(
            .#method_ident(
                MonsterTeam::builder()
                    #team_monster_tokens
            )
        )
    };
    let ally_team_tokens = get_team_tokens(ally_team_expr, quote!(add_ally_team));
    let opponent_team_tokens = get_team_tokens(opponent_team_expr, quote!(add_opponent_team));
    let output = quote!(
        BattleState::builder()
            #ally_team_tokens
            #opponent_team_tokens
            .build()
    );
    output.into()
}