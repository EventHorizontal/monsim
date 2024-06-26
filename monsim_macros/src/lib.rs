mod syntax;

#[cfg(feature = "event_gen")]
use convert_case::Casing;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitStr, Type};

#[cfg(feature = "battle_builder")]
use syntax::battle_macro_syntax::{BattleExpr, MonsterExpr, MonsterTeamExpr};
use syntax::entity_fetcher_macro_syntax::ExprEntityFetcher;
#[cfg(feature = "event_gen")]
use syntax::event_system_macro_syntax::{EventExpr, EventListExpr};

#[cfg(feature = "entity_fetchers")]
/// Shorthand for fetching an `Monster` from the Simulator. **Note** with the current implementation, this requires a `BattleSimulator` to be in scope within the identifier `sim`.
#[proc_macro]
pub fn mon(input: TokenStream) -> TokenStream {
    construct_accessor(input, quote!(monster))
}

#[cfg(feature = "entity_fetchers")]
/// Shorthand for fetching an `Move` from the Simulator. **Note** with the current implementation, this requires a `BattleSimulator` to be in scope within the identifier `sim`.
#[proc_macro]
pub fn mov(input: TokenStream) -> TokenStream {
    construct_accessor(input, quote!(move_))
}

#[cfg(feature = "entity_fetchers")]
/// Shorthand for fetching an `Ability` from the Simulator. **Note** with the current implementation, this requires a `BattleSimulator` to be in scope within the identifier `sim`.
#[proc_macro]
pub fn abl(input: TokenStream) -> TokenStream {
    construct_accessor(input, quote!(ability))
}

#[cfg(feature = "entity_fetchers")]
fn construct_accessor(input: TokenStream, accessor_name: TokenStream2) -> TokenStream {
    let ExprEntityFetcher { is_mut, path_to_entity, span } = parse_macro_input!(input as ExprEntityFetcher);
    if is_mut {
        // add "_mut" to the accessor
        let mut accessor = accessor_name.to_string();
        if accessor.ends_with("_") {
            accessor.push_str("mut");
        } else {
            accessor.push_str("_mut");
        }
        let suffixed_accessor = Ident::new(accessor.as_str(), span);
        quote!(battle.#suffixed_accessor(#path_to_entity)).into()
    } else {
        quote!(battle.#accessor_name(#path_to_entity)).into()
    }
}

// Event system macros ------

#[proc_macro_derive(Event, attributes(returns, context, handler, no_broadcaster))]
pub fn event_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attributes = input.attrs;
    let mut return_type = quote![()];
    let mut context = quote![()];
    let mut handler = quote![k3jrkejrke3lr];
    let mut broadcaster = quote![];
    for attribute in attributes {
        if let Some(attribute_name) = attribute.path().get_ident() {
            let attribute_name = attribute_name.to_string();
            if attribute_name == "returns" {
                let return_tokens = attribute.parse_args::<Type>().unwrap();
                return_type = quote![#return_tokens];
            } else if attribute_name == "context" {
                let context_tokens = attribute.parse_args::<Type>().unwrap();
                context = quote![#context_tokens];
            } else if attribute_name == "handler" {
                let handler_tokens = attribute.parse_args::<Ident>().unwrap();
                handler = quote![#handler_tokens]
            } else if attribute_name == "no_broadcaster" {
                broadcaster = quote![, ()]
            }
        }
    }
    let name = input.ident;
    let name_literal = LitStr::new(name.to_string().as_str(), name.span());
    quote!(
        impl Event<#context, #return_type #broadcaster> for #name {
            #[cfg(feature = "debug")]
            fn name(&self) -> &'static str {
                #name_literal
            }

            fn get_event_handler_with_receiver<M: MechanicID>(
                &self,
                event_listener: &'static dyn EventListener<M>,
            ) -> Option<EventHandler<#context, #return_type, M, MonsterID #broadcaster>> {
                event_listener.#handler()
            }

            fn get_event_handler_without_receiver<M: MechanicID>(
                &self,
                event_listener: &'static dyn EventListener<M, Nothing>,
            ) -> Option<EventHandler<#context, #return_type, M, Nothing #broadcaster>> {
                event_listener.#handler()
            }
        }
    )
    .into()
}

/// Generates a bunch of stuff that is pertaining to the individual events that would be a pain
/// to write by hand. Currently that includes a struct called `EventHandlerDeck`, a constant which
/// represents an empty `EventHandlerDeck` called `DEFAULT_EVENT_HANDLERS` and the individual
/// implementations of `Event` for each of the event structs.
/// ```
///     event event_name_1(<ContextType>) => <EventReturnType>,
///     ...
///     event event_name_n(<ContextType>) => <EventReturnType>,
///
/// pub const CONSTANT_NAME;
/// pub trait TraitName;
/// ```
#[cfg(feature = "event_gen")]
#[proc_macro]
pub fn generate_events(input: TokenStream) -> TokenStream {
    let EventListExpr { event_exprs } = parse_macro_input!(input as EventListExpr);

    let mut event_handler_impl_new_tokens = quote![];
    let mut trait_enum_tokens = quote![];
    let mut event_handler_set_field_tokens = quote![];
    let mut event_handler_set_defaults_tokens = quote![];
    let mut trigger_event_function_tokens = quote![];

    for event_expr in event_exprs {
        let EventExpr {
            event_name_pascal_case,
            event_context_type_name_pascal_case,
            event_return_type_name,
            superevent_name_snake_case,
            is_trial_event,
            default_expr,
        } = event_expr;

        let event_name_snake_case = event_name_pascal_case.to_string().to_case(convert_case::Case::Snake);
        let event_name_snake_case = Ident::new(&event_name_snake_case, event_name_pascal_case.span());

        event_handler_impl_new_tokens.extend(quote!(
            #event_name_snake_case: vec![],
        ));

        trait_enum_tokens.extend(quote![
            #event_name_pascal_case,
        ]);

        event_handler_set_field_tokens.extend(quote![
            pub #event_name_snake_case: Option<EventHandler<#event_return_type_name, #event_context_type_name_pascal_case>>,
        ]);

        event_handler_set_defaults_tokens.extend(quote![
            #event_name_snake_case: None,
        ]);

        let function_name_snake_case = event_name_snake_case.to_string();
        let function_name_snake_case = Ident::new(&(String::from("trigger_") + &function_name_snake_case + "_event"), event_name_snake_case.span());
        let mut vec_of_event_handler_set_fields_tokens = quote![
            event_handler_set.#event_name_snake_case
        ];
        if let Some(event_name) = superevent_name_snake_case {
            vec_of_event_handler_set_fields_tokens.extend(quote![
                , event_handler_set.#event_name
            ]);
        };
        let default_tokens = if let Some(default) = default_expr {
            quote![#default]
        } else {
            quote![NOTHING]
        };

        if is_trial_event {
            trigger_event_function_tokens.extend(quote![
                pub(crate) fn #function_name_snake_case(
                    sim: &mut BattleSimulator,

                    broadcaster_id: MonsterID,
                    event_context: #event_context_type_name_pascal_case,
                ) -> #event_return_type_name {
                    EventDispatcher::dispatch_trial_event(
                        sim,
                        broadcaster_id,
                        |event_handler_set| {
                            vec![#vec_of_event_handler_set_fields_tokens]
                        },
                        event_context,
                    )
                }
            ]);
        } else {
            trigger_event_function_tokens.extend(quote![
                pub(crate) fn #function_name_snake_case(
                    sim: &mut BattleSimulator,

                    broadcaster_id: MonsterID,
                    event_context: #event_context_type_name_pascal_case,
                ) -> #event_return_type_name {
                    EventDispatcher::dispatch_event(
                        sim,
                        broadcaster_id,
                        |event_handler_set| {
                            vec![#vec_of_event_handler_set_fields_tokens]
                        },
                        event_context,
                        #default_tokens,
                        None
                    )
                }
            ]);
        }
    }

    let output = quote![
        #[derive(Debug, Clone, Copy)]
        pub struct EventHandlerDeck {
            #event_handler_set_field_tokens
        }

        pub(super) const DEFAULT_EVENT_HANDLERS: EventHandlerDeck = EventHandlerDeck {
            #event_handler_set_defaults_tokens
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum EventID {
            #trait_enum_tokens
        }

        #trigger_event_function_tokens
    ];

    output.into()
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
#[cfg(feature = "battle_builder")]
#[proc_macro]
pub fn battle(input: TokenStream) -> TokenStream {
    let battle_expr = parse_macro_input!(input as BattleExpr);

    let (ally_team_expr, opponent_team_expr) = battle_expr.team_exprs();
    let get_team_tokens = |team_expr: MonsterTeamExpr, method_ident: TokenStream2| {
        let team_monster_tokens = team_expr.monster_exprs().fold(quote!(), |mut tokens_so_far, monster_expr| {
            let MonsterExpr {
                monster_ident,
                maybe_nickname_literal,
                moveset_expr,
                ability_expr,
            } = monster_expr;

            let nickname_tokens = maybe_nickname_literal.map_or(quote!(), |nickname| quote!(.with_nickname(#nickname)));

            let mut number_of_moves = 0;
            let move_tokens = moveset_expr.move_exprs.into_iter().fold(quote!(), |mut tokens_so_far, move_expr| {
                number_of_moves += 1;
                let power_point_tokens = move_expr.maybe_power_points.clone().map(|lit_int| quote!(.with_power_points(#lit_int)));

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
            });
            let empty_moveslot_tokens = (number_of_moves + 1..=4).fold(quote!(), |mut tokens_so_far, _| {
                tokens_so_far.extend(quote![None,]);
                tokens_so_far
            });

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
        });
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
