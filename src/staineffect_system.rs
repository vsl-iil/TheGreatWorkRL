use rltk::RandomNumberGenerator;
use specs::prelude::*;

use crate::{components::{CombatStats, Name, Position, ProvidesHealing, Puddle, Stained, Teleport, Viewshed}, gamelog::GameLog, map::Map};

pub struct StainEffect {}

impl<'a> System<'a> for StainEffect {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        // ReadExpect<'a, Entity>,
                        WriteStorage<'a, Stained>,
                        ReadStorage<'a, Puddle>,
                        Entities<'a>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteExpect<'a, Map>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Viewshed>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, Name>,

                        WriteStorage<'a, ProvidesHealing>,
                        WriteStorage<'a, Teleport>,
                      );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut stained, puddle, entities, mut rng, mut map, mut pos, mut viewsheds, mut log, names, mut heal, mut teleport) = data;

        for (ents, _stain, _puddle) in (&entities, &mut stained, !&puddle).join() {
            // INFLICTS
            // Heal
            if let Some(healing) = heal.get(ents) {
                if let Some(stat) = stats.get_mut(ents) {
                    stat.hp = i32::min(stat.max_hp, stat.hp + healing.heal_amount);
                }

                heal.remove(ents);
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
                    mobpos.x = x;
                    mobpos.y = y;
                }

                viewsheds.get_mut(ents).map(|vs| vs.dirty = true);
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == crate::map::TileType::Wall {
                    if let Some(stat) = stats.get_mut(ents) {
                        stat.hp = 0;
                    }
                } else {
                    for mob in map.tile_content[idx].iter_mut() {
                        if let Some(mobstat) = stats.get_mut(*mob) {
                            mobstat.hp = 0;
                            let causer = names.get(ents).map_or("someone", |name| &name.name);
                            let victim = names.get(*mob).map_or("someone", |name| &name.name);
                            log.entries.push(format!("{causer} telefragged a poor {victim}."));
                        }
                    }
                }

                teleport.remove(ents);
            }

            // Inficts
            
        }

        stained.clear();
    }
}