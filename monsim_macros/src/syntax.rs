
pub mod accessor_macro_syntax {
    use syn::{parse::Parse, Token};
    use proc_macro2::Ident;

    pub struct ExprMechanicAccessor {
        pub is_mut: bool,
        pub ident: Ident,
    }
    
    impl Parse for ExprMechanicAccessor {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let is_mut = input.parse::<Token!(mut)>().is_ok();
            let ident: Ident = input.parse()?;
            Ok(ExprMechanicAccessor {
                is_mut,
                ident,
            })
        }
    }
}

pub mod event_system_macro_syntax {
    use proc_macro2::Ident;
    use syn::{braced, parse::{Parse, ParseStream}, Attribute, ExprMatch, Token};

    pub struct ExprEventHandlerDeck {
        pub doc_comment: Attribute,
        pub first_pub_keyword: Token![pub],
        pub struct_keyword: Token![struct],
        pub struct_name: Ident,
        pub match_expr: ExprMatch,
        pub const_keyword: Token![const],
        pub default_handler_constant_name: Ident,
        pub default_handler_value: Ident,
        pub second_pub_keyword: Token![pub],
        pub trait_keyword: Token![trait],
        pub trait_name: Ident,
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
}

pub mod battle_macro_syntax {
    use quote::{quote, ToTokens};
    use syn::{braced, parenthesized, parse::Parse, token::{Brace, Comma, Token}, Error, Ident, LitInt, LitStr, Token};

    mod keywords{
        use syn::custom_keyword;

        custom_keyword!(moveset);
        custom_keyword!(ability);
        custom_keyword!(team);
        custom_keyword!(power_points);
    }

    #[derive(Clone)]
    pub struct BattleExpr {
        pub ally_team_expr: MonsterTeamExpr,
        pub opponent_team_expr: MonsterTeamExpr,
    }

    impl Parse for BattleExpr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let items = input.parse_terminated(MonsterTeamExpr::parse, Comma)?;

            if items.len() != 2 {
                return Err(Error::new(input.span(), format!["Expected exactly 2 MonsterTeam expressions, but {} were found", items.len()]))
            }

            let expr_ally_team = (*items.iter()
                .find(|item| { item.team_ident.to_string() == "Allies" })
                .expect("Expected one team to be marked with `Allies`"))
                .clone();

            let expr_opponent_team = items.into_iter()
                .find(|item| { item.team_ident.to_string() == "Opponents" })
                .expect("Expected one team to be marked with `Opponents`");

            Ok(BattleExpr {
                ally_team_expr: expr_ally_team,
                opponent_team_expr: expr_opponent_team,
            })
        }
    }

    impl BattleExpr {
        pub fn team_exprs(self) -> (MonsterTeamExpr, MonsterTeamExpr) {
            (self.ally_team_expr, self.opponent_team_expr)
        }
    }

    /// syntax: 
    /// ```no_compile
    /// team: Allies/Opponents
    /// {
    ///     MonsterExpr,
    ///     ... 0-5 more
    /// }
    /// ```
    #[derive(Clone)]
    pub struct MonsterTeamExpr {
        pub team_ident: Ident,
        pub monster_exprs: Vec<MonsterExpr>,
    }
    
    const MAX_MONSTERS_PER_TEAM: usize = 6;

    impl Parse for MonsterTeamExpr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let _: keywords::team = input.parse()?;
            let _: Token![:] = input.parse()?;
            let team_ident: Ident = input.parse()?;
            let content;
            let _ = braced!(content in input);
            let items = content.parse_terminated(MonsterExpr::parse, Comma)?;
            let monster_expr = items.into_iter().collect::<Vec<_>>();
            if monster_expr.is_empty() {
                return Err(Error::new(content.span(), "Expected at least one Monster expression, but none were found"));
            } else if monster_expr.len() > MAX_MONSTERS_PER_TEAM {
                return Err(Error::new(content.span(), format!["Expected at most 6 Monster expressions, but {} were found", monster_expr.len()]));
            }

            Ok(MonsterTeamExpr {
                team_ident,
                monster_exprs: monster_expr,
            })
        }
    }

    impl MonsterTeamExpr {
        pub fn monster_exprs(self) -> impl Iterator<Item = MonsterExpr> {
            self.monster_exprs.into_iter()
        }
    }

    /// syntax:
    /// ```no_compile
    /// *MonsterName*: "*OptionalNicknameStrLiteral*" {
    ///     moveset: ExprMoveSet,
    ///     ability: ExprAbility
    /// }
    /// ```
    #[derive(Clone)]
    pub struct MonsterExpr {
        pub monster_ident: Ident,
        pub maybe_nickname_literal: Option<LitStr>,
        pub moveset_expr: MoveSetExpr,
        pub ability_expr: AbilityExpr,
    }

    impl Parse for MonsterExpr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let monster_ident: Ident = input.parse()?;
            let maybe_nickname_literal: Option<LitStr> = match input.parse::<Token![:]>() {
                Ok(_) => {
                    Some(input.parse()?)
                },
                Err(_) => None,
            };
            let braced_content; let _ = braced!(braced_content in input);
            let moveset_expr: MoveSetExpr = braced_content.parse()?;
            let _: Token![,] = braced_content.parse()?;
            let ability_expr: AbilityExpr = braced_content.parse()?;
            // Last comma is optional
            let _ = braced_content.parse::<Token![,]>();
            
            Ok(MonsterExpr {
                monster_ident,
                maybe_nickname_literal,
                moveset_expr,
                ability_expr,
            })
        }
    }

    #[derive(Clone)]
    /// syntax: `moveset: (ExprMove, ...0-3 more)`
    pub struct MoveSetExpr {
        pub move_exprs: Vec<MoveExpr>,
        
    }

    impl Parse for MoveSetExpr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let _: keywords::moveset = input.parse()?;
            let _: Token![:] = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            let content_span = content.span();
            let items = content.parse_terminated(MoveExpr::parse, Comma)?; 
            let move_exprs = items.into_iter().collect::<Vec<_>>();
            if move_exprs.len() < 1 {
                return Err(Error::new(content_span, "Expected at least one Move expression, but none were found"));
            } else if move_exprs.len() > 4 {
                return Err(Error::new(content_span, format!["Expected at most 4 Move expressions, but {} were found", move_exprs.len()]));
            }

            Ok(MoveSetExpr {
                move_exprs,
            })
        }
    }

    
    /// syntax: `*MoveName*`
    #[derive(Clone)]
    pub struct MoveExpr {
        pub move_ident: Ident,
        pub maybe_power_points: Option<LitInt>,
    }
    
    impl Parse for MoveExpr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let move_ident: Ident = input.parse()?;
            let mut maybe_power_points: Option<LitInt> = None;
            if input.peek(Brace) {
                let content; let _ = braced!(content in input);

                // Optional power point field 
                if content.parse::<keywords::power_points>().is_ok() {
                    let _: Token![:] = content.parse()?;
                    maybe_power_points = Some(content.parse()?);
                };
            }

            Ok(MoveExpr {
                move_ident,
                maybe_power_points,
            })
        }
    }

    impl ToTokens for MoveExpr {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let move_ident = &self.move_ident;
            tokens.extend(quote!(#move_ident));
        }
    } 

    /// syntax: `ability: *AbilityName*`
    #[derive(Clone)]
    pub struct AbilityExpr {
        pub ability_ident: Ident,
    }

    impl Parse for AbilityExpr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let _: keywords::ability = input.parse()?;
            let _ : Token![:] = input.parse()?;
            let ability_ident: Ident = input.parse()?;

            Ok(AbilityExpr {
                ability_ident,
            })
        }
    }

    impl ToTokens for AbilityExpr {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let ability_ident = &self.ability_ident;
            tokens.extend(quote!(#ability_ident));
        }
    }

}