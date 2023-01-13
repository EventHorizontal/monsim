use super::*;

use std::{
    iter::Chain,
    slice::{Iter, IterMut}, fmt::Display,
};

type BattlerIterator<'a> = Chain<Iter<'a, Option<Battler>>, Iter<'a, Option<Battler>>>;
type MutableBattlerIterator<'a> = Chain<IterMut<'a, Option<Battler>>, IterMut<'a, Option<Battler>>>;

#[test]
fn test_priority_sorting_deterministic() {

    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let mut test_bcontext = bcontext!(
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
                    if let Some (handler) = OnTryMove.associated_handler(&event_handler_set_info.event_handler_set) {
                        Some(EventHandlerInfo {
                            event_handler: handler,
                            owner_uid: event_handler_set_info.owner_uid,
                            activation_order: event_handler_set_info.activation_order,
                            filters: EventHandlerFilters::default(),
                        })
                    } else{
                        None
                    }
                }
            )
            .collect::<Vec<_>>();

        test_bcontext.priority_sort::<bool>(&mut unwrapped_event_handler_plus_info);
        
        result[i] = unwrapped_event_handler_plus_info.into_iter().map(|event_handler_info| {
            test_bcontext.read_monster(event_handler_info.owner_uid).nickname
        }).collect::<Vec<_>>();
    }

    assert_eq!(result[0], result[1]);    
    assert_eq!(result[0][0], "Drifblim");
    assert_eq!(result[0][1], "Emerald");
    assert_eq!(result[0][2], "Ruby");
    assert_eq!(result[0][3], "Sapphire");
        
}

#[test]
fn test_event_filtering_for_event_sources() {

    let test_bcontext = bcontext!(
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
        BattlerUID { team_id: TeamID::Ally, battler_number: BattlerNumber::First }, 
        BattlerUID { team_id: TeamID::Opponent, battler_number: BattlerNumber::First }, 
        EventHandlerFilters::default(),
    );
    assert!(passed_filter);   
}

#[test]
fn test_priority_sorting_with_speed_ties() {

    let mut result = [Vec::new(), Vec::new()];
    for i in 0..=1 {
        let mut test_bcontext = bcontext!(
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
                    if let Some (handler) = OnTryMove.associated_handler(&event_handler_set_info.event_handler_set) {
                        Some(EventHandlerInfo {
                            event_handler: handler,
                            owner_uid: event_handler_set_info.owner_uid,
                            activation_order: event_handler_set_info.activation_order,
                            filters: EventHandlerFilters::default(),
                        })
                    } else{
                        None
                    }
                }
            )
            .collect::<Vec<_>>();

        test_bcontext.priority_sort::<bool>(&mut unwrapped_event_handler_plus_info);
        
        result[i] =  unwrapped_event_handler_plus_info.into_iter().map(|event_handler_info| {
            test_bcontext.read_monster(event_handler_info.owner_uid).nickname
        }).collect::<Vec<_>>();
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
    pub state: BattleState,
    pub prng: LCRNG,
    pub ally_team: MonsterTeam,
    pub opponent_team: MonsterTeam,
}

#[test]
fn test_display_battle_context() {
    let test_bcontext = bcontext!(
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
            self.opponent_team.battlers().iter().flatten().count()
        ];
        
        for k in 0..=1 {
            out.push_str(team_names[k]);
            for (i, battler) in teams[k].battlers().iter().flatten().enumerate() {
                let is_not_last_monster = i < number_of_monsters[k] - 1;
                if is_not_last_monster {
                    out.push_str("\t├── ");
                    out.push_str(format![
                        "{} the {} ({}) [HP: {}/{}]\n", 
                        battler.monster.nickname, 
                        battler.monster.species.name, 
                        battler.uid, 
                        battler.monster.current_health, 
                        battler.monster.max_health
                    ].as_str());
                    out.push_str("\t│\t│\n");
                    let number_of_effects = battler.moveset.moves().flatten().count();
                    out.push_str("\t│\t├── ");
        
                    out.push_str(format!["type {:?}/{:?} \n", battler.monster.species.primary_type, battler.monster.species.secondary_type].as_str());
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
                    out.push_str(format![
                        "{} the {} ({}) [HP: {}/{}]\n", 
                        battler.monster.nickname, 
                        battler.monster.species.name, 
                        battler.uid, 
                        battler.monster.current_health, 
                        battler.monster.max_health
                    ].as_str());
                    out.push_str("\t\t│\n");
                    let number_of_effects = battler.moveset.moves().flatten().count();
                    out.push_str("\t\t├──");
                    
                    out.push_str(format!["type {:?}/{:?} \n", battler.monster.species.primary_type, battler.monster.species.secondary_type].as_str());
                    out.push_str("\t\t├──");
        
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
            state: BattleState::UsingMove { 
                move_uid: MoveUID {
                    battler_uid: BattlerUID { 
                        team_id: TeamID::Ally, 
                        battler_number: BattlerNumber::First 
                    },
                    move_number: MoveNumber::First,

                },
                target_uid: BattlerUID { 
                    team_id: TeamID::Opponent, 
                    battler_number: BattlerNumber::First 
                } 
            },
            prng: LCRNG::new(prng::seed_from_time_now()),
            ally_team,
            opponent_team,
        }
    }

    fn battlers(&self) -> BattlerIterator {
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
            .find(|it| { it.uid == battler_uid })
            .expect("Error: Requested look up for a monster with ID that does not exist in this battle.")
    } 

    pub fn is_battler_on_field(&self, battler_uid: BattlerUID) -> bool {
        self.find_battler(battler_uid).on_field   
    }
    
    pub fn active_battler(&self) -> &Battler {
        if let BattleState::UsingMove { move_uid, target_uid: _ } = self.state {
            let battler_uid = move_uid.battler_uid;
            self.battlers()
            .flatten()
            .find(|it| it.uid == battler_uid)
            .expect(format!["Theres should exist a monster with id {:?}", battler_uid].as_str())
        } else {
            panic!("The battle is not calculating a move.")
        }
    }

    pub fn is_active_battler(&self, test_monster_uid: BattlerUID) -> bool {
        if let BattleState::UsingMove { move_uid, target_uid } = self.state {
            move_uid.battler_uid == test_monster_uid || target_uid == test_monster_uid
        } else {
            false
        }
    }

    pub fn write_monster(&mut self, uid: BattlerUID, change: &mut dyn FnMut(Monster) -> Monster) -> () {
        let maybe_battler = self.battlers_mut()
            .flatten()
            .find(|it| it.uid == uid);
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
            .expect(format!["There should be a move in the {:?} slot.", move_uid.move_number].as_str())
    }

    pub fn event_handler_sets_plus_info(&self) -> EventHandlerSetInfoList {
        let mut out = Vec::new();
        out.append(&mut self.ally_team.event_handlers());
        out.append(&mut self.opponent_team.event_handlers());
        out
    }

    /// Shuffles the event handler order for consecutive speed-tied monsters in place.
    pub(crate) fn priority_sort<R: Clone+Copy>(&mut self, vector: &mut EventHandlerInfoList<R>) {
        
        // Sort without resolving speed ties, this sorting is stable, so it doesn't affect the order of condition-wise equal elements.
        vector.sort_by(|a, b| {
                a.activation_order.cmp(&b.activation_order) 
            }
        );
        vector.reverse();
        
        let vector_length = vector.len();
        match vector_length.cmp(&2) {
            std::cmp::Ordering::Less => (),
            std::cmp::Ordering::Equal => {
                self.resolve_speed_tie(vector, &mut vec![0, 1]);
            },
            std::cmp::Ordering::Greater => {
                let mut tied_monster_indices: Vec<usize> = vec![0];
                // If there are more than two items, iterated through the 2nd through last index of the vector, comparing each item to the previous one.
                for i in 1..vector_length {
                    let previous_item = vector[i-1].activation_order;
                    let this_item = vector[i].activation_order;
                    // If the item we are looking at has the same speed as the previous, add its index to the tied queue.
                    if previous_item == this_item { // TODO: Investigate whether this should be `previous_item == this_item` instead
                        tied_monster_indices.push(i);
                        if i == (vector_length - 1) {
                            self.resolve_speed_tie(vector, &mut tied_monster_indices);
                        }
                    // If the priority or speed of the last item is higher, sort the current tied items using the PRNG and then reset the tied queue.
                    } else if previous_item > this_item {
                        self.resolve_speed_tie(vector, &mut tied_monster_indices);
                        tied_monster_indices = vec![i];
                    }
                }
            },
        }
    }

    fn resolve_speed_tie<R: Clone+Copy>(&mut self, vector: &mut EventHandlerInfoList<R>, tied_monster_indices: &mut Vec<usize>) {
        if tied_monster_indices.len() < 2 {
            return;
        }
        let mut i: usize = 0;
        let vector_copy = vector.clone(); 
        let offset = tied_monster_indices[0];
        'iteration_over_tied_indices: while tied_monster_indices.len() > 0 {
            let number_tied = tied_monster_indices.len() as u16;
            // Roll an n-sided die and put the monster corresponding to the roll at the front of the tied order.
            let prng_roll = self.prng.generate_number_in_range(0..=number_tied-1) as usize;
            vector[i+offset] = vector_copy[tied_monster_indices.remove(prng_roll)];
            // Once there is only one remaining tied monster, put it at the end of the queue.
            if tied_monster_indices.len() == 1 {
                vector[i+offset+1] = vector_copy[tied_monster_indices[0]];
                break 'iteration_over_tied_indices;
            }
            i += 1;
        }
    }

    pub(crate) fn filter_event_handlers(&self, event_caller_uid: BattlerUID, owner_uid: BattlerUID, event_handler_filters: EventHandlerFilters) -> bool {
        let bitmask = {
            let mut bitmask = 0b0000;
            if event_caller_uid == owner_uid { bitmask |= TargetFlags::SELF.bits() } // 0x01
            if self.are_allies(owner_uid, event_caller_uid) { bitmask |= TargetFlags::ALLIES.bits() } // 0x02
            if self.are_opponents(owner_uid, event_caller_uid) { bitmask |= TargetFlags::OPPONENTS.bits() } //0x04
            // TODO: When the Environment is implemented, add the environment to the bitmask. (0x08)
            bitmask
        };
        let event_source_filter_passed = event_handler_filters.whose_event.bits() == bitmask;
        let on_battlefield_passed = self.is_active_battler(owner_uid);

        event_source_filter_passed && on_battlefield_passed
    }

    fn is_on_ally_team(&self, uid: BattlerUID) -> bool {
        self.ally_team.battlers().iter()
            .flatten()
            .any(|it| { it.uid == uid })
    }

    fn is_on_opponent_team(&self, uid: BattlerUID) -> bool {
        self.opponent_team.battlers().iter()
            .flatten()
            .any(|it| { it.uid == uid })
    }
    
    fn are_opponents(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_opponent_team(event_caller_uid)) || 
        (self.is_on_ally_team(event_caller_uid) && self.is_on_opponent_team(owner_uid)) 
    }

    fn are_allies(&self, owner_uid: BattlerUID, event_caller_uid: BattlerUID) -> bool {
        if owner_uid == event_caller_uid {
            return false;
        }

        (self.is_on_ally_team(owner_uid) && self.is_on_ally_team(event_caller_uid)) || 
        (self.is_on_opponent_team(event_caller_uid) && self.is_on_opponent_team(owner_uid))
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