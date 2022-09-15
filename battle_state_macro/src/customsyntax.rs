use proc_macro2::Ident;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Error, Result,
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
            ally_team_fields: ally_team_expr.fields,
            opponent_team_fields: opponent_team_expr.fields,
        })
    }
}

#[derive(Clone, Debug)]
struct TeamExpr {
    team_ident: Ident,
    fields: Punctuated<MonsterExpr, Comma>,
}

impl Parse for TeamExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TeamExpr {
            team_ident: input.parse().expect(
                "Error: Failed to parse TeamExpression identifier in expected TeamExpression.",
            ),
            fields: parse_braced_comma_separated_list::<MonsterExpr>(input)?,
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

// Syntax looks like this:
// mon <MonsterName>  <Nickname> { <EffectExpression>, ... }
#[derive(Clone, Debug)]
pub struct MonsterExpr {
    pub monster_ident: Ident,
    pub nickname_ident: Ident,
    pub fields: Punctuated<EffectExpr, Comma>,
}

const MONSTER_KEYWORD: &str = "mon";

impl Parse for MonsterExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let monster_keyword: Ident = input
            .parse()
            .expect("Error: Failed to parse keyword in expected MonsterExpression.");
        let monster_ident: Ident = input
            .parse()
            .expect("Error: Failed to parse Monster identifier in expected MonsterExpression.");
        let nickname_ident: Ident =  input
            .parse()
            .expect("Error: Failed to parse Monster nickname in expected MonsterExpression");
        if is_expected_keyword(MONSTER_KEYWORD, &monster_keyword) {
            let fields = parse_braced_comma_separated_list::<EffectExpr>(input)?;

            return Ok(MonsterExpr {
                monster_ident,
                nickname_ident,
                fields,
            });
        } else {
            Err(Error::new_spanned(
                monster_keyword.clone(),
                format!(
                    "Error: Expected 'mon' at the start of a MonsterExpression, found {} instead.",
                    monster_keyword.to_string().as_str()
                ),
            ))
        }
    }
}

// mov <MoveName>
// abl <AbilityName>
// itm <ItemName>
#[derive(Clone, Debug)]
pub struct EffectExpr {
    pub effect_ident: Ident,
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
        let effect_ident: Ident = input
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
            effect_ident,
            effect_type,
        })
    }
}

#[derive(Clone, Debug)]
pub enum EffectType {
    Move,
    Ability,
    Item,
}
