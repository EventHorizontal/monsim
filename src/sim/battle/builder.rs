use monsim_utils::{Ally, Count, MaxSizedVec, Opponent};
use tap::Pipe;

use crate::{
    effects,
    prng::Prng,
    sim::{
        game_mechanics::{Ability, AbilitySpecies, MonsterNature, MonsterSpecies, MoveSpecies, StatModifierSet, StatSet},
        targetting::{BoardPosition, FieldPosition},
    },
    AbilityID, Battle, Environment, Item, ItemID, ItemSpecies, Monster, MonsterID, MonsterTeam, Move, MoveCategory, MoveID, MoveNumber, PerTeam, Stat, TeamID,
    Terrain, TerrainSpecies, Weather, WeatherSpecies, ALLY_1, ALLY_2, ALLY_3, ALLY_4, ALLY_5, ALLY_6, OPPONENT_1, OPPONENT_2, OPPONENT_3, OPPONENT_4,
    OPPONENT_5, OPPONENT_6,
};

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
    environment: EnvironmentBuilder,
    format: BattleFormat,
}

impl Battle {
    pub fn spawn() -> BattleBuilder {
        BattleBuilder {
            maybe_ally_team: None,
            maybe_opponent_team: None,
            format: BattleFormat::Single,
            environment: Environment::spawn(),
        }
    }
}

impl BattleBuilder {
    pub fn with_ally_team(mut self, ally_team_builder: MonsterTeamBuilder) -> Self {
        assert!(self.maybe_ally_team.is_none(), "Only one Ally Team is allowed per battle, but found multiple.");
        self.maybe_ally_team = Some(Ally::new(ally_team_builder));
        self
    }

    pub fn with_opponent_team(mut self, opponent_team: MonsterTeamBuilder) -> Self {
        assert!(
            self.maybe_opponent_team.is_none(),
            "Only one Opponent Team is allowed per battle, but found multiple."
        );
        self.maybe_opponent_team = Some(Opponent::new(opponent_team));
        self
    }

    pub fn with_environment(mut self, environment: EnvironmentBuilder) -> Self {
        self.environment = environment;
        self
    }

    pub fn with_format(mut self, battle_format: BattleFormat) -> Self {
        self.format = battle_format;
        self
    }

    pub fn build(self) -> Battle {
        let mut prng = Prng::from_current_time();

        let ally_board_positions = self.format.ally_board_positions();

        const ALLY_IDS: [MonsterID; 6] = [ALLY_1, ALLY_2, ALLY_3, ALLY_4, ALLY_5, ALLY_6];

        let ally_team = self
            .maybe_ally_team
            .expect("Building the BattleState requires adding an Ally Team, found none.")
            .map_consume(|ally_team_builder| ally_team_builder.build(ALLY_IDS, ally_board_positions, TeamID::Allies));

        let opponent_board_positions = self.format.opponent_board_positions();

        const OPPONENT_IDS: [MonsterID; 6] = [OPPONENT_1, OPPONENT_2, OPPONENT_3, OPPONENT_4, OPPONENT_5, OPPONENT_6];

        let opponent_team = self
            .maybe_opponent_team
            .expect("Building the BattleState requires adding an Opponent Team, found none.")
            .map_consume(|opponent_team_builder| opponent_team_builder.build(OPPONENT_IDS, opponent_board_positions, TeamID::Opponents));

        let environment = self.environment.build(&mut prng);

        Battle::new(ally_team, opponent_team, environment, self.format, prng)
    }
}

pub struct MonsterTeamBuilder {
    maybe_monsters: Option<MaxSizedVec<MonsterBuilder, 6>>,
}

impl MonsterTeam {
    pub fn spawn() -> MonsterTeamBuilder {
        MonsterTeamBuilder { maybe_monsters: None }
    }
}

impl MonsterTeamBuilder {
    pub fn with_monster(mut self, monster: MonsterBuilder) -> Self {
        match self.maybe_monsters {
            Some(ref mut monsters) => {
                monsters.push(monster);
            }
            None => {
                self.maybe_monsters = Some(MaxSizedVec::from_vec(vec![monster]));
            }
        }
        self
    }

    fn build(self, monster_ids: [MonsterID; 6], board_positions: [BoardPosition; 6], team_id: TeamID) -> MonsterTeam {
        self.maybe_monsters
            .unwrap_or_else(|| panic!("Expected {team_id} to have at least one monster, but none were given"))
            .into_iter()
            .zip(monster_ids)
            .zip(board_positions)
            .map(|((monster_builder, monster_id), board_position)| monster_builder.build(monster_id, board_position))
            .collect::<Vec<_>>()
            .pipe(|monsters| MonsterTeam::new(monsters, team_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleFormat {
    Single,
    Double,
    Triple,
}

impl BattleFormat {
    pub(crate) fn valid_positions(&self) -> Vec<FieldPosition> {
        match self {
            BattleFormat::Single => {
                vec![FieldPosition::AllySideCentre, FieldPosition::OpponentSideCentre]
            }
            BattleFormat::Double => {
                vec![
                    FieldPosition::AllySideCentre,
                    FieldPosition::AllySideRight,
                    FieldPosition::OpponentSideCentre,
                    FieldPosition::OpponentSideRight,
                ]
            }
            BattleFormat::Triple => {
                vec![
                    FieldPosition::AllySideLeft,
                    FieldPosition::AllySideCentre,
                    FieldPosition::AllySideRight,
                    FieldPosition::OpponentSideLeft,
                    FieldPosition::OpponentSideCentre,
                    FieldPosition::OpponentSideRight,
                ]
            }
        }
    }

    fn ally_board_positions(&self) -> [BoardPosition; 6] {
        match self {
            BattleFormat::Single => [
                BoardPosition::Field(FieldPosition::AllySideCentre),
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
            ],
            BattleFormat::Double => [
                BoardPosition::Field(FieldPosition::AllySideCentre),
                BoardPosition::Field(FieldPosition::AllySideRight),
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
            ],
            BattleFormat::Triple => [
                BoardPosition::Field(FieldPosition::AllySideCentre),
                BoardPosition::Field(FieldPosition::AllySideLeft),
                BoardPosition::Field(FieldPosition::AllySideRight),
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
            ],
        }
    }

    fn opponent_board_positions(&self) -> [BoardPosition; 6] {
        match self {
            BattleFormat::Single => [
                BoardPosition::Field(FieldPosition::OpponentSideCentre),
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
            ],
            BattleFormat::Double => [
                BoardPosition::Field(FieldPosition::OpponentSideCentre),
                BoardPosition::Field(FieldPosition::OpponentSideRight),
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
            ],
            BattleFormat::Triple => [
                BoardPosition::Field(FieldPosition::OpponentSideCentre),
                BoardPosition::Field(FieldPosition::OpponentSideLeft),
                BoardPosition::Field(FieldPosition::OpponentSideRight),
                BoardPosition::Bench,
                BoardPosition::Bench,
                BoardPosition::Bench,
            ],
        }
    }
}

#[derive(Clone)]
pub struct MonsterBuilder {
    species: &'static MonsterSpecies,
    moves: MaxSizedVec<MoveBuilder, 4>,
    ability: AbilityBuilder,
    item: Option<ItemBuilder>,
    nickname: Option<&'static str>,
    _level: Option<u16>,
    _nature: Option<MonsterNature>,
    _stat_modifiers: Option<StatModifierSet>,
    current_health: Option<u16>,
}

pub trait MonsterBuilderExt {
    fn spawn(&'static self, moves: (MoveBuilder, Option<MoveBuilder>, Option<MoveBuilder>, Option<MoveBuilder>), ability: AbilityBuilder) -> MonsterBuilder;
}

impl MonsterBuilderExt for MonsterSpecies {
    fn spawn(&'static self, moves: (MoveBuilder, Option<MoveBuilder>, Option<MoveBuilder>, Option<MoveBuilder>), ability: AbilityBuilder) -> MonsterBuilder {
        Monster::with(self, moves, ability)
    }
}

impl Monster {
    /// Starting point for building a Monster.
    pub fn with(
        species: &'static MonsterSpecies,
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
            });

        let moves = MaxSizedVec::from_vec(moves);

        MonsterBuilder {
            species,
            moves,
            ability,
            nickname: None,
            _level: None,
            _nature: None,
            _stat_modifiers: None,
            current_health: None,
            item: None,
        }
    }
}

impl MonsterBuilder {
    pub fn with_nickname(mut self, nickname: &'static str) -> Self {
        self.nickname = Some(nickname);
        self
    }

    pub fn with_item(mut self, item: ItemBuilder) -> Self {
        self.item = Some(item);
        self
    }

    pub fn with_hitpoints(mut self, hitpoints: u16) -> MonsterBuilder {
        assert!(hitpoints <= Monster::calculate_max_health(self.species.base_stat(Stat::Hp), 31, 252, 50));
        self.current_health = Some(hitpoints);
        self
    }

    pub fn build(self, monster_id: MonsterID, board_position: BoardPosition) -> Monster {
        let nickname = self.nickname;

        let move_ids: [MoveID; 4] = [
            MoveID {
                owner_id: monster_id,
                move_number: MoveNumber::_1,
            },
            MoveID {
                owner_id: monster_id,
                move_number: MoveNumber::_2,
            },
            MoveID {
                owner_id: monster_id,
                move_number: MoveNumber::_3,
            },
            MoveID {
                owner_id: monster_id,
                move_number: MoveNumber::_4,
            },
        ];

        let moveset = self
            .moves
            .into_iter()
            .zip(move_ids)
            .map(|(move_builder, move_id)| move_builder.build(move_id))
            .collect::<Vec<_>>()
            .pipe(MaxSizedVec::from_vec);

        let ability = self.ability.build(AbilityID { owner_id: monster_id });

        let level = 50;
        // FEATURE: EVs and IVs should be settable through the builder.
        const IVS: StatSet = StatSet::new(31, 31, 31, 31, 31, 31);
        const EVS: StatSet = StatSet::new(252, 252, 252, 252, 252, 252);
        // In-game hp-stat determination formula
        let nature = MonsterNature::Serious;

        let held_item = self.item.map(|item| item.build(ItemID { item_holder_id: monster_id }));

        let max_health = Monster::calculate_max_health(self.species.base_stat(Stat::Hp), 31, 252, level);
        Monster {
            id: monster_id,
            species: self.species,
            nickname,
            effort_values: EVS,
            current_health: self.current_health.unwrap_or(max_health),
            individual_values: IVS,
            level,
            nature,
            board_position,
            stat_modifiers: StatModifierSet::blank(),
            moveset,
            ability,
            persistent_status: None,
            volatile_statuses: MaxSizedVec::empty(),
            held_item,
            consumed_item: None,
        }
    }
}

#[derive(Clone)]
pub struct MoveBuilder {
    species: &'static MoveSpecies,
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
        MoveBuilder { species, power_points: None }
    }
}

impl MoveBuilder {
    pub fn with_power_points(mut self, power_points: u8) -> MoveBuilder {
        assert!(
            power_points < self.species.max_power_points(),
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
        if species.category() == MoveCategory::Status && species.on_hit_effect() as usize == effects::deal_calculated_damage as usize {
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
    pub species: &'static AbilitySpecies,
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
        AbilityBuilder { species }
    }
}

// This implementation doesn't really afford us anything extra, but if
// Abilities become more complicated in the future, this will scale better.
impl AbilityBuilder {
    fn build(self, id: AbilityID) -> Ability {
        Ability { id, species: self.species }
    }
}

#[derive(Clone)]
pub struct ItemBuilder {
    pub species: &'static ItemSpecies,
}

pub trait ItemBuilderExt {
    fn spawn(&'static self) -> ItemBuilder;
}

impl ItemBuilderExt for ItemSpecies {
    fn spawn(&'static self) -> ItemBuilder {
        Item::builder(self)
    }
}

impl Item {
    pub fn builder(species: &'static ItemSpecies) -> ItemBuilder {
        ItemBuilder { species }
    }
}

impl ItemBuilder {
    pub fn build(self, id: ItemID) -> Item {
        Item { id, species: self.species }
    }
}

// Environment -------------------------------------------- //

#[derive(Clone)]
pub struct EnvironmentBuilder {
    pub maybe_weather: Option<WeatherBuilder>,
    pub maybe_terrain: Option<TerrainBuilder>,
}

impl Environment {
    pub fn spawn() -> EnvironmentBuilder {
        EnvironmentBuilder {
            maybe_weather: None,
            maybe_terrain: None,
        }
    }
}

impl WeatherSpecies {
    pub fn spawn(&'static self) -> WeatherBuilder {
        WeatherBuilder { species: self }
    }
}

impl EnvironmentBuilder {
    pub fn with_weather(mut self, species: &'static WeatherSpecies) -> Self {
        self.maybe_weather = Some(WeatherBuilder { species });
        self
    }

    pub fn with_terrain(mut self, species: &'static TerrainSpecies) -> Self {
        self.maybe_terrain = Some(TerrainBuilder { species });
        self
    }

    pub fn build(self, prng: &mut Prng) -> Environment {
        Environment {
            weather: self.maybe_weather.map(|weather_builder| weather_builder.build(prng)),
            terrain: self.maybe_terrain.map(|terrain_builder| terrain_builder.build(prng)),
            traps: PerTeam::new(Ally::new(None), Opponent::new(None)),
        }
    }
}

#[derive(Clone)]
pub struct WeatherBuilder {
    pub species: &'static WeatherSpecies,
}

impl WeatherBuilder {
    pub fn build(self, prng: &mut Prng) -> Weather {
        let remaining_turns = self.species.lifetime_in_turns().init(prng);
        Weather {
            species: self.species,
            remaining_turns,
        }
    }
}

#[derive(Clone)]
pub struct TerrainBuilder {
    pub species: &'static TerrainSpecies,
}

impl TerrainBuilder {
    pub fn build(self, prng: &mut Prng) -> Terrain {
        let remaining_turns = self.species.lifetime_in_turns().init(prng);
        Terrain {
            species: self.species,
            remaining_turns,
        }
    }
}

pub trait InitCount {
    fn init(&self, prng: &mut Prng) -> u8;
}

impl InitCount for Count {
    fn init(&self, prng: &mut Prng) -> u8 {
        match *self {
            Count::Fixed(number) => number,
            Count::RandomInRange { min, max } => prng.roll_random_number_in_range(min as u16..=max as u16) as u8,
        }
    }
}
