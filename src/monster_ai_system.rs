use rltk::{Point, RandomNumberGenerator, RGB};
use specs::{ReadStorage, System};
use specs::prelude::*;

use crate::components::{Agitated, Bomber, Boss, BossState, Confusion, Explosion, InstantHarm, Item, LingerType, LingeringEffect, Monster, Name, Position, Potion, Renderable, SufferDamage, Viewshed, WantsToMelee, WantsToThrowItem};
use crate::map::Map;
use crate::{inventory_system, RunState};

pub struct MonsterAI { }

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, WantsToThrowItem>,
                        WriteStorage<'a, SufferDamage>,
                        WriteStorage<'a, Confusion>,
                        WriteStorage<'a, Agitated>,
                        ReadStorage<'a, Bomber>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities, mut viewshed, monster, mut position, mut want_melee, mut want_throw, mut suffer, mut confused, mut agitated, bombers) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, viewshed, _monster, pos) in (&entities, &mut viewshed, &monster, &mut position).join() {

            let mut is_agitated = true;
            let mut can_act = true;

            if let Some(agitation) = agitated.get_mut(entity) {
                agitation.turns -= 1;
                if agitation.turns < 1 {
                    agitated.remove(entity);
                }
            } else {
                if viewshed.visible_tiles.contains(&*player_pos) {
                    agitated.insert(entity, Agitated { turns: 5 }).expect("Unable to agitate enemy");
                } 
                is_agitated = false;
            }

            if let Some(confusion) = confused.get_mut(entity) {
                confusion.turns -= 1;
                if confusion.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                // AI
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);

                if distance < 1.5 {
                    if let Some(bomber) = bombers.get(entity) {
                        // kamikadze
                        want_throw.insert(entity, WantsToThrowItem { item: bomber.effect, target: *player_pos }).expect("Unable to kamikadze player");
                        SufferDamage::new_damage(&mut suffer, entity, std::i32::MAX);
                    } else {
                        want_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Unable to insert attack on player");
                    }
                } else if viewshed.visible_tiles.contains(&*player_pos) || is_agitated {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &*map
                    );
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            } 
        }
    }
}


pub struct BossAI {}

impl<'a> System<'a> for BossAI {
    type SystemData = (WriteStorage<'a, Boss>,
                       WriteExpect<'a, Map>,
                       Entities<'a>,
                       ReadExpect<'a, Entity>,
                       ReadExpect<'a, Point>,
                       ReadExpect<'a, RunState>,
                       WriteStorage<'a, Viewshed>,
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, WantsToMelee>,
                       WriteStorage<'a, Confusion>,
                       WriteStorage<'a, WantsToThrowItem>,
                       WriteStorage<'a, Potion>,
                       WriteStorage<'a, Item>,
                       WriteStorage<'a, Renderable>,
                       WriteExpect<'a, RandomNumberGenerator>,
                       WriteStorage<'a, LingeringEffect>,
                       WriteStorage<'a, InstantHarm>,
                       WriteStorage<'a, Explosion>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut boss, mut map, entities, player_entity, player_pos, runstate, mut viewsheds, mut positions, mut want_melee, mut confused, mut intentthrow, mut potions, mut items, mut renders, mut rng, mut linger, mut harm, mut explosion) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, viewshed, pos, boss) in (&entities, &mut viewsheds, &mut positions, &mut boss).join() {
            let mut can_act = true;
            if let Some(confusion) = confused.get_mut(entity) {
                confusion.turns -= 1;
                if confusion.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                // AI
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);

                if distance < 1.5 {
                    want_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Unable to insert attack on player");
                } else {
                    match boss.state {
                        BossState::ClosingIn(_) => {
                            dbg!("closing in");
                            let path = rltk::a_star_search(
                                map.xy_idx(pos.x, pos.y) as i32,
                                map.xy_idx(player_pos.x, player_pos.y) as i32,
                                &*map
                            );
                            if path.success && path.steps.len() > 1 {
                                let mut idx = map.xy_idx(pos.x, pos.y);
                                map.blocked[idx] = false;
                                pos.x = path.steps[1] as i32 % map.width;
                                pos.y = path.steps[1] as i32 / map.width;
                                idx = map.xy_idx(pos.x, pos.y);
                                map.blocked[idx] = true;
                            }
                            boss.state = state_table(boss.state, distance);
                        },
                        BossState::GainingDistance(_) => {
                            dbg!("gain distance");
                            // TODO run away
                            boss.state = state_table(boss.state, distance);
                        },
                        BossState::ThrowingPotions(turns) => {
                            if turns % 4 == 1 { 
                                let potion = entities.create();

                                potions.insert(potion, Potion {}).expect("Unable to insert boss potion");
                                items.insert(potion, Item {}).expect("Unable to insert boss potion item");

                                let color;
                                match rng.roll_dice(1, 16) {
                                    1..=4 => {
                                        explosion.insert(potion, Explosion { maxdmg: 10, radius: 4 }).expect("Unable to insert boss potion explosion");
                                        color = RGB::named(rltk::ORANGE);
                                    }
                                    5..=12 => {
                                        harm.insert(potion, InstantHarm { dmg: 7 }).expect("Unable to insert boss potion harm");
                                        color = RGB::named(rltk::DARKRED);
                                    }
                                    _ => {
                                        let etype = match rng.roll_dice(1, 2) {
                                            1 => {
                                                color = RGB::named(rltk::RED);
                                                LingerType::Fire
                                            },
                                            _ => {
                                                color = RGB::named(rltk::GREEN);
                                                LingerType::Poison
                                            },
                                        };
                                        linger.insert(potion, LingeringEffect { etype, duration: 5, dmg: 3 }).expect("Unable to insert boss potion linger");
                                    }
                                }

                                renders.insert(potion, Renderable { 
                                    glyph: rltk::to_cp437('!'), 
                                    fg: color,
                                    bg: RGB::named(rltk::BLACK), 
                                    render_order: 2 
                                }).expect("Unable to insert boss potion render");

                                intentthrow.insert(*player_entity, WantsToThrowItem { item: potion, target: *player_pos }).expect("Unable to insert boss throw intent");
                            }

                            dbg!("throwing potions");
                            boss.state = state_table(boss.state, distance);
                        },
                    }

                    viewshed.dirty = true;
                }
            }

        }
    }
}

const LBOUND: f32 = 5.0;
const RBOUND: f32 = 7.0;

fn state_table(prev_state: BossState, distance: f32) -> BossState {
    
    let lower_than_lbound = distance <= LBOUND;
    let higher_than_rbound = distance >= RBOUND;
    match prev_state {
        BossState::ThrowingPotions(0) => { BossState::ClosingIn(5) },
        BossState::ClosingIn(0)       => {
            if lower_than_lbound {
                BossState::GainingDistance(5)
            } else {
                BossState::ThrowingPotions(12)
            }
        },
        BossState::GainingDistance(0) => {
            if lower_than_lbound {
                BossState::ClosingIn(5)
            } else {
                BossState::ThrowingPotions(12)
            }
        },

        BossState::ThrowingPotions(n) => {
            if lower_than_lbound {
                BossState::ClosingIn(5)
            } else {
                BossState::ThrowingPotions(n-1)
            }
        },
        BossState::ClosingIn(n) => BossState::ClosingIn(n-1),
        BossState::GainingDistance(n) => {
            if higher_than_rbound {
                BossState::ThrowingPotions(12)
            } else {
                BossState::GainingDistance(n-1)
            }
        },
    }
}