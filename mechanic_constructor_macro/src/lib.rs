use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::{ParseStream, Parse}, parse_macro_input, braced, Token, ExprClosure, LitInt, LitStr};

#[proc_macro]
pub fn define_ability(input: TokenStream) -> TokenStream {    
    let expr_ability_def: ExprAbilityDefinition = parse_macro_input!(input);

    let dex_number_literal = expr_ability_def.dex_number_literal;
    let ability_name_literal = expr_ability_def.ability_name_literal;
    let expr_event_handler_list = expr_ability_def.expr_event_handler_list;
    let on_activate_callback = expr_ability_def.on_activate_callback.expr_closure;
    let _filter_value_ident = expr_ability_def.filter_value_ident;
    let order_value_literal = expr_ability_def.order_value_literal;

    let mut event_handlers = quote!();
    for ExprCallback {
        name_ident: expr_event_handler,
        colon_token: _,
        expr_closure: expr_event_handler_closure,
        comma_token: _,
    } in expr_event_handler_list.into_iter() {
        event_handlers = quote!(
            #event_handlers

            #expr_event_handler: Some(EventHandler {
                #[cfg(feature = "debug")]
                dbg_location: monsim::debug_location!(#ability_name_literal),
                callback: #expr_event_handler_closure,
            }),
        )
    }

    let ability_name_in_pascal_case = to_pascal_case(ability_name_literal.value());
    let ability_name_in_pascal_case = Ident::new(
        &ability_name_in_pascal_case,
        ability_name_literal.span(),
    );

    let output_token_stream = quote!(
        pub const #ability_name_in_pascal_case: AbilitySpecies = AbilitySpecies {
            dex_number: #dex_number_literal,
            name: #ability_name_literal,
            event_handlers: EventHandlerSet {
                #event_handlers
                ..DEFAULT_HANDLERS
            },
            on_activate: #on_activate_callback,
            filters: EventHandlerFilters::default(),
            order: #order_value_literal,
        };        
    );
    output_token_stream.into()
}

struct ExprAbilityDefinition {
    dex_number_literal: LitInt,
    ability_name_literal: LitStr,
    expr_event_handler_list: Vec<ExprCallback>,
    on_activate_callback: ExprCallback,
    filter_value_ident: Ident,
    order_value_literal: LitInt,
    
}

impl Parse for ExprAbilityDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let dex_number_literal: LitInt = input.parse()?;
        let ability_name_literal: LitStr = input.parse()?;
        let outer_brace_content;
        _ = braced!(outer_brace_content in input);
        let event_handler_list_brace_content;
        _ = braced!(event_handler_list_brace_content in outer_brace_content);
        let mut expr_event_handler_list = Vec::with_capacity(30);
        loop {
            let expr_event_handler: ExprCallback = match event_handler_list_brace_content.parse() {
                Ok(expr) => { expr },
                Err(_) => { break; },
            };
            expr_event_handler_list.push(expr_event_handler);
        }
        let _: Token![,] = outer_brace_content.parse()?;
        let on_activate_callback: ExprCallback = outer_brace_content.parse()?;

        // Filters field
        let filters_ident: Ident = outer_brace_content.parse()?;
        assert!(&filters_ident.to_string() == "filters");
        let _: Token![:] = outer_brace_content.parse()?;
        let filter_value_ident: Ident = outer_brace_content.parse()?;
        assert!(&filter_value_ident.to_string() == "DEFAULT");
        let _: Token![,] = outer_brace_content.parse()?;

        // Order field
        let order_ident: Ident = outer_brace_content.parse()?;
        assert!(&order_ident.to_string() == "order");
        let _: Token![:] = outer_brace_content.parse()?;
        let order_value_literal: LitInt = outer_brace_content.parse()?;
        let _: Token![,] = outer_brace_content.parse()?;
        Ok(
            Self {
                dex_number_literal,
                ability_name_literal,
                expr_event_handler_list,
                on_activate_callback,
                filter_value_ident,
                order_value_literal,
            }
        )
    }
}

struct ExprCallback {
    name_ident: Ident,
    colon_token: Token![:],
    expr_closure: ExprClosure,
    comma_token: Token![,],
}

const VALID_EVENT_CALLBACKS: [&str; 2] = [
    "on_try_move",
    "on_activate",
];

impl Parse for ExprCallback {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_ident: Ident = input.parse()?;
        assert!(VALID_EVENT_CALLBACKS.contains(&name_ident.to_string().as_str()));
        let colon_token: Token![:] = input.parse()?;
        let expr_closure: ExprClosure = input.parse()?;
        let comma_token: Token![,] = input.parse()?;
        Ok(
            Self {
                name_ident,
                colon_token,
                expr_closure,
                comma_token,
            }
        )
    }
}

fn to_pascal_case(input_string: String) -> String {
    let output_string = input_string.replace("\"", "");
    output_string.replace(" ", "")
}
