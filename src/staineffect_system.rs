use std::i32;

use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

use crate::{components::{CombatStats, Explosion, InstantHarm, LingerType, LingeringEffect, Name, Position, Potion, ProvidesHealing, Puddle, SufferDamage, Teleport, Viewshed}, gamelog::GameLog, map::Map};

pub struct StainEffect {}

impl<'a> System<'a> for StainEffect {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        // ReadExpect<'a, Entity>,
                        // ReadStorage<'a, CombatStats>,
                        ReadStorage<'a, Puddle>,
                        Entities<'a>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteExpect<'a, Map>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Viewshed>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, SufferDamage>,
                        WriteExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadStorage<'a, Potion>,

                        WriteStorage<'a, ProvidesHealing>,
                        WriteStorage<'a, Teleport>,
                        WriteStorage<'a, LingeringEffect>,
                        WriteStorage<'a, InstantHarm>,
                        WriteStorage<'a, Explosion>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (mut combat, puddle, entities, mut rng, mut map, mut pos, mut viewsheds, mut log, names, mut suffer, mut playerpos, player_entity, potions,   mut heal, mut teleport, mut linger, mut harm, mut explosion) = data;

        for (ents, stat, _puddle) in (&entities, &mut combat, !&puddle).join() {
            // INFLICTS
            // Heal
            if let Some(healing) = heal.get(ents) {
                stat.hp = i32::min(stat.max_hp, stat.hp + healing.heal_amount);

                heal.remove(ents);
            }

            // Lingering effect (fire, poison)
            if linger.get(ents).is_some() {
                let LingeringEffect {etype, duration, dmg};
                {
                    let lingering = linger.get_mut(ents).unwrap();
                    lingering.duration -= 1;

                    etype = lingering.etype;
                    duration = lingering.duration;
                    dmg = lingering.dmg;
                }

                // TODO resistance
                #[cfg(debug_assertions)]
                log.entries.push(format!("{} is burning/poisoned!", names.get(ents).map_or("someone", |n| &n.name)));
                SufferDamage::new_damage(&mut suffer, ents, dmg);

                // fire spreads to adjacent mobs
                if etype == LingerType::Fire {
                    if let Some(mobpos) = pos.get(ents) {
                        let Position {x: mobx, y: moby} = mobpos;
                        for (x, y) in (-1..=1).flat_map(|x| (-1..=1).map(move |y| (x, y))).filter(|p| !(p.0 == 0 && p.1 == 0)) {
                            for adjent in map.tile_content[map.xy_idx(mobx+x, moby+y)].iter() {
                                // 50% chance to burn
                                if rng.roll_dice(1, 1) == 1 {
                                    linger.insert(*adjent, LingeringEffect { etype: LingerType::Fire, duration: 3, dmg })
                                          .expect("Unable to insert lingering fire on adjacent entities");
                                } 
                            }
                        }
                    }
                }

                if duration == 0 {
                    linger.remove(ents);
                } 

            }

            // instant harm
            if let Some(harming) = harm.get(ents) {
                #[cfg(debug_assertions)]
                log.entries.push(format!("{} suffers damage!", names.get(ents).map_or("someone", |n| &n.name)));
                SufferDamage::new_damage(&mut suffer, ents, harming.dmg);

                harm.remove(ents);
            }

            // explosion
            if let Some(exploding) = explosion.get(ents) {
                if let Some(mobpos) = pos.get(ents) {
                    let Position {x, y } = mobpos;
                    let mut blast_tiles = rltk::field_of_view(Point { x:*x, y:*y }, exploding.radius, &*map);
                    blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);

                    for tile in blast_tiles.iter() {
                        let idx = map.xy_idx(tile.x, tile.y);
                        for mob in map.tile_content[idx].iter() {
                            #[cfg(debug_assertions)]
                            log.entries.push(format!("{} gets caught in the explosion!", names.get(*mob).map_or("someone", |n| &n.name)));

                            let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                                Point {x: *x, y: *y}, 
                                    Point { x: tile.x, y: tile.y }
                            ).round().clamp(1.0, 999.0);
                            let dmg = exploding.maxdmg / (2.0f32 * distance) as i32;
                            SufferDamage::new_damage(&mut suffer, *mob, dmg);

                            if potions.get(*mob).is_some() && rng.roll_dice(1, 2) == 1 {
                                // entities.build_entity().with(c, storage)
                            }
                        }
                    }
                }

                explosion.remove(ents);
            }

            // Teleport
            if let Some(teleporting) = teleport.get(ents) {
                let mut x = rng.roll_dice(1, map.width-2)+1;
                let mut y = rng.roll_dice(1, map.height-2)+1;

                while map.tiles[map.xy_idx(x, y)] == crate::map::TileType::Wall && teleporting.safe {
                    x = rng.roll_dice(1, map.width-2)+1; 
                    y = rng.roll_dice(1, map.height-2)+1;
                }

                if let Some(mobpos) = pos.get_mut(ents) {
                    if ents == *player_entity {
                        playerpos.x = x;
                        playerpos.y = y;
                    }
                    mobpos.x = x;
                    mobpos.y = y;

                    log.entries.push(format!("{} teleports away!", names.get(ents).map_or("someone", |n| &n.name)));
                }

                viewsheds.get_mut(ents).map(|vs| vs.dirty = true);
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == crate::map::TileType::Wall {
                    stat.hp = 0;
                } else {
                    for mob in map.tile_content[idx].iter_mut() {
                        SufferDamage::new_damage(&mut suffer, *mob, i32::MAX);
                        let causer = names.get(ents).map_or("someone", |name| &name.name);
                        let victim = names.get(*mob).map_or("someone", |name| &name.name);
                        log.entries.push(format!("{causer} telefragged a poor {victim}."));
                    }
                }

                teleport.remove(ents);
            }
        }
    }
}