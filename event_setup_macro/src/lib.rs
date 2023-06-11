use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal};
use quote::quote;
use syn::{parse_macro_input, ExprMatch, Token, parse::{Parse, ParseStream}, braced, Pat};

/// Generates the struct `EventResponder`, the default constant and the `InBattleEvent` trait plus implementations for each event, when given a list of event identifiers.
/// The syntax for this is as follows
/// ```
/// pub struct EventResponder {
/// match event {
///         event_name_1 => <EventReturnType>,
///         ...
///         event_name_n => <EventReturnType>,
///     }
/// }
/// pub const DEFAULT_RESPONDER;
/// pub trait InBattleEvent;  
/// ```
#[proc_macro]
pub fn event_setup(input: TokenStream) -> TokenStream {
    let expr_ehs: ExprEventResponder = parse_macro_input!(input);
    
    let first_pub_keyword = expr_ehs.first_pub_keyword;
    let struct_keyword = expr_ehs.struct_keyword;
    let struct_name = expr_ehs.struct_name;
    let match_expr = expr_ehs.match_expr;
    let second_pub_keyword = expr_ehs.second_pub_keyword;
    let const_keyword = expr_ehs.const_keyword;
    let default_responder_constant_name = expr_ehs.default_responder_constant_name;
    let default_responder_value = expr_ehs.default_responder_value;
    let third_pub_keyword = expr_ehs.third_pub_keyword;
    let trait_keyword = expr_ehs.trait_keyword;
    let trait_name = expr_ehs.trait_name;

    let mut fields = quote!();
    let mut fields_for_constant = quote!();
    let mut events = quote!();
    for expression in match_expr.arms {
        let responder_identifier = expression.pat;
        let responder_return_type = *expression.body;
        
        let pat_ident = match responder_identifier {
            Pat::Ident( ref pat_ident) => pat_ident.clone(),
            _ => panic!("Error: Expected responder_identifier to be an identifier."),
        };
        let trait_name_string_in_pascal_case = to_pascal_case(pat_ident.clone().ident.to_string());
        let event_trait_literal = Literal::string(&trait_name_string_in_pascal_case);
        let trait_name_ident_in_pascal_case = Ident::new(
            &trait_name_string_in_pascal_case,
            pat_ident.ident.span(),
        );
            fields = quote!( 
                #fields
                pub #responder_identifier: Option<EventResponder<#responder_return_type>>,
            );
            fields_for_constant = quote!(
                #fields_for_constant
                #responder_identifier: #default_responder_value,
            );
            events = quote!(
                #events

                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub struct #trait_name_ident_in_pascal_case;

                impl #trait_name for #trait_name_ident_in_pascal_case {
                    type EventReturnType = #responder_return_type;
                    fn corresponding_responder(&self, composite_event_responder: &#struct_name) -> Option<EventResponder<Self::EventReturnType>> {
                        composite_event_responder.#responder_identifier
                    }
        
                    fn name(&self) -> &'static str {
                        #event_trait_literal
                    }
                }
            );
    }

    let output_token_stream = quote!(
        #[derive(Debug, Clone, Copy)]
        #first_pub_keyword #struct_keyword #struct_name {
            #fields
        }

        #second_pub_keyword #const_keyword #default_responder_constant_name: #struct_name = #struct_name {
            #fields_for_constant
        };

        #third_pub_keyword #trait_keyword #trait_name {
            type EventReturnType: Sized + Clone + Copy;

            fn corresponding_responder(
                &self,
                composite_event_responder: &#struct_name,
            ) -> Option<EventResponder<Self::EventReturnType>>;

            fn name(&self) -> &'static str;
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

struct ExprEventResponder {
    first_pub_keyword: Token![pub],
    struct_keyword: Token![struct],
    struct_name: Ident,
    match_expr: ExprMatch,
    second_pub_keyword: Token![pub],
    const_keyword: Token![const],
    default_responder_constant_name: Ident,
    default_responder_value: Ident,
    third_pub_keyword: Token![pub],
    trait_keyword: Token![trait],
    trait_name: Ident,
}

impl Parse for ExprEventResponder {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first_pub_keyword: Token![pub] = input.parse()?;
        let struct_keyword: Token![struct] = input.parse()?;
        let struct_name: Ident = input.parse()?;
        let content;
         _ = braced!(content in input);
        let match_expr: ExprMatch = content.parse()?;
        let second_pub_keyword: Token![pub] = input.parse()?;
        let const_keyword: Token![const] = input.parse()?;
        let default_responder_constant_name: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let default_responder_value: Ident = input.parse()?;
        let _: Token![;] = input.parse()?;
        let third_pub_keyword: Token![pub] = input.parse()?;
        let trait_keyword: Token![trait] = input.parse()?;
        let trait_name: Ident = input.parse()?;
        let _: Token![;] = input.parse()?;
        Ok(
            Self {
                first_pub_keyword,
                struct_keyword,
                struct_name,
                match_expr,
                default_responder_constant_name,
                second_pub_keyword,
                const_keyword,
                default_responder_value,
                third_pub_keyword,
                trait_keyword,
                trait_name,
            }
        )
    }
}