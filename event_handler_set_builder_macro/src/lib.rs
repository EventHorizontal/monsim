use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal};
use quote::quote;
use syn::{parse_macro_input, ExprMatch, Token, parse::{Parse, ParseStream}, braced};

#[proc_macro]
pub fn build_event_handler_set(input: TokenStream) -> TokenStream {
    let expr_event_handler_set: ExprEventHandlerSet = parse_macro_input!(input);
    
    let pub_keyword = expr_event_handler_set.pub_keyword;
    let struct_keyword = expr_event_handler_set.struct_keyword;
    let struct_name = expr_event_handler_set.struct_name;
    let match_expr = expr_event_handler_set.match_expr;
    let default_handler_constant_name = expr_event_handler_set.default_handler_constant_name;

    let mut fields = quote!();
    let mut fields_for_constant = quote!();
    for expression in match_expr.arms {
        let handler_identifier = expression.pat;
        let handler_return_type = *expression.body;
            fields = quote!( 
                #fields
                pub #handler_identifier: Option<EventHandler<#handler_return_type>>,
            );
            fields_for_constant = quote!(
                #fields_for_constant
                #handler_identifier: None,
            )
    }

    let output_token_stream = quote!(
        #[derive(Debug, Clone, Copy)]
        #pub_keyword #struct_keyword #struct_name {
            #fields
        }

        pub const #default_handler_constant_name: #struct_name = #struct_name {
            #fields_for_constant
        };
    );

    output_token_stream.into()
}

struct ExprEventHandlerSet {
    pub_keyword: Token![pub],
    struct_keyword: Token![struct],
    struct_name: Ident,
    match_expr: ExprMatch,
    default_handler_constant_name: Ident,
}

impl Parse for ExprEventHandlerSet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pub_keyword: Token![pub] = input.parse()?;
        let struct_keyword: Token![struct] = input.parse()?;
        let struct_name: Ident = input.parse()?;
        let content;
         _ = braced!(content in input);
        let match_expr: ExprMatch = content.parse()?;
        let set_ident: Ident = input.parse()?;
        assert!(&set_ident.to_string() == "set");
        let default_handler_constant_name: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let default_handler_value: Ident = input.parse()?;
        assert!(default_handler_value.to_string() == String::from("None"));
        Ok(
            Self {
                pub_keyword,
                struct_keyword,
                struct_name,
                match_expr,
                default_handler_constant_name,
            }
        )
    }
}