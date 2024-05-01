use monsim_utils::{Ally, MaxSizedVec, Opponent};
use tap::Pipe;

use crate::{sim::{game_mechanics::{Ability, AbilitySpecies, MonsterNature, MonsterSpecies, MoveSpecies, StatModifierSet, StatSet}, targetting::{BoardPosition, FieldPosition}}, AbilityID, BattleState, DealDefaultDamage, Monster, MonsterID, MonsterNumber, MonsterTeam, Move, MoveCategory, MoveID, Stat, TeamID, ALLY_1, ALLY_2, ALLY_3, ALLY_4, ALLY_5, ALLY_6, OPPONENT_1, OPPONENT_2, OPPONENT_3, OPPONENT_4, OPPONENT_5, OPPONENT_6};

/*  
    FEATURE: Better Validation -> Some basic state validation will be done 
    now, but later on I want to extend that to more stuff, such as validating 
    that a certain monster species is allowed to have certain abilities etc. 
    Mostly that kind of thing where the user is alerted that the combination 
    of things they provided is not allowed. The thing to keep in mind here is
    that monsim will eventually run with a GUI, this is just the engine, and so
    the interactive UI will allow reporting this errors iteratively.
*/
pub struct BattleBuilder {
    maybe_ally_team: Option<Ally<MonsterTeamBuilder>>,
    maybe_opponent_team: Option<Opponent<MonsterTeamBuilder>>,
    format: BattleFormat
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleFormat {
    Single,
    Double,
    Triple,
}

impl BattleState {
    pub fn spawn() -> BattleBuilder {
        BattleBuilder { 
            maybe_ally_team: None, 
            maybe_opponent_team: None, 
            format: BattleFormat::Single
        }
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

    pub fn with_format(mut self, battle_format: BattleFormat) -> Self {
        self.format = battle_format;
        self
    }

    pub fn build(self) -> BattleState {
        
        let ally_board_positions = match self.format {
            BattleFormat::Single => {
                [
                    BoardPosition::Field(FieldPosition::AllyCentre),
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                ]
            },
            BattleFormat::Double => {
                [
                    BoardPosition::Field(FieldPosition::AllyCentre),
                    BoardPosition::Field(FieldPosition::AllyRight),
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                ]
            },
            BattleFormat::Triple => {
                [
                    BoardPosition::Field(FieldPosition::AllyCentre),
                    BoardPosition::Field(FieldPosition::AllyLeft),
                    BoardPosition::Field(FieldPosition::AllyRight),
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                ]
            },
        };
        
        const ALLY_IDS: [MonsterID; 6] = [
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
                ally_team_builder.build(ALLY_IDS, ally_board_positions, TeamID::Allies)                
            });

        let opponent_board_positions = match self.format {
            BattleFormat::Single => {
                [
                    BoardPosition::Field(FieldPosition::OpponentCentre),
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                ]
            },
            BattleFormat::Double => {
                [
                    BoardPosition::Field(FieldPosition::OpponentCentre),
                    BoardPosition::Field(FieldPosition::OpponentRight),
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                ]
            },
            BattleFormat::Triple => {
                [
                    BoardPosition::Field(FieldPosition::OpponentCentre),
                    BoardPosition::Field(FieldPosition::OpponentLeft),
                    BoardPosition::Field(FieldPosition::OpponentRight),
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                    BoardPosition::Bench,
                ]
            },
        };

        const OPPONENT_IDS: [MonsterID; 6] = [
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
                opponent_team_builder.build(OPPONENT_IDS, opponent_board_positions, TeamID::Opponents)                
            });

        BattleState::new(ally_team, opponent_team, self.format)
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

    fn build(self, monster_ids: [MonsterID; 6], board_positions: [BoardPosition; 6], team_id: TeamID) -> MonsterTeam {
        self.maybe_monsters
            .expect(
                format!["Expected {team_id} to have at least one monster, but none were given"].as_str()
            )
            .into_iter()
            .zip(monster_ids.into_iter())
            .zip(board_positions.into_iter())
            .map(|((monster_builder, monster_id), board_position)| {
                monster_builder.build(monster_id, board_position)
            })
            .collect::<Vec<_>>()
            .pipe(|monsters| MonsterTeam::new(monsters, team_id))
    }
}

#[derive(Clone)]
pub struct MonsterBuilder {
    species: &'static MonsterSpecies,
    moves: MaxSizedVec<MoveBuilder, 4>,
    ability: AbilityBuilder,
    nickname: Option<&'static str>,
    _level: Option<u16>,
    _nature: Option<MonsterNature>,
    _stat_modifiers: Option<StatModifierSet>,
    _current_health: Option<u16>,
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
            _level: None,
            _nature: None,
            _stat_modifiers: None,
            _current_health: None, 
        }
    }
}

impl MonsterBuilder {
    pub fn with_nickname(mut self, nickname: &'static str) -> Self {
        self.nickname = Some(nickname);
        self
    } 

    pub fn build(self, monster_id: MonsterID, board_position: BoardPosition) -> Monster {
        
        let nickname = self.nickname;
        
        let move_ids: [MoveID; 4] = [
            MoveID { owner_id: monster_id, move_number: crate::MoveNumber::_1 },
            MoveID { owner_id: monster_id, move_number: crate::MoveNumber::_2 },
            MoveID { owner_id: monster_id, move_number: crate::MoveNumber::_3 },
            MoveID { owner_id: monster_id, move_number: crate::MoveNumber::_4 },
        ];

        let moveset = self.moves
            .into_iter()
            .zip(move_ids.into_iter()).map(|(move_builder, move_id)| {
                move_builder.build(move_id)
            })
            .collect::<Vec<_>>()
            .pipe(|vec| { MaxSizedVec::from_vec(vec) });
        
        let ability = self.ability
            .build(AbilityID { owner_id: monster_id});
        
        let level = 50;
        // FEATURE: EVs and IVs should be settable through the builder.
        const IVS: StatSet = StatSet::new(31, 31, 31, 31, 31, 31);
        const EVS: StatSet = StatSet::new(252, 252, 252, 252, 252, 252);
        // In-game hp-stat determination formula
        let nature = MonsterNature::Serious;
        
        Monster {
            id: monster_id,
            nickname,
            effort_values: EVS,
            current_health: Monster::calculate_max_health(self.species.base_stat(Stat::Hp), 31, 252, level),
            individual_values: IVS,
            level,
            nature,
            stat_modifiers: StatModifierSet::new(0, 0, 0, 0, 0),
            species: self.species,
            moveset,
            ability,
            board_position,
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
        assert!(power_points < self.species.max_power_points(), 
            "Expected move {move_name} to have less than {max_pp} power points",
            move_name = self.species.name(),
            max_pp = self.species.max_power_points(),
        );
         
        self.power_points = Some(power_points);
        self
    }

    fn build(self, move_id: MoveID) -> Move {
        let species = self.species;
        // FEATURE: When the engine is more mature, we'd like to make warnings like this toggleable.
        if species.category() == MoveCategory::Status && species.
        on_use_effect() == DealDefaultDamage {
            println!("\n Warning: The user created move {} has been given the category \"Status\" but deals damage only. Consider changing its category to Physical or Special. If this is intentional, ignore this message.", species.name())
        }
        Move {
            id: move_id,
            
            current_power_points: self.power_points.unwrap_or(self.species.max_power_points()),
            species,
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
    fn build(self, id: AbilityID) -> Ability {
        Ability {
            id,
            species: self.species,
        }
    }
}