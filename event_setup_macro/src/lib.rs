use convert_case::{Case::Pascal, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal};
use quote::{quote, ToTokens};
use syn::{braced, parse::{Parse, ParseStream}, parse_macro_input, Attribute, ExprMatch, ExprTuple, Generics, Pat, Token, Type, TypePath};

/// Generates the struct `CollectionType`, the default constant and the `TraitName` trait plus implementations for each event, when given a list of event identifiers.
/// The syntax for this is as follows
/// ```
/// pub struct <CollectionType> {
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
pub fn event_setup(input: TokenStream) -> TokenStream {
    
    let event_handler_type = quote!(EventHandler);
    
    let ExprEventHandlerDeck { 
        doc_comment, 
        first_pub_keyword, 
        struct_keyword, 
        struct_name, 
        maybe_lifetime_annotation, 
        match_expr, 
        const_keyword, 
        default_handler_constant_name, 
        default_handler_value, 
        second_pub_keyword, 
        trait_keyword, 
        trait_name 
    }: ExprEventHandlerDeck = parse_macro_input!(input);

    let mut fields_for_struct = quote!();
    let mut fields_for_constant = quote!();
    let mut events = quote!();

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
                let expr_tuple_result = attr.parse_args::<ExprTuple>();
                match expr_tuple_result {
                    Ok(expr_tuple) => { maybe_context_type = Some(quote!(#expr_tuple))},
                    Err(_) => {
                        let type_parse_result = attr.parse_args::<Type>();
                        match type_parse_result {
                            Ok(type_ident) => {
                                maybe_context_type = Some(quote!(#type_ident))
                            },
                            Err(_) => panic!("The context must be a valid type."),
                        }
                    },
                }
            } else {
                panic!("Only doc comment and `context` attributes are allowed in this macro.")
            }
        }
        let event_context_type = match maybe_context_type {
            Some(tokens) => tokens,
            None => panic!("A context must be specified for each field."),
        };
        let handler_ident = expression.pat;
        let handler_return_type = *expression.body;
        
        let pat_ident = match handler_ident {
            Pat::Ident( ref pat_ident) => pat_ident.clone(),
            _ => panic!("Error: Expected handler_ident to be an identifier."),
        };
        
        let trait_name_string_in_pascal_case = pat_ident.clone().ident.to_string().to_case(Pascal);
        let trait_name_as_string_literal = Literal::string(&trait_name_string_in_pascal_case);
        let pascal_case_event_ident = Ident::new(
            &trait_name_string_in_pascal_case,
            pat_ident.ident.span(),
        );
            fields_for_struct = quote!( 
                #fields_for_struct
                #comments
                pub #handler_ident: Option<for<'b> fn(&'b mut Battle<'b>, #event_context_type<'b>, #handler_return_type) -> #handler_return_type>,
            );
            fields_for_constant = quote!(
                #fields_for_constant
                #handler_ident: #default_handler_value,
            );
            events = quote!(
                #events

                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub struct #pascal_case_event_ident;

                impl<'a> #trait_name<'a> for #pascal_case_event_ident {
                    type EventReturnType = #handler_return_type;
                    type EventContext = #event_context_type<'a>;
                    fn corresponding_handler(&self, event_handler_deck: &#struct_name) -> Option<for<'b> fn(&'b mut Battle<'b>, #event_context_type<'b>, #handler_return_type) -> #handler_return_type> {
                        event_handler_deck.#handler_ident
                    }
        
                    fn name(&self) -> &'static str {
                        #trait_name_as_string_literal
                    }
                }
            );
    }

    let maybe_lifetime_annotation = maybe_lifetime_annotation.map_or(quote!(), |lifetime_annotation| {quote!(#lifetime_annotation)});

    let output_token_stream = quote!(
        #doc_comment
        #[derive(Debug, Clone, Copy)]
        #first_pub_keyword #struct_keyword #struct_name {
            #fields_for_struct
        }

        #const_keyword #default_handler_constant_name: #struct_name = #struct_name {
            #fields_for_constant
        };

        #second_pub_keyword #trait_keyword #trait_name<'a>: Clone + Copy {
            type EventReturnType: Sized + Clone + Copy;
            type EventContext: Sized + Clone + Copy;

            fn corresponding_handler(
                &self,
                event_handler_deck: &#struct_name,
            ) -> Option<for<'b> fn(&'b mut Battle<'b>, Self::EventContext, Self::EventReturnType) -> Self::EventReturnType>;

            fn name(&self) -> &'static str;
        }

        pub mod event_dex {
            use super::*;

            #events
        }
    );
    output_token_stream.into()
}

struct ExprEventHandlerDeck {
    doc_comment: Attribute,
    first_pub_keyword: Token![pub],
    struct_keyword: Token![struct],
    struct_name: Ident,
    maybe_lifetime_annotation: Option<Generics>,
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
        let maybe_lifetime_annotation: Option<Generics> = input.parse().ok();
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
                maybe_lifetime_annotation,
                match_expr,
                
                const_keyword,
                default_handler_value,
                default_handler_constant_name,
                
                second_pub_keyword,
                trait_keyword,
                trait_name,
            }
        )
    }
}

struct ExprContext {
    context_type: TokenStream
}