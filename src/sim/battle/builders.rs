use monsim_utils::MaxSizedVec;

use crate::{sim::game_mechanics::{Ability, AbilitySpecies, MonsterNature, MonsterSpecies, MoveSpecies, StatModifierSet, StatSet}, BattleState, Monster, Move, Stat, ALLY_3};

// TODO: Some basic state validation will be done now, but later 
// on I want to extend that to more stuff, such as validating that 
// a certain monster species is allowed to have certain abilities etc. 
// Mostly that kind of thing where the user is alerted that the 
// combination of things they provided is not allowed.

pub struct BattleBuilder {
    ally_team: Option<Vec<Monster>>,
    opponent_team: Option<Vec<Monster>>,
}

impl BattleState {
    pub fn empty() -> BattleBuilder {
        BattleBuilder { ally_team: None, opponent_team: None }
    }
}

impl BattleBuilder {
    pub fn ally_team(&mut self, monsters: Vec<Monster>) -> &mut Self {
        todo!()
    }

    pub fn with_opponent_team(&mut self, monsters: Vec<Monster>) -> &mut Self {
        todo!()
    }    
}

pub struct MonsterBuilder {
    species: &'static MonsterSpecies,
    moves: Option<MaxSizedVec<Move, 4>>,
    ability: Option<Ability>,
    nickname: Option<&'static str>,
    level: Option<u16>,
    nature: Option<MonsterNature>,
    stat_modifiers: Option<StatModifierSet>,
    current_health: Option<u16>,
}

impl Monster {
    /// Starting point for building a Monster.
    pub fn of_species(species: &'static MonsterSpecies) -> MonsterBuilder {
        MonsterBuilder {
            species,
            moves: None,
            ability: None,
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
    pub fn with_move(&mut self, move_: Move) -> &mut Self {
        match self.moves {
            Some(ref mut moves) => { 
                assert!(moves.count() < MAX_MOVES_PER_MOVESET, 
                "Couldn't add {move_name}, {monster_name} already has {MAX_MOVES_PER_MOVESET}.",
                move_name = move_.species.name,
                monster_name = self.species.name,
            );
                moves.push(move_); 
            },
            None => { self.moves = Some(MaxSizedVec::from_vec(vec![move_]))},
        }
        self
    }

    pub fn build(self) -> Monster {
        
        let placeholder_uid = ALLY_3;
        let nickname = self.nickname;
        
        let moveset = self.moves.expect(format!["{} must be given 1-4 moves but none were given.", self.species.name].as_str());
        let ability = self.ability.expect(format!["{} must be given an ability but none were given", self.species.name].as_str());
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
            uid: placeholder_uid,
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

pub struct MoveBuilder {
    species: & 'static MoveSpecies,
    power_points: Option<u8>,
}

impl Move {
    pub fn of_species(species: &'static MoveSpecies) -> MoveBuilder {
        MoveBuilder {
            species,
            power_points: None,
        }
    }
}

impl MoveBuilder {
    pub fn with_power_points(&mut self, power_points: u8) -> &mut MoveBuilder {
        assert!(power_points < self.species.max_power_points, 
            "Expected move {move_name} to have less than {max_pp} power points",
            move_name = self.species.name,
            max_pp = self.species.max_power_points,
        ); 
        self.power_points = Some(power_points);
        self
    }

    pub fn build(self) -> Move {
        Move {
            species: self.species,
            base_accuracy: self.species.base_accuracy,
            base_power: self.species.base_power,
            category: self.species.category,
            power_points: self.power_points.unwrap_or(self.species.max_power_points),
            priority: self.species.priority,
            type_: self.species.type_,
        }
    } 
}

pub struct AbilityBuilder {
    pub species: & 'static AbilitySpecies,
}

impl Ability {
    /// Starting point for building an Ability.
    pub fn of_species(species: &'static AbilitySpecies) -> AbilityBuilder {
        AbilityBuilder {
            species,
        }
    }
}

// This implementation doesn't really afford us anything extra, but if
// Abilities become more complicated in the future, this will scale better.
impl AbilityBuilder {
    pub fn build(self) -> Ability {
        Ability {
            species: self.species,
        }
    } 
}

