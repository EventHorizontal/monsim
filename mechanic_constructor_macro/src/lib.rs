use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{ParseStream, Parse};

#[proc_macro]
pub fn define_ability(input: TokenStream) -> TokenStream {    
    let output_token_stream = quote!();
    output_token_stream.into()
}

struct ExprAbilityDefinition {

}

impl Parse for ExprAbilityDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}