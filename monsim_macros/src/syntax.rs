#[cfg(feature = "entity_fetchers")]
pub mod entity_fetcher_macro_syntax {
    use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
    use quote::quote;
    use syn::{parse::Parse, Token};

    pub struct ExprEntityFetcher {
        pub is_mut: bool,
        pub path_to_entity: TokenStream2,
        pub span: Span,
    }

    impl Parse for ExprEntityFetcher {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let is_mut = input.parse::<Token!(mut)>().is_ok();
            let span = Span::call_site();
            let ident: Ident = input.parse()?;
            let mut path_to_entity = quote!(#ident);
            let mut parsed_dot = true;
            while parsed_dot {
                let optional_dot: Option<Token![.]> = input.parse().ok();
                if optional_dot.is_some() {
                    let ident: Ident = input.parse()?;
                    path_to_entity.extend(quote!(.#ident));
                    parsed_dot = true;
                } else {
                    parsed_dot = false;
                }
            }
            Ok(ExprEntityFetcher { is_mut, path_to_entity, span })
        }
    }
}

#[cfg(feature = "event_gen")]
pub mod event_system_macro_syntax {
    use proc_macro2::Ident;
    use syn::{
        parenthesized,
        parse::{Parse, ParseStream},
        token::Comma,
        Error, Expr, ExprStruct, Token,
    };

    pub struct EventListExpr {
        pub event_exprs: Vec<EventExpr>,
    }

    impl Parse for EventListExpr {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let items = input.parse_terminated(EventExpr::parse, Comma)?;
            let events = items.into_iter().collect::<Vec<_>>();
            Ok(Self { event_exprs: events })
        }
    }

    pub struct EventExpr {
        pub is_trial_event: bool,
        pub event_name_pascal_case: Ident,
        pub event_context_type_name_pascal_case: Ident,
        pub event_return_type_name: Ident,
        pub superevent_name_snake_case: Option<Expr>,
        pub default_expr: Option<Expr>,
    }

    mod keywords {
        use syn::custom_keyword;

        custom_keyword!(event);
        custom_keyword!(inherits);
        custom_keyword!(default);
    }

    impl Parse for EventExpr {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let is_trial_event = input.parse::<Token![try]>().is_ok();
            let _: keywords::event = input.parse()?;
            let event_name_pascal_case: Ident = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let event_context_type_name: Ident = content.parse()?;
            let _: Token![=>] = input.parse()?;
            let event_return_type_name: Ident = input.parse()?;
            let mut superevent_name_pascal_case = None;
            let mut default_expr = None;
            if input.parse::<Token![;]>().is_ok() {
                let settings = input.parse::<ExprStruct>()?;
                assert!(
                    settings.path.get_ident().expect("Expected settings struct to have an identifier").to_string() == "Settings",
                    "Expected settings struct to have identifier `Settings`"
                );
                for field in settings.fields {
                    match field.member.clone() {
                        syn::Member::Named(ident) => {
                            if ident.to_string() == "inherits" {
                                superevent_name_pascal_case = Some(field.expr)
                            } else if ident.to_string() == "default" {
                                default_expr = Some(field.expr)
                            } else {
                                return Err(Error::new_spanned(field.member, "The only allowed settings are 'inherits' and 'default'."));
                            }
                        }
                        syn::Member::Unnamed(_) => {
                            return Err(Error::new_spanned(field.member, "Unnamed fields are not allowed"));
                        }
                    }
                }
            }

            Ok(EventExpr {
                is_trial_event,
                event_name_pascal_case,
                event_context_type_name_pascal_case: event_context_type_name,
                event_return_type_name,
                superevent_name_snake_case: superevent_name_pascal_case,
                default_expr,
            })
        }
    }
}
