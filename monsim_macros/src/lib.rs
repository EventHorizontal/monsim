mod syntax;

use convert_case::Casing;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::parse_macro_input;

use syntax::battle_macro_syntax::{MonsterExpr, BattleExpr, MonsterTeamExpr};
use syntax::event_system_macro_syntax::{EventExpr, EventListExpr};
use syntax::accessor_macro_syntax::MechanicAccessorExpr;

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
    let MechanicAccessorExpr { is_mut, ident } = parse_macro_input!(input as MechanicAccessorExpr);
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
        quote!(entities.#suffixed_accessor(#ident)).into()
    } else {
        quote!(entities.#accessor_name(#ident)).into()
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

    let EventListExpr { event_exprs } = parse_macro_input!(input as EventListExpr);

    let mut event_handler_storage_tokens = quote![];
    let mut event_handler_impl_new_tokens = quote![];
    let mut trait_impl_block_tokens = quote![];
    let mut trait_enum_tokens = quote![];
    let mut event_handler_deck_field_tokens = quote![];
    let mut event_handler_deck_defaults_tokens = quote![];
    let mut event_id_iterator_tokens = quote![];

    for event_expr in event_exprs {
        let EventExpr { event_name_pascal_case, event_context_type_name_pascal_case, event_return_type_name } = event_expr;
        let event_name_snake_case = event_name_pascal_case.to_string().to_case(convert_case::Case::Snake);
        let event_name_snake_case = Ident::new(&event_name_snake_case, event_name_pascal_case.span());

        event_handler_storage_tokens.extend(quote!(
            pub #event_name_snake_case: Vec<OwnedEventHandler<#event_return_type_name, #event_context_type_name_pascal_case>>,
        ));

        event_handler_impl_new_tokens.extend(quote!(
            #event_name_snake_case: vec![],
        ));

        trait_impl_block_tokens.extend(quote![
            
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            /// #event_name_in_pascal_case(broadcaster, context)
            pub struct #event_name_pascal_case;
        
            impl Event for #event_name_pascal_case {
                
                type EventResult = #event_return_type_name;
                type Context = #event_context_type_name_pascal_case;
                
                fn corresponding_handlers<'a>(&self, event_handler_storage: &'a EventHandlerStorage) -> &'a Vec<OwnedEventHandler<Self::EventResult, Self::Context>> {
                    &event_handler_storage.#event_name_snake_case
                }
                
                fn corresponding_handlers_mut<'a>(&self, event_handler_storage: &'a mut EventHandlerStorage) -> &'a mut Vec<OwnedEventHandler<Self::EventResult, Self::Context>> {
                    &mut event_handler_storage.#event_name_snake_case
                }
        
                fn uid(&self) -> EventID {
                    EventID::#event_name_pascal_case
                }
            }

        ]);

        trait_enum_tokens.extend(quote![
            #event_name_pascal_case,
        ]);

        event_handler_deck_field_tokens.extend(quote![
            pub #event_name_snake_case: Option<EventCallback<#event_return_type_name, #event_context_type_name_pascal_case>>,
        ]);

        event_handler_deck_defaults_tokens.extend(quote![
            #event_name_snake_case: None,
        ]);
        
        event_id_iterator_tokens.extend(quote![
            EventID::#event_name_pascal_case,    
        ]);

    }

    let output = quote![
        #[derive(Debug, Clone)]
        pub struct EventHandlerStorage {
            #event_handler_storage_tokens
        }

        impl EventHandlerStorage {
            pub const fn new() -> Self {
                Self {
                    #event_handler_impl_new_tokens
                }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct EventHandlerDeck {
            #event_handler_deck_field_tokens
        }

        impl EventHandlerDeck {
            pub const fn const_default() -> EventHandlerDeck {
                EventHandlerDeck {
                    #event_handler_deck_defaults_tokens
                }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub enum EventID {
            #trait_enum_tokens
        }

        impl EventID {
            pub fn all() -> impl Iterator<EventID> {
                [
                    #event_id_iterator_tokens
                ].into_iter()
            }
        }
        
        pub mod event_dex {
            use super::{MonsterUID, contexts::*, Event, EventHandlerStorage, OwnedEventHandler, EventID, Outcome, Nothing, Percent};
            #trait_impl_block_tokens
        }
    ];

    output.into()
    
    // let event_handler_type = quote!(EventHandler);
    
    // let ExprEventHandlerDeck { 
    //     doc_comment, 
    //     first_pub_keyword, 
    //     struct_keyword, 
    //     struct_name, 
    //     match_expr, 
    //     const_keyword, 
    //     default_handler_constant_name, 
    //     default_handler_value, 
    //     second_pub_keyword, 
    //     trait_keyword, 
    //     trait_name 
    // }: ExprEventHandlerDeck = parse_macro_input!(input);

    // let mut fields = quote!();
    // let mut fields_for_constant = quote!();
    // let mut events = quote!();
    // let mut macro_fields = quote!();

    // for expression in match_expr.arms {
    //     let mut comments = quote!();
    //     let mut maybe_context_type = None;
    //     for attr in expression.attrs {
    //         let attribute_name = attr.path().get_ident().expect("There should be an ident").to_string();
    //         if attribute_name == "doc" {
    //             comments = quote!(
    //                 #comments
    //                 #attr
    //             );
    //         } else if attribute_name == "context" {
    //             let type_token = attr.parse_args::<ExprTuple>();
    //             match type_token {
    //                 Ok(type_token) => maybe_context_type = Some(quote!(#type_token)),
    //                 Err(_) => {
    //                     let type_token = attr.parse_args::<Ident>().expect("Context must be a type or `None`");
    //                     if type_token.to_string() == "None" {
    //                         maybe_context_type = Some(quote!(()));
    //                     } else {
    //                         maybe_context_type = Some(quote!(#type_token));
    //                     }
    //                 },
    //             }
    //         } else {
    //             panic!("Only doc comment and `context` attributes are allowed in this macro.")
    //         }
    //     }
    //     let context_type = match maybe_context_type {
    //         Some(tokens) => tokens,
    //         None => panic!("A context must be specified for each field."),
    //     };
    //     let handler_ident = expression.pat;
    //     let handler_return_type = *expression.body;
        
    //     let pat_ident = match handler_ident {
    //         Pat::Ident( ref pat_ident) => pat_ident.clone(),
    //         _ => panic!("Error: Expected handler_ident to be an identifier."),
    //     };
    //     let trait_name_string_in_pascal_case = to_pascal_case(pat_ident.clone().ident.to_string());
    //     let event_trait_literal = Literal::string(&trait_name_string_in_pascal_case);
    //     let event_name_ident_in_pascal_case = Ident::new(
    //         &trait_name_string_in_pascal_case,
    //         pat_ident.ident.span(),
    //     );
    //         fields = quote!( 
    //             #fields
    //             #comments
    //             pub #handler_ident: Option<#event_handler_type<#handler_return_type, #context_type>>,
    //         );
    //         fields_for_constant = quote!(
    //             #fields_for_constant
    //             #handler_ident: #default_handler_value,
    //         );
    //         events = quote!(
    //             #events

    //             #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    //             pub struct #event_name_ident_in_pascal_case;

    //             impl #trait_name for #event_name_ident_in_pascal_case {
    //                 type EventReturnType = #handler_return_type;
    //                 type ContextType = #context_type;
    //                 fn corresponding_handler(&self, event_handler_deck: &#struct_name) -> Option<#event_handler_type<Self::EventReturnType, Self::ContextType>> {
    //                     event_handler_deck.#handler_ident
    //                 }
        
    //                 fn name(&self) -> &'static str {
    //                     #event_trait_literal
    //                 }
    //             }
    //         );

    //         macro_fields = quote!(
    //             #macro_fields
    //             (stringify![#event_name_ident_in_pascal_case]) => { #handler_ident }
    //         )
    // }

    // let output_token_stream = quote!(
    //     #doc_comment
    //     #[derive(Debug, Clone, Copy)]
    //     #first_pub_keyword #struct_keyword #struct_name {
    //         #fields
    //     }

    //     #const_keyword #default_handler_constant_name: #struct_name = #struct_name {
    //         #fields_for_constant
    //     };

    //     #second_pub_keyword #trait_keyword #trait_name: Clone + Copy {
    //         type EventReturnType: Sized + Clone + Copy;
    //         type ContextType: Sized + Clone + Copy;

    //         fn corresponding_handler(
    //             &self,
    //             event_handler_deck: &#struct_name,
    //         ) -> Option<#event_handler_type<Self::EventReturnType, Self::ContextType>>;

    //         fn name(&self) -> &'static str;
    //     }

    //     #[macro_export]
    //     macro_rules! corresponding_handler {
    //         ($x: expr) => {
    //             match stringify![$x] {
    //                 #macro_fields
    //             }
    //         }            
    //     }

    //     pub mod event_dex {
    //         use super::*;

    //         #events
    //     }
    // );
    // output_token_stream.into()
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

                    let mut number_of_moves = 0;
                    let move_tokens = moveset_expr.move_exprs
                        .into_iter()
                        .fold(quote!(), |mut tokens_so_far, move_expr| {
                                number_of_moves += 1;
                                let power_point_tokens = move_expr.maybe_power_points
                                    .clone()
                                    .map(|lit_int| {
                                        quote!(.with_power_points(#lit_int))
                                    });

                                if number_of_moves > 1 {
                                    tokens_so_far.extend(quote!(
                                        Some(#move_expr.spawn()
                                            #power_point_tokens),
                                    ));    
                                } else {
                                    tokens_so_far.extend(quote!(
                                        #move_expr.spawn()
                                            #power_point_tokens,
                                    )); 
                                }
                                tokens_so_far
                            }
                        );
                    let empty_moveslot_tokens = (number_of_moves+1..=4).fold(
                        quote!(),
                        |mut tokens_so_far, _| {
                            tokens_so_far.extend(quote![None,]);
                            tokens_so_far
                        }
                    );
                            
                    let ability_tokens = quote!(#ability_expr.spawn());

                    let monster_tokens = quote!(
                        .add_monster(#monster_ident.spawn(
                                (
                                    #move_tokens
                                    #empty_moveslot_tokens
                                ),
                                #ability_tokens
                            )
                            #nickname_tokens
                        )
                    );
                    tokens_so_far.extend(monster_tokens);
                    tokens_so_far
                }
            );
        quote!(
            .#method_ident(
                MonsterTeam::spawn()
                    #team_monster_tokens
            )
        )
    };
    let ally_team_tokens = get_team_tokens(ally_team_expr, quote!(add_ally_team));
    let opponent_team_tokens = get_team_tokens(opponent_team_expr, quote!(add_opponent_team));
    let output = quote!(
        BattleState::spawn()
            #ally_team_tokens
            #opponent_team_tokens
            .build()
    );
    output.into()
}