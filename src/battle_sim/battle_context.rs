use super::*;

use std::{
    fmt::Display,
    iter::Chain,
    slice::{Iter, IterMut},
};

type BattlerIterator<'a> = Chain<Iter<'a, Option<Battler>>, Iter<'a, Option<Battler>>>;
type MutableBattlerIterator<'a> = Chain<IterMut<'a, Option<Battler>>, IterMut<'a, Option<Battler>>>;

#[test]
fn test_priority_sorting_deterministic() {
    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let mut test_bcontext = bcontext_internal!(
            {
                AllyTeam {
                    mon Torchic "Ruby" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Mudkip "Sapphire" {
                        mov Tackle,
                        mov Bubble,
                        abl FlashFire,
                    },
                    mon Treecko "Emerald" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                },
                OpponentTeam {
                    mon Drifblim {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                }
            }
        );

        let event_handler_set_plus_info = test_bcontext.event_handler_sets_plus_info();
        use super::event::event_dex::OnTryMove;
        let mut unwrapped_event_handler_plus_info = event_handler_set_plus_info
            .iter()
            .filter_map(|event_handler_set_info| {
                if let Some(handler) =
                    OnTryMove.associated_handler(&event_handler_set_info.event_handler_set)
                {
                    Some(EventHandlerInfo {
                        event_handler: handler,
                        owner_uid: event_handler_set_info.owner_uid,
                        activation_order: event_handler_set_info.activation_order,
                        filters: EventHandlerFilters::default(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Battle::priority_sort::<EventHandlerInfo<bool>>(
            &mut test_bcontext.prng,
            &mut unwrapped_event_handler_plus_info,
            &mut |it| it.activation_order,
        );

        result[i] = unwrapped_event_handler_plus_info
            .into_iter()
            .map(|event_handler_info| {
                test_bcontext
                    .read_monster(event_handler_info.owner_uid)
                    .nickname
            })
            .collect::<Vec<_>>();
    }

    assert_eq!(result[0], result[1]);
    assert_eq!(result[0][0], "Drifblim");
    assert_eq!(result[0][1], "Emerald");
    assert_eq!(result[0][2], "Ruby");
    assert_eq!(result[0][3], "Sapphire");
}

#[test]
fn test_event_filtering_for_event_sources() {
    let test_bcontext = bcontext_internal!(
        {
            AllyTeam {
                mon Torchic "Ruby" {
                    mov Ember,
                    mov Scratch,
                    abl FlashFire,
                },
                mon Mudkip "Sapphire" {
                    mov Tackle,
                    mov Bubble,
                    abl FlashFire,
                },
            },
            OpponentTeam {
                mon Treecko "Emerald" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            }
        }
    );

    let passed_filter = test_bcontext.filter_event_handlers(
        BattlerUID {
            team_id: TeamID::Ally,
            battler_number: BattlerNumber::First,
        },
        BattlerUID {
            team_id: TeamID::Opponent,
            battler_number: BattlerNumber::First,
        },
        EventHandlerFilters::default(),
    );
    assert!(passed_filter);
}

#[test]
fn test_priority_sorting_with_speed_ties() {
    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let mut test_bcontext = bcontext_internal!(
            {
                AllyTeam {
                    mon Torchic "A" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "B" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "C" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "D" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "E" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Mudkip "F" {
                        mov Tackle,
                        mov Bubble,
                        abl FlashFire,
                    }
                },
                OpponentTeam {
                    mon Drifblim "G" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "H" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "I" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "J" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "K" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                    mon Torchic "L" {
                        mov Scratch,
                        mov Ember,
                        abl FlashFire,
                    },
                }
            }
        );
        test_bcontext.prng = LCRNG::new(i as u64);

        let event_handler_set_plus_info = test_bcontext.event_handler_sets_plus_info();
        use super::event::event_dex::OnTryMove;
        let mut unwrapped_event_handler_plus_info = event_handler_set_plus_info
            .iter()
            .filter_map(|event_handler_set_info| {
                if let Some(handler) =
                    OnTryMove.associated_handler(&event_handler_set_info.event_handler_set)
                {
                    Some(EventHandlerInfo {
                        event_handler: handler,
                        owner_uid: event_handler_set_info.owner_uid,
                        activation_order: event_handler_set_info.activation_order,
                        filters: EventHandlerFilters::default(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Battle::priority_sort::<EventHandlerInfo<bool>>(
            &mut test_bcontext.prng,
            &mut unwrapped_event_handler_plus_info,
            &mut |it| it.activation_order,
        );

        result[i] = unwrapped_event_handler_plus_info
            .into_iter()
            .map(|event_handler_info| {
                test_bcontext
                    .read_monster(event_handler_info.owner_uid)
                    .nickname
            })
            .collect::<Vec<_>>();
    }

    // Check that the two runs are not equal, there is an infinitesimal chance they won't be, but the probability is negligible.
    assert_ne!(result[0], result[1]);
    // Check that Drifblim is indeed the in the front.
    assert_eq!(result[0][0], "G");
    // Check that the Torchics are all in the middle.
    for name in ["A", "B", "C", "D", "E", "H", "I", "J", "K", "L"].iter() {
        assert!(result[0].contains(name));
    }
    //Check that the Mudkip is last.
    assert_eq!(result[0][11], "F");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleContext {
    pub current_action: ActionChoice,
    pub state: BattleState,
    pub prng: LCRNG,
    pub ally_team: MonsterTeam,
    pub opponent_team: MonsterTeam,
}

#[test]
fn test_display_battle_context() {
    let test_bcontext = bcontext_internal!(
        {
            AllyTeam {
                mon Torchic "Ruby" {
                    mov Ember,
                    mov Scratch,
                    abl FlashFire,
                },
                mon Mudkip "Sapphire" {
                    mov Tackle,
                    mov Bubble,
                    abl FlashFire,
                },
                mon Treecko "Emerald" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            },
            OpponentTeam {
                mon Drifloon "Cheerio" {
                    mov Scratch,
                    mov Ember,
                    abl FlashFire,
                },
            }
        }
    );
    println!("{}", test_bcontext);
}

impl Display for BattleContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        let teams = [self.ally_team, self.opponent_team];
        let team_names = ["Ally Team\n", "Opponent Team\n"];
        let number_of_monsters = [
            self.ally_team.battlers().iter().flatten().count(),
            self.opponent_team.battlers().iter().flatten().count(),
        ];

        for k in 0..=1 {
            out.push_str(team_names[k]);
            for (i, battler) in teams[k].battlers().iter().flatten().enumerate() {
                let is_not_last_monster = i < number_of_monsters[k] - 1;
                if is_not_last_monster {
                    out.push_str("\t├── ");
                    out.push_str(
                        format![
                            "{} the {} ({}) [HP: {}/{}]\n",
                            battler.monster.nickname,
                            battler.monster.species.name,
                            battler.uid,
                            battler.monster.current_health,
                            battler.monster.max_health
                        ]
                        .as_str(),
                    );
                    out.push_str("\t│\t│\n");
                    let number_of_effects = battler.moveset.moves().flatten().count();
                    out.push_str("\t│\t├── ");

                    out.push_str(
                        format![
                            "type {:?}/{:?} \n",
                            battler.monster.species.primary_type,
                            battler.monster.species.secondary_type
                        ]
                        .as_str(),
                    );
                    out.push_str("\t│\t├── ");
                    out.push_str(format!["abl {}\n", battler.ability.species.name].as_str());

                    for (j, move_) in battler.moveset.moves().flatten().enumerate() {
                        let is_not_last_move = j < number_of_effects - 1;
                        if is_not_last_move {
                            out.push_str("\t│\t├── ");
                        } else {
                            out.push_str("\t│\t└── ");
                        }
                        out.push_str(format!["mov {}\n", move_.species.name].as_str());
                    }
                    out.push_str("\t│\t\n");
                } else {
                    out.push_str("\t└── ");
                    out.push_str(
                        format![
                            "{} the {} ({}) [HP: {}/{}]\n",
                            battler.monster.nickname,
                            battler.monster.species.name,
                            battler.uid,
                            battler.monster.current_health,
                            battler.monster.max_health
                        ]
                        .as_str(),
                    );
                    out.push_str("\t\t│\n");
                    let number_of_effects = battler.moveset.moves().flatten().count();
                    out.push_str("\t\t├── ");

                    out.push_str(
                        format![
                            "type {:?}/{:?} \n",
                            battler.monster.species.primary_type,
                            battler.monster.species.secondary_type
                        ]
                        .as_str(),
                    );
                    out.push_str("\t\t├── ");

                    out.push_str(format!["abl {}\n", battler.ability.species.name].as_str());

                    for (j, move_) in battler.moveset.moves().flatten().enumerate() {
                        let is_not_last_move = j < number_of_effects - 1;
                        if is_not_last_move {
                            out.push_str("\t\t├── ");
                        } else {
                            out.push_str("\t\t└── ");
                        }
                        out.push_str(format!["mov {}\n", move_.species.name].as_str());
                    }
                    out.push_str("\t\t\n");
                }
            }
        }
        write!(f, "{}", out)
    }
}

impl BattleContext {
    pub fn new(ally_team: MonsterTeam, opponent_team: MonsterTeam) -> Self {
        Self {
            current_action: ActionChoice::None,
            state: BattleState::UsingMove {
                move_uid: MoveUID {
                    battler_uid: BattlerUID {
                        team_id: TeamID::Ally,
                        battler_number: BattlerNumber::First,
                    },
                    move_number: MoveNumber::First,
                },
                target_uid: BattlerUID {
                    team_id: TeamID::Opponent,
                    battler_number: BattlerNumber::First,
                },
            },
            prng: LCRNG::new(prng::seed_from_time_now()),
            ally_team,
            opponent_team,
        }
    }

    pub fn battlers(&self) -> BattlerIterator {
        let left = self.ally_team.battlers();
        let right = self.opponent_team.battlers();

        left.into_iter().chain(right)
    }

    fn battlers_mut(&mut self) -> MutableBattlerIterator {
        let left = self.ally_team.battlers_mut();
        let right = self.opponent_team.battlers_mut();

        left.into_iter().chain(right)
    }

    pub fn read_monster(&self, uid: BattlerUID) -> Monster {
        self.battlers()
            .flatten()
            .find(|it| it.uid == uid)
            .expect(format!["Theres should exist a monster with id {:?}", uid].as_str())
            .monster
    }

    fn find_battler(&self, battler_uid: BattlerUID) -> &Battler {
        self.battlers()
            .flatten()
            .find(|it| it.uid == battler_uid)
            .expect(
            "Error: Requested look up for a monster with ID that does not exist in this battle.",
        )
    }

    pub fn is_battler_on_field(&self, battler_uid: BattlerUID) -> bool {
        self.find_battler(battler_uid).on_field
    }

    pub fn current_action_user(&self) -> &Battler {
        self.find_battler(self.current_action.chooser())
    }

    pub fn is_current_action_user(&self, test_monster_uid: BattlerUID) -> bool {
        test_monster_uid == self.current_action.chooser()
    }

    pub(crate) fn current_action_target(&self) -> &Battler {
        self.find_battler(self.current_action.target())
    }

    pub(crate) fn is_current_action_target(&self, test_monster_uid: BattlerUID) -> bool {
        test_monster_uid == self.current_action.target()
    }

    pub fn write_monster(
        &mut self,
        uid: BattlerUID,
        change: &mut dyn FnMut(Monster) -> Monster,
    ) -> () {
        let maybe_battler = self.battlers_mut().flatten().find(|it| it.uid == uid);
        if let Some(battler) = maybe_battler {
            let new_monster = change(battler.monster);
            battler.monster = new_monster;
        };
    }

    pub fn read_ability(&self, owner_uid: BattlerUID) -> Ability {
        self.battlers()
            .flatten()
            .find(|it| it.uid == owner_uid)
            .expect(format!["Theres should exist a monster with id {:?}", owner_uid].as_str())
            .ability
    }

    pub fn read_move(&self, move_uid: MoveUID) -> Move {
        let owner_uid = move_uid.battler_uid;
        self.battlers()
            .flatten()
            .find(|it| it.uid == owner_uid)
            .expect(format!["Theres should exist a monster with id {:?}", owner_uid].as_str())
            .moveset
            .move_(move_uid.move_number)
            .expect(
                format![
                    "There should be a move in the {:?} slot.",
                    move_uid.move_number
                ]
                .as_str(),
            )
    }

    pub fn event_handler_sets_plus_info(&self) -> EventHandlerSetInfoList {
        let mut out = Vec::new();
        out.append(&mut self.ally_team.event_handlers());
        out.append(&mut self.opponent_team.event_handlers());
        out
    }

    pub(crate) fn filter_event_handlers(
        &self,
        event_caller_uid: BattlerUID,
        owner_uid: BattlerUID,
        event_handler_filters: EventHandlerFilters,
    ) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if event_caller_uid == owner_uid {
                bitmask |= TargetFlags::SELF.bits()
            } // 0x01
            if self.are_allies(owner_uid, event_caller_uid) {
                bitmask |= TargetFlags::ALLIES.bits()
            } // 0x02
            if self.are_opponents(owner_uid, event_caller_uid) {
                bitmask |= TargetFlags::OPPONENTS.bits()
            } //0x04
              // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = event_handler_filters.whose_event.bits() == bitmask;
        let on_battlefield_passed = self.is_battler_on_field(owner_uid);

        event_source_filter_passed && on_battlefield_passed
    }

    fn is_on_ally_team(&self, uid: BattlerUID) -> bool {
        self.ally_team
            .battlers()
            .iter()
            .flatten()
            .any(|it| it.uid == uid)
    }

    fn is_on_opponent_team(&self, uid: BattlerUID) -> bool {
        self.opponent_team
            .battlers()
            .iter()
            .flatten()
            .any(|it| it.uid == uid)
    }

    fn are_opponents(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_opponent_team(event_caller_uid))
            || (self.is_on_ally_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    fn are_allies(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_ally_team(event_caller_uid))
            || (self.is_on_opponent_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
    }

    pub fn battlers_on_field(&self) -> Vec<&Battler> {
        self.battlers()
            .flatten()
            .filter(|it| it.on_field)
            .collect::<Vec<_>>()
    }

    /// Given an action choice, computes its activation order. This is handled by BattleContext because the order is
    /// context dependent.
    pub(crate) fn choice_activation_order(&self, choice: ActionChoice) -> ActivationOrder {
        match choice {
            ActionChoice::Move {
                move_uid,
                target_uid: _,
            } => ActivationOrder {
                priority: self.read_move(move_uid).species.priority,
                speed: self.read_monster(move_uid.battler_uid).stats[Stat::Speed],
                order: 0,
            },
            ActionChoice::None => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleState {
    ChoosingActions,
    UsingMove {
        move_uid: MoveUID,
        target_uid: BattlerUID,
    },
    Finished,
}
