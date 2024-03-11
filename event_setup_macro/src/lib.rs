use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal};
use quote::quote;
use syn::{braced, parse::{Parse, ParseStream}, parse_macro_input, Attribute, ExprMatch, ExprTuple, Pat, Token};

/// Generates the struct `EventResponder`, the default constant and the `InBattleEvent` trait plus implementations for each event, when given a list of event identifiers.
/// The syntax for this is as follows
/// ```
/// pub struct EventResponder {
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
/// pub const DEFAULT_RESPONDER;
/// pub trait InBattleEvent;  
/// ```
#[proc_macro]
pub fn event_setup(input: TokenStream) -> TokenStream {
    
    let event_handler_type = quote!(EventHandler);
    
    let expr_ehd: ExprEventHandlerDeck = parse_macro_input!(input);
    
    let doc_comment = expr_ehd.doc_comment;
    let first_pub_keyword = expr_ehd.first_pub_keyword;
    let struct_keyword = expr_ehd.struct_keyword;
    let struct_name = expr_ehd.struct_name;
    let match_expr = expr_ehd.match_expr;
    let const_keyword = expr_ehd.const_keyword;
    let default_handler_constant_name = expr_ehd.default_handler_constant_name;
    let default_handler_value = expr_ehd.default_handler_value;
    let second_pub_keyword = expr_ehd.second_pub_keyword;
    let trait_keyword = expr_ehd.trait_keyword;
    let trait_name = expr_ehd.trait_name;

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

struct ExprEventHandlerDeck {
    doc_comment: Attribute,
    first_pub_keyword: Token![pub],
    struct_keyword: Token![struct],
    struct_name: Ident,
    match_expr: ExprMatch,
    const_keyword: Token![const],
    default_handler_constant_name: Ident,
    default_handler_value: Ident,
    second_pub_keyword: Token![pub],
    trait_keyword: Token![trait],
    trait_name: Ident,
}

impl Parse for ExprEventHandlerDeck {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let doc_comment = input.call(Attribute::parse_outer)?[0].clone();
        let first_pub_keyword: Token![pub] = input.parse()?;
        let struct_keyword: Token![struct] = input.parse()?;
        let struct_name: Ident = input.parse()?;
        let content;
         _ = braced!(content in input);
        let match_expr: ExprMatch = content.parse()?;
        let const_keyword: Token![const] = input.parse()?;
        let default_handler_constant_name: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let default_handler_value: Ident = input.parse()?;
        let _: Token![;] = input.parse()?;
        let second_pub_keyword: Token![pub] = input.parse()?;
        let trait_keyword: Token![trait] = input.parse()?;
        let trait_name: Ident = input.parse()?;
        let _: Token![;] = input.parse()?;
        Ok(
            Self {
                doc_comment,
                first_pub_keyword,
                struct_keyword,
                struct_name,
                match_expr,
                default_handler_constant_name,
                const_keyword,
                default_handler_value,
                second_pub_keyword,
                trait_keyword,
                trait_name,
            }
        )
    }
}