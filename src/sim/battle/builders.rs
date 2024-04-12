use monsim_utils::{Ally, MaxSizedVec, Opponent};
use tap::Pipe;

use crate::{sim::game_mechanics::{Ability, AbilitySpecies, MonsterNature, MonsterSpecies, MoveSpecies, StatModifierSet, StatSet}, AbilityUID, Battle, Monster, MonsterTeam, MonsterUID, Move, MoveUID, Stat, TeamUID, ALLY_1, ALLY_2, ALLY_3, ALLY_4, ALLY_5, ALLY_6, OPPONENT_1, OPPONENT_2, OPPONENT_3, OPPONENT_4, OPPONENT_5, OPPONENT_6};

// TODO: Some basic state validation will be done now, but later 
// on I want to extend that to more stuff, such as validating that 
// a certain monster species is allowed to have certain abilities etc. 
// Mostly that kind of thing where the user is alerted that the 
// combination of things they provided is not allowed.

pub struct BattleBuilder {
    maybe_ally_team: Option<Ally<MonsterTeamBuilder>>,
    maybe_opponent_team: Option<Opponent<MonsterTeamBuilder>>,
}

impl Battle {
    pub fn spawn() -> BattleBuilder {
        BattleBuilder { maybe_ally_team: None, maybe_opponent_team: None }
    }
}

impl BattleBuilder {
    pub fn add_ally_team(mut self, ally_team_builder: MonsterTeamBuilder) -> Self {
        assert!(self.maybe_ally_team.is_none(), "Only one Ally Team is allowed per battle, but found multiple.");
        self.maybe_ally_team = Some(Ally::new(ally_team_builder));
        self
    }

    pub fn add_opponent_team(mut self, opponent_team: MonsterTeamBuilder) -> Self {
        assert!(self.maybe_opponent_team.is_none(), "Only one Opponent Team is allowed per battle, but found multiple.");
        self.maybe_opponent_team = Some(Opponent::new(opponent_team));
        self
    }

    pub fn build(self) -> Battle {
        const ALLY_UIDS: [MonsterUID; 6] = [
            ALLY_1,
            ALLY_2,
            ALLY_3,
            ALLY_4,
            ALLY_5,
            ALLY_6,
        ];

        let ally_team = self.maybe_ally_team
            .expect("Building the BattleState requires adding an Ally Team, found none.")
            .map_consume(|ally_team_builder| {
                ally_team_builder.build(ALLY_UIDS, TeamUID::Allies)                
            });

        const OPPONENT_UIDS: [MonsterUID; 6] = [
            OPPONENT_1,
            OPPONENT_2,
            OPPONENT_3,
            OPPONENT_4,
            OPPONENT_5,
            OPPONENT_6,
        ];

        let opponent_team = self.maybe_opponent_team
            .expect("Building the BattleState requires adding an Opponent Team, found none.")
            .map_consume(|opponent_team_builder| {
                opponent_team_builder.build(OPPONENT_UIDS, TeamUID::Opponents)                
            });

        Battle::new(ally_team, opponent_team)
    }    
}

pub struct MonsterTeamBuilder {
    maybe_monsters: Option<MaxSizedVec<MonsterBuilder, 6>>
}

impl MonsterTeam {
    pub fn spawn() -> MonsterTeamBuilder {
        MonsterTeamBuilder {
            maybe_monsters: None
        }
    }
}

impl MonsterTeamBuilder {
    pub fn add_monster(mut self, monster: MonsterBuilder) -> Self {
        match self.maybe_monsters {
            Some(ref mut monsters) => {
                monsters.push(monster);
            },
            None => {
                self.maybe_monsters = Some(MaxSizedVec::from_vec(vec![monster]));
            },
        }
        self
    }

    fn build(self, monster_uids: [MonsterUID; 6], team_uid: TeamUID) -> MonsterTeam {
        self.maybe_monsters
            .expect(
                format!["Expected {team_uid} to have at least one monster, but none were given"].as_str()
            )
            .into_iter()
            .zip(monster_uids.into_iter())
            .map(|(monster_builder, monster_uid)| {
                monster_builder.build(monster_uid)
            })
            .collect::<Vec<_>>()
            .pipe(|monsters| MonsterTeam::new(monsters, team_uid))
    }
}

#[derive(Clone)]
pub struct MonsterBuilder {
    species: &'static MonsterSpecies,
    moves: MaxSizedVec<MoveBuilder, 4>,
    ability: AbilityBuilder,
    nickname: Option<&'static str>,
    level: Option<u16>,
    nature: Option<MonsterNature>,
    stat_modifiers: Option<StatModifierSet>,
    current_health: Option<u16>,
}

pub trait MonsterBuilderExt {
    fn spawn(&'static self, moves: (MoveBuilder, Option<MoveBuilder>, Option<MoveBuilder>, Option<MoveBuilder>), ability: AbilityBuilder) -> MonsterBuilder;
}

impl MonsterBuilderExt for MonsterSpecies {
    fn spawn(&'static self, moves: (MoveBuilder, Option<MoveBuilder>, Option<MoveBuilder>, Option<MoveBuilder>), ability: AbilityBuilder) -> MonsterBuilder {
        Monster::with(&self, moves, ability)
    }
}

impl Monster {
    /// Starting point for building a Monster.
    pub fn with(
        species:  &'static MonsterSpecies,
        moves: (MoveBuilder, Option<MoveBuilder>, Option<MoveBuilder>, Option<MoveBuilder>),
        ability: AbilityBuilder,
    ) -> MonsterBuilder {
        
        let moves = vec![moves.0]
            .pipe(|mut vec| {  
                if let Some(move_) = moves.1 {
                    vec.push(move_)
                }
                vec
            })
            .pipe(|mut vec| {  
                if let Some(move_) = moves.2 {
                    vec.push(move_)
                }
                vec
            })
            .pipe(|mut vec| {  
                if let Some(move_) = moves.3 {
                    vec.push(move_)
                }
                vec
            }
        );

        let moves = MaxSizedVec::from_vec(moves);
        
        MonsterBuilder {
            species,
            moves,
            ability,
            nickname: None,
            level: None,
            nature: None,
            stat_modifiers: None,
            current_health: None, 
        }
    }
}

const MAX_MOVES_PER_MOVESET: usize = 4;

impl MonsterBuilder {
    pub fn with_nickname(mut self, nickname: &'static str) -> Self {
        self.nickname = Some(nickname);
        self
    } 

    pub fn build(self, monster_uid: MonsterUID) -> Monster {
        
        let nickname = self.nickname;
        
        let move_uids: [MoveUID; 4] = [
            MoveUID { owner_uid: monster_uid, move_number: crate::MoveNumber::_1 },
            MoveUID { owner_uid: monster_uid, move_number: crate::MoveNumber::_2 },
            MoveUID { owner_uid: monster_uid, move_number: crate::MoveNumber::_3 },
            MoveUID { owner_uid: monster_uid, move_number: crate::MoveNumber::_4 },
        ];

        let moveset = self.moves
            .into_iter()
            .zip(move_uids.into_iter()).map(|(move_builder, move_uid)| {
                move_builder.build(move_uid)
            })
            .collect::<Vec<_>>()
            .pipe(|vec| { MaxSizedVec::from_vec(vec) });
        
        let ability = self.ability
            .build(monster_uid);
        
        let level = 50;
        // TODO: EVs and IVs are hardcoded for now. Decide what to do with this later.
        let iv_in_stat = 31;
        let ev_in_stat = 252;
        // In-game hp-stat determination formula
        let health_stat = ((2 * self.species.base_stats[Stat::Hp] + iv_in_stat + (ev_in_stat / 4)) * level) / 100 + level + 10;
        let nature = MonsterNature::Serious;

        // In-game non-hp-stat determination formula
        let get_non_hp_stat = |stat: Stat| -> u16 {
            // TODO: EVs and IVs are hardcoded for now. Decide what to do with this later.
            let iv_in_stat = 31;
            let ev_in_stat = 252;
            let mut out = ((2 * self.species.base_stats[stat] + iv_in_stat + (ev_in_stat / 4)) * level) / 100 + 5;
            out = f64::floor(out as f64 * nature[stat]) as u16;
            out
        };
        
        Monster {
            uid: monster_uid,
            nickname,
            level,
            max_health: health_stat,
            nature,
            current_health: health_stat,
            is_fainted: false,
            species: self.species,
            moveset,
            ability,
            stats: StatSet::new(
                health_stat,
                get_non_hp_stat(Stat::PhysicalAttack),
                get_non_hp_stat(Stat::PhysicalDefense),
                get_non_hp_stat(Stat::SpecialAttack),
                get_non_hp_stat(Stat::SpecialDefense),
                get_non_hp_stat(Stat::Speed),
            ),
            stat_modifiers: StatModifierSet::new(
                0,
                0,
                0,
                0,
                0,
            ),
        } 
    }
}

#[derive(Clone)]
pub struct MoveBuilder {
    species: & 'static MoveSpecies,
    power_points: Option<u8>,
}

pub trait MoveBuilderExt {
    fn spawn(&'static self) -> MoveBuilder;
}

impl MoveBuilderExt for MoveSpecies {
    fn spawn(&'static self) -> MoveBuilder {
        Move::builder(self)
    }
}

impl Move {
    pub fn builder(species: &'static MoveSpecies) -> MoveBuilder {
        MoveBuilder {
            species,
            power_points: None,
        }
    }
}

impl MoveBuilder {
    pub fn with_power_points(mut self, power_points: u8) -> MoveBuilder {
        assert!(power_points < self.species.max_power_points, 
            "Expected move {move_name} to have less than {max_pp} power points",
            move_name = self.species.name,
            max_pp = self.species.max_power_points,
        ); 
        self.power_points = Some(power_points);
        self
    }

    fn build(self, move_uid: MoveUID) -> Move {
        let species = self.species;
        Move {
            uid: move_uid,
            species,
            base_accuracy: species.base_accuracy,
            base_power: species.base_power,
            category: species.category,
            power_points: self.power_points.unwrap_or(species.max_power_points),
            priority: species.priority,
            type_: species.type_,
        }
    } 
}

#[derive(Clone)]
pub struct AbilityBuilder {
    pub species: & 'static AbilitySpecies,
}

pub trait AbilityBuilderExt {
    fn spawn(&'static self) -> AbilityBuilder;
}

impl AbilityBuilderExt for AbilitySpecies {
    fn spawn(&'static self) -> AbilityBuilder {
        Ability::builder(self)
    }
}

impl Ability {
    /// Starting point for building an Ability.
    pub fn builder(species: &'static AbilitySpecies) -> AbilityBuilder {
        AbilityBuilder {
            species,
        }
    }
}

// This implementation doesn't really afford us anything extra, but if
// Abilities become more complicated in the future, this will scale better.
impl AbilityBuilder {
    fn build(self, uid: AbilityUID) -> Ability {
        Ability { 
            uid, 
            species: self.species 
        }
    }
}