pub use battle_state_macro_syntax::*;
pub use event_system_macro_syntax::*;
pub use accessor_macro_syntax::*;

mod battle_state_macro_syntax {
    use proc_macro2::{Ident, Literal};
    use syn::{braced, parse::{Parse, ParseStream}, punctuated::Punctuated, token::Comma, Error, Result, ExprPath, Token,};
    #[derive(Clone)]
    pub struct ExprBattle {
        pub ally_expr_monster_team: ExprMonsterTeam,
        pub opponent_expr_monster_team: ExprMonsterTeam,
    }
    
    impl Parse for ExprBattle {
        fn parse(input: ParseStream) -> Result<Self> {
            let battle_contents = parse_braced_comma_separated_list::<ExprMonsterTeam>(input)?;
            let mut battle_contents = battle_contents.iter();
    
            let mut check_team_uid = |id_to_match: &str| -> Result<ExprMonsterTeam> {
                let team_expr = battle_contents
                    .next()
                    .expect("Error: Failed to parse team identifier.")
                    .clone();
    
                let team_path = team_expr.team_path.clone();
    
                if is_expected_type(id_to_match, &path_to_ident(&team_path)) {
                    Ok(team_expr)
                } else {
                    return Err(Error::new_spanned(
                        team_path.clone(),
                        format!(
                            "Error: Expected Team identifier {}, found {}.",
                            id_to_match,
                            path_to_ident(&team_path).to_string().as_str()
                        ),
                    ));
                }
            };
    
            let ally_expr_monster_team = check_team_uid("Allies")?;
            let opponent_expr_monster_team = check_team_uid("Opponents")?;
    
            Ok(ExprBattle {
                ally_expr_monster_team,
                opponent_expr_monster_team,
            })
        }
    }
    
    #[derive(Clone)]
    pub struct ExprMonsterTeam {
        pub team_path: ExprPath,
        pub team_type: ExprPath,
        pub monster_fields: Punctuated<ExprMonster, Comma>,
    }
    
    impl Parse for ExprMonsterTeam {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let team_uident: ExprPath = input.parse().expect(
                "Error: Failed to parse TeamExpression identifier in expected TeamExpression.",
            );
    
            let _: Token![:] = input.parse()?;
            
            let team_type: ExprPath = input.parse()?;
    
            let monster_fields = parse_braced_comma_separated_list::<ExprMonster>(input)?;
            let monster_count = monster_fields.clone().iter().len();
            if monster_count > 6 || monster_count < 1 {
                return Err(Error::new_spanned(
                    team_uident.clone(),
                    format!(
                        "Error: You can put betweeen one and six monsters on a team. {:?} has {} monsters.", 
                        path_to_ident(&team_uident).to_string(),
                        monster_count,
                    ),
                ));
            };
    
            Ok(ExprMonsterTeam {
                team_path: team_uident,
                team_type,
                monster_fields,
            })
        }
    }
    
    fn parse_braced_comma_separated_list<T: Parse>(input: ParseStream) -> Result<Punctuated<T, Comma>> {
        let content;
        braced!(content in input);
        content.parse_terminated(T::parse, Comma)
    }
    
    fn is_expected_type(expected_keyword_name: &str, keyword: &Ident) -> bool {
        let keyword_as_string = keyword.to_string();
        let keyword_as_str = keyword_as_string.as_str();
        keyword_as_str == expected_keyword_name
    }
    
    // Syntax:
    // let <MonsterName>: <PathToMonsterType> = <Nickname> { <GameMechanicExpression>, ... }
    #[derive(Clone)]
    pub struct ExprMonster {
        pub monster_instance_path: ExprPath,
        pub monster_type_path: ExprPath,
        pub nickname_literal: Option<Literal>,
        pub fields: Punctuated<ExprGameMechanic, Comma>,
        pub move_count: usize,
    }
    
    const MONSTER_KEYWORD: &str = "Monster";
    
    impl Parse for ExprMonster {
        fn parse(input: ParseStream) -> Result<Self> {
    
            let monster_instance_path: ExprPath = input.parse()?;
    
            let _: Token![:] = input.parse()?;
            
            let monster_type_path: ExprPath = input.parse()?;
    
            let monster_type = path_to_ident(&monster_type_path);
    
            let mut nickname_literal = None;
            if input.parse::<Token!(=)>().is_ok() {
                let nickname_ident_result =  input.parse::<Literal>();
                nickname_literal = nickname_ident_result.ok();
            }
    
            if monster_type.to_string().as_str() == MONSTER_KEYWORD {
            let fields = parse_braced_comma_separated_list::<ExprGameMechanic>(input)?;
                
            // Alerting the user if the number of moves is greater than 4.
            let move_count = fields.iter()
                .filter(|it| { it.game_mechanic_type == GameMechanicType::Move })
                .count();
            let monster_ident = path_to_ident(&monster_instance_path).to_string();
            let default_nickname_literal = Literal::string(&monster_ident);
            let nickname_or_default = nickname_literal
                .clone()
                .unwrap_or(default_nickname_literal)
                .to_string();
            // let nickname_or_default = "TEST";
            if move_count > 4 || move_count < 1 {
                let incorrect_move_count_error = Err(Error::new_spanned(
                    monster_instance_path.clone(),
                    format!(
                        "Error: You can put betweeen one and four moves on a monster. {:?} has {} moves.", 
                        nickname_or_default,
                        move_count,
                    ),
                ));
                return incorrect_move_count_error;
            };
            
            // Alerting the user if the number of abilities is greater than 1.
            let ability_count = fields.iter()
                .filter(|it| { it.game_mechanic_type == GameMechanicType::Ability })
                .count();
            
            if ability_count == 0 {
                let incorrect_ability_count_error = Err(Error::new_spanned(
                    monster_instance_path.clone(),
                    format!(
                        "Error: You must put at leaset one ability on a monster.{:?} has no ability.", 
                        nickname_or_default
                    ),
                ));
                return incorrect_ability_count_error;
            } else if ability_count > 1 {
                let incorrect_ability_count_error = Err(Error::new_spanned(
                    monster_instance_path.clone(),
                    format!(
                        "Error: You can only put one ability on a monster.{:?} has more than one ability.", 
                        nickname_or_default
                    ),
                ));
                return incorrect_ability_count_error;
            }
            
            // Alerting the user if the monster has more than one of any type of effect.
            let game_mechanic_names = fields.iter()
                .map(|it| {
                    path_to_ident(&it.game_mechanic_instance_path)
                })
                .collect::<Vec<_>>();
                
            let has_duplicate_names = (1..game_mechanic_names.len())
                .any(|i| game_mechanic_names[i..].contains(&game_mechanic_names[i - 1])
            );
            if has_duplicate_names {
                let duplicate_mechanic_error = Err(Error::new_spanned(
                    monster_instance_path.clone(),
                    format!(
                        "Error: More than one of any effect, i.e. move, ability etc. is not allowed. Please check if you have duplicated any attribute of {:?}", 
                        nickname_or_default,
                    ),
                ));
                return duplicate_mechanic_error;
            }
            return Ok(ExprMonster {
                monster_instance_path,
                monster_type_path,
                nickname_literal,
                fields,
                move_count,
            });
        } else {
            Err(Error::new_spanned(
                    monster_type.clone(),
                    format!(
                        "Error: Expected 'Monster' as the type parameter of the MonsterExpression, found {} instead.",
                        monster_type.to_string().as_str()
                        ),
                    )
                )
            }
        }
    }
    
    // Syntax:
    // <MoveName>: <PathToMoveType>
    // <AbilityName>: <PathToAbilityType>
    // <ItemName>: <PathToItemType>
    
    #[derive(Clone)]
    pub struct ExprGameMechanic {
        pub game_mechanic_instance_path: ExprPath,
        pub game_mechanic_type_path: ExprPath,
        pub game_mechanic_type: GameMechanicType
    }
    
    const MOVE_KEYWORD: &str = "Move";
    const ABILITY_KEYWORD: &str = "Ability";
    const ITEM_KEYWORD: &str = "Item";
    
    impl Parse for ExprGameMechanic {
        fn parse(input: ParseStream) -> Result<Self> {
            
            let game_mechanic_instance_path: ExprPath = input.parse()?;
            let _: Token![:] = input.parse()?;
            let game_mechanic_type_path: ExprPath = input.parse()?;
            let game_mechanic_type = path_to_ident(&game_mechanic_type_path);
            let game_mechanic_type = to_enum_value(game_mechanic_type, &game_mechanic_instance_path)?;
    
            Ok(ExprGameMechanic {
                game_mechanic_instance_path,
                game_mechanic_type_path,
                game_mechanic_type,
            })
        }
    }
    
    pub fn path_to_ident(expr_path: &ExprPath) -> Ident {
        expr_path
            .path
            .segments
            .last()
            .expect("There should be at least one segment in the path to the monster.")
            .ident
            .clone()
    }
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum GameMechanicType {
        Move,
        Ability,
        Item,
    }
    
    fn to_enum_value(game_mechanic_type: Ident, game_mechanic_instance_path: &ExprPath) -> Result<GameMechanicType> {
        let game_mechanic_type = match game_mechanic_type.to_string().as_str() {
            MOVE_KEYWORD => { GameMechanicType::Move },
            ABILITY_KEYWORD => { GameMechanicType::Ability },
            ITEM_KEYWORD => { GameMechanicType::Item },
            _ => return Err(Error::new_spanned(game_mechanic_instance_path, format!(
                "Error: Expected a valid Game Mechanic identifier, found {} instead.",
                game_mechanic_type.to_string().as_str()
            ))), 
        };
        Ok(game_mechanic_type)
    }
}

mod accessor_macro_syntax {
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

mod event_system_macro_syntax {
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