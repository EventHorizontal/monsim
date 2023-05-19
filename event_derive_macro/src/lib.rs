extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use syn::parse::ParseStream;
use syn::{parenthesized, parse, parse2, Result};
use syn::{parse::Parse, DeriveInput};

#[proc_macro_derive(InBattleEvent, attributes(return_type, callback))]
pub fn derive_in_battle_event_trait(input: TokenStream) -> TokenStream {
    let mut ast: DeriveInput = parse(input).unwrap();
    let struct_name = ast.ident.clone();

    let return_type = ast.attrs.remove(0).tokens;
    let return_type: ParenthesisedReturnTypeExpr =
        parse2(return_type).expect("Failed to parse parenthesised expression");
    let return_type = return_type.0;

    let callback = ast.attrs.remove(0).tokens;
    let callback: ParenthesisedIdentifierExpr = parse2(callback).expect("There should be a parant");
    let callback = callback.0;
    let struct_name_literal = Literal::string(&struct_name.to_string());

    quote::quote!(
        impl InBattleEvent for #struct_name {
            type EventReturnType = #return_type;
            fn corresponding_handler(&self, event_handler_set: &EventHandlerSet) -> Option<EventHandler<Self::EventReturnType>> {
                event_handler_set.#callback
            }

            fn name(&self) -> &'static str {
                #struct_name_literal
            }
        }
    ).into()
}

struct ParenthesisedReturnTypeExpr(syn::Type);

impl Parse for ParenthesisedReturnTypeExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let type_expr: syn::Type = content.parse()?;
        Ok(ParenthesisedReturnTypeExpr(type_expr))
    }
}

struct ParenthesisedIdentifierExpr(syn::Ident);

impl Parse for ParenthesisedIdentifierExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let type_expr: syn::Ident = content.parse()?;
        Ok(ParenthesisedIdentifierExpr(type_expr))
    }
}
