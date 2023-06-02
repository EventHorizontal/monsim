use proc_macro2::{Ident, Literal};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Error, Result, ExprPath,
};

#[derive(Clone, Debug)]
pub struct BattleStateExpr {
    pub ally_team_fields: Punctuated<MonsterExpr, Comma>,
    pub opponent_team_fields: Punctuated<MonsterExpr, Comma>,
}

impl Parse for BattleStateExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let battle_contents = parse_braced_comma_separated_list::<TeamExpr>(input)?;
        let mut battle_contents = battle_contents.iter();

        let mut team_id_checker = |name: &str| -> Result<TeamExpr> {
            let team_expr = battle_contents
                .next()
                .expect("Error: Failed to parse team identifier.")
                .clone();

            let team_ident = team_expr.team_ident.clone();

            if is_expected_keyword(name, &team_ident) {
                Ok(team_expr)
            } else {
                return Err(Error::new_spanned(
                    team_ident.clone(),
                    format!(
                        "Error: Expected Team identifier {}, found {}.",
                        name,
                        team_ident.to_string().as_str()
                    ),
                ));
            }
        };

        let ally_team_expr = team_id_checker("AllyTeam")?;
        let opponent_team_expr = team_id_checker("OpponentTeam")?;

        Ok(BattleStateExpr {
            ally_team_fields: ally_team_expr.monster_fields,
            opponent_team_fields: opponent_team_expr.monster_fields,
        })
    }
}

#[derive(Clone, Debug)]
struct TeamExpr {
    team_ident: Ident,
    monster_fields: Punctuated<MonsterExpr, Comma>,
}

impl Parse for TeamExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let team_ident: Ident = input.parse().expect(
            "Error: Failed to parse TeamExpression identifier in expected TeamExpression.",
        );
        let monster_fields = parse_braced_comma_separated_list::<MonsterExpr>(input)?;
        let monster_count = monster_fields.clone().iter().len();
        if monster_count > 6 || monster_count < 1 {
            return Err(Error::new_spanned(
                team_ident.clone(),
                format!(
                    "Error: You can put betweeen one and six monsters on a team. {:?} has {} monsters.", 
                    team_ident.to_string(),
                    monster_count,
                ),
            ));
        };

        Ok(TeamExpr {
            team_ident,
            monster_fields,
        })
    }
}

fn parse_braced_comma_separated_list<T: Parse>(input: ParseStream) -> Result<Punctuated<T, Comma>> {
    let content;
    braced!(content in input);
    content.parse_terminated::<T, Comma>(T::parse)
}

fn is_expected_keyword(expected_keyword_name: &str, keyword: &Ident) -> bool {
    let keyword_as_string = keyword.to_string();
    let keyword_as_str = keyword_as_string.as_str();
    keyword_as_str == expected_keyword_name
}

// Syntax:
// mon <MonsterName>  <Nickname> { <EffectExpression>, ... }
#[derive(Clone, Debug)]
pub struct MonsterExpr {
    pub monster_ident: ExprPath,
    pub nickname_ident: Option<Literal>,
    pub fields: Punctuated<EffectExpr, Comma>,
    pub move_count: usize,
}

const MONSTER_KEYWORD: &str = "mon";

impl Parse for MonsterExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        
        let monster_keyword: Ident = input
            .parse()
            .expect("Error: Failed to parse keyword in expected MonsterExpression.");
        
        let monster_ident: ExprPath = input
            .parse()
            .expect("Error: Failed to parse Monster identifier in expected MonsterExpression.");
        
        let nickname_ident =  input
            .parse::<Literal>();

        let nickname_ident = nickname_ident.ok();
        
        if is_expected_keyword(MONSTER_KEYWORD, &monster_keyword) {
        let fields = parse_braced_comma_separated_list::<EffectExpr>(input)?;
            
        // Alerting the user if the number of moves is greater than 4.
        let move_count = fields.iter()
            .filter(|it| { it.effect_type == EffectType::Move })
            .count();
        
        if move_count > 4 || move_count < 1 {
            return Err(Error::new_spanned(
                monster_ident.clone(),
                format!(
                    "Error: You can put betweeen one and four moves on a monster. {:?} has {} moves.", 
                    {
                        if let Some(v) = nickname_ident {
                            v.to_string()
                        } else {
                            format!("{:?}", monster_ident.path)
                        } 
                    },
                    move_count,
                ),
            ));
        };
        
        // Alerting the user if the number of abilities is greater than 1.
        let ability_count = fields.iter()
            .filter(|it| { it.effect_type == EffectType::Ability })
            .count();
        
        if ability_count != 1 {
            return Err(Error::new_spanned(
                monster_ident.clone(),
                format!(
                    "Error: You can only put one ability on a monster.{:?} has more than one ability.", 
                    {
                        if let Some(v) = nickname_ident {
                            v.to_string()
                        } else {
                            format!("{:?}", monster_ident.path)
                        } 
                    }
                ),
            ));
        }
        
        // Alerting the user if the monster has more than one of any type of effect.
        let effect_names = fields.iter()
            .map(|it| {
                it.effect_path.clone()
            })
            .collect::<Vec<_>>();
            
        let has_duplicate_names = (1..effect_names.len())
            .any(|i| effect_names[i..].contains(&effect_names[i - 1])
        );
        if has_duplicate_names {
            return Err(Error::new_spanned(
                monster_ident.clone(),
                format!(
                    "Error: More than one of any effect, i.e. move, ability etc. is not allowed. Please check if you have duplicated any attribute of {:?}", 
                    {
                        if let Some(v) = nickname_ident {
                            v.to_string()
                        } else {
                            format!("{:?}", monster_ident.path)
                        } 
                    }
                ),
            ));
        }
        return Ok(MonsterExpr {
            monster_ident,
            nickname_ident,
            fields,
            move_count,
        });
    } else {
        Err(Error::new_spanned(
                monster_keyword.clone(),
                format!(
                    "Error: Expected 'mon' at the start of a MonsterExpression, found {} instead.",
                    monster_keyword.to_string().as_str()
                    ),
                )
            )
        }
    }
}

// Syntax:
// mov <MoveName>
// abl <AbilityName>
// itm <ItemName>

/// By effect we mean anything that belongs to a monster such as abilities, moves and items.
#[derive(Clone, Debug)]
pub struct EffectExpr {
    pub effect_path: ExprPath,
    pub effect_type: EffectType,
}

const MOVE_KEYWORD: &str = "mov";
const ABILITY_KEYWORD: &str = "abl";
const ITEM_KEYWORD: &str = "itm";

impl Parse for EffectExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let effect_keyword: Ident = input
            .parse()
            .expect("Error: Failed to parse keyword in expected EffectExpression.");
        let effect_ident: ExprPath = input
            .parse()
            .expect("Error: Failed to parse Effect identifier in expected MonsterExpression.");
        let effect_type;
        if is_expected_keyword(MOVE_KEYWORD, &effect_keyword) {
            effect_type = EffectType::Move;
        } else if is_expected_keyword(ABILITY_KEYWORD, &effect_keyword) {
            effect_type = EffectType::Ability;
        } else if is_expected_keyword(ITEM_KEYWORD, &effect_keyword) {
            effect_type = EffectType::Item;
        } else {
            return Err(Error::new_spanned(
                effect_keyword.clone(),
                format!(
                    "Error: Expected a valid Effect identifier, found {} instead.",
                    effect_keyword.to_string().as_str()
                ),
            ));
        }
        Ok(EffectExpr {
            effect_path: effect_ident,
            effect_type,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectType {
    Move,
    Ability,
    Item,
}
