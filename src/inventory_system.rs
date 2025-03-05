use rltk::{Point, RandomNumberGenerator};
use specs::prelude::*;

use crate::{components::{Agitated, AreaOfEffect, Boss, CombatStats, Confusion, Consumable, Explosion, InBackpack, InflictsDamage, InstantHarm, LingeringEffect, Name, Position, ProvidesHealing, SufferDamage, Teleport, Viewshed, WantsToDropItem, WantsToPickupItem, WantsToThrowItem, WantsToUseItem, Weight}, gamelog::GameLog, map::Map};

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>,
                        ReadStorage<'a, Boss>,
                        // WriteExpect<'a, Map>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut log, mut wants_pickup, mut pos, name, mut backpack, boss) = data;

        for pickup in wants_pickup.join() {
            pos.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to insert backpack entry");
            
            if pickup.collected_by == *player_entity {
                if boss.get(pickup.item).is_some() {
                    log.entries.push("You obtained the Philosopher's Stone!".to_owned());
                } else {
                    let mut log_name = "something";
                    if let Some(item_name) = name.get(pickup.item) {
                        log_name = &item_name.name;
                    }
                    log.entries.push(format!("You picked up {}.", log_name));
                }
            }
        }

        wants_pickup.clear();
    }
}


pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, ProvidesHealing>,
                        ReadStorage<'a, InflictsDamage>,
                        WriteStorage<'a, Confusion>,
                        WriteStorage<'a, Teleport>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, Consumable>,
                        WriteStorage<'a, CombatStats>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteExpect<'a, Map>
                    );

 fn run(&mut self, data: Self::SystemData) {
    let (player_entity, mut gamelog, entities, mut want_use, names, mut viewsheds, healing, damaging, mut confusion, teleport, mut playerpos, aoe, mut suffering, consumables, mut combat_stats, mut rng, mut map) = data;

    for (entity, usable) in (&entities, &want_use).join() {
        let mut targets = vec![];
        match usable.target {
            None => { targets.push(*player_entity)},
            Some(target) => {
                let area_effect = aoe.get(usable.item);
                match area_effect {
                    None => {
                        let idx = map.xy_idx(target.x, target.y);
                        for mob in map.tile_content[idx].iter() {
                            targets.push(*mob);
                        }
                    },
                    Some(area) => {
                        let mut blast_tiles = rltk::field_of_view(target, area.radius, &*map);
                        blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);
                        for tile_idx in blast_tiles.iter() {
                            let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                    }
                }
            }
        }

        let item_heals = healing.get(usable.item);
        match item_heals {
            None => {},
            Some(healer) => {
                for target in targets.iter() {
                    let stats = combat_stats.get_mut(*target);
                    if let Some(stats) = stats {
                        stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount*3);
                        if entity == *player_entity {
                            gamelog.entries.push(format!("You used the {}, healing {} hp.", names.get(usable.item).unwrap().name, healer.heal_amount*3));
                        }
                    }
                }
            }
        }

        let item_damages = damaging.get(usable.item);
        match item_damages {
            None => {},
            Some(damage) => {
                for mob in targets.iter() {
                    SufferDamage::new_damage(&mut suffering, *mob, damage.damage);
                    if entity == *player_entity {
                        let mut mob_name = "someone";
                        if let Some(mname) = names.get(*mob) {
                            mob_name = &mname.name;
                        }
                        let mut item_name = "something";
                        if let Some(iname) = names.get(usable.item) {
                            item_name = &iname.name;
                        }

                        gamelog.entries.push(format!("You used {} on {}, inflicting {} hp.", item_name, mob_name, damage.damage));
                    }
                }
            }
        }

        let mut add_confusion = vec![];
        let item_confuses = confusion.get(usable.item);
        match item_confuses {
            None => {},
            Some(confuse) => {
                for mob in targets.iter() {
                    add_confusion.push((*mob, confuse.turns));
                    if entity == *player_entity {
                        let mut mob_name = "someone";
                        if let Some(mname) = names.get(*mob) {
                            mob_name = &mname.name;
                        }
                        let mut item_name = "something";
                        if let Some(iname) = names.get(usable.item) {
                            item_name = &iname.name;
                        }

                        gamelog.entries.push(format!("You used {} on {}, confusing them.", item_name, mob_name));
                    }
                }
            }
        }

        for mob in add_confusion {
            confusion.insert(mob.0, Confusion { turns: mob.1 }).expect("Unable to insert confusion status");
        }

        let item_teleports = teleport.get(usable.item);
        match item_teleports {
            None => {},
            Some(teleporting) => {
                let mut x = rng.roll_dice(1, map.width-2)+1;
                let mut y = rng.roll_dice(1, map.height-2)+1;

                while map.tiles[map.xy_idx(x, y)] == crate::map::TileType::Wall && teleporting.safe {
                    x = rng.roll_dice(1, map.width-2)+1; 
                    y = rng.roll_dice(1, map.height-2)+1;
                }

                if let Some(player_pos) = playerpos.get_mut(*player_entity) {
                    player_pos.x = x;
                    player_pos.y = y;
                }

                viewsheds.get_mut(*player_entity).unwrap().dirty = true;
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == crate::map::TileType::Wall {
                    if let Some(stats) = combat_stats.get_mut(*player_entity) {
                        stats.hp = 0;
                        gamelog.entries.push("You teleported into a wall and suffocated.".to_string());
                    }
                } else {
                    for mob in map.tile_content[idx].iter_mut() {
                        if let Some(stats) = combat_stats.get_mut(*mob) {
                            stats.hp = 0;
                            gamelog.entries.push(format!("You telefragged a poor {}.", names.get(*mob).unwrap().name));
                        }
                    }
                }
            }
        }

        if consumables.get(usable.item).is_some() {
            entities.delete(usable.item).expect("Unable to delete consumable");
        }
    }

    want_use.clear();
 }
}


pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut log, entities, mut drop, mut pos, names, mut backpack) = data;

        for (entity, to_drop) in (&entities, &drop).join() {
            let mut dropper_pos = Position { x: 0, y: 0 };
            {
                let dropped_pos = pos.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            pos.insert(to_drop.item, dropper_pos).expect("Unable to insert drop position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                let mut item_name = "something";
                if let Some(name) = names.get(to_drop.item) {
                    item_name = &name.name;
                }
                log.entries.push(format!("You dropped the {}", item_name));
            }
        }

        drop.clear();
    }
}


pub struct ItemThrowSystem {}

impl<'a> System<'a> for ItemThrowSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( //ReadExpect<'a, Entity>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToThrowItem>,
                        WriteExpect<'a, Map>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, Weight>,
                        WriteStorage<'a, Agitated>,
                        // эффекты
                        WriteStorage<'a, ProvidesHealing>,
                        WriteStorage<'a, Teleport>,
                        WriteStorage<'a, LingeringEffect>,
                        WriteStorage<'a, InstantHarm>,
                        WriteStorage<'a, Explosion>,
                        WriteStorage<'a, Confusion>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut intentthrow, map, mut backpack, mut pos, mut suffer, weight, mut agitate,   mut healing, mut teleport, mut linger, mut harm, mut explosion, mut confusion) = data;

        for to_throw in (&mut intentthrow).join() {
            let Point {x, y} = to_throw.target;
            
            let mut is_inflictor = false;
            for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                // INFLICTS
                // Список эффектов: ProvidesHealing, Teleport, Lingering, Harm, Explosion

                // Heal
                if let Some(heal) = healing.get(to_throw.item) {
                    // let heal_amount = heal.heal_amount;
                    healing.insert(*mob, *heal).expect("Unable to apply healing inflict to entity");
                    is_inflictor = true;
                }

                // Teleport
                if let Some(tp) = teleport.get(to_throw.item) {
                    teleport.insert(*mob, *tp).expect("Unable to apply teleport inflict to entity");
                    is_inflictor = true;
                }

                // Lingering
                if let Some(lingering) = linger.get(to_throw.item) {
                    linger.insert(*mob, *lingering).expect("Unable to apply lingering inflict to entity");
                    is_inflictor = true;
                }

                // Instant damage
                if let Some(dmg) = harm.get(to_throw.item) {
                    harm.insert(*mob, *dmg).expect("Unable to apply harm inflict to entity");
                    is_inflictor = true;
                }

                // Explosion
                if let Some(boom) = explosion.get(to_throw.item) {
                    explosion.insert(*mob, *boom).expect("Unable to apply explosion inflict to entity");
                    is_inflictor = true;
                }

                // Confusion
                if let Some(confuse) = confusion.get(to_throw.item) {
                    confusion.insert(*mob, *confuse).expect("Unable to confuse entity");
                    is_inflictor = true;
                }

                // Эй, кто в меня кинул?!
                agitate.insert(*mob, Agitated { turns: 2 }).expect("Unable to agitate enemy after throw.");
            }
            
            // damage based on weight
            if let Some(target) = map.tile_content[map.xy_idx(x, y)].first() {
                SufferDamage::new_damage(&mut suffer, *target, weight.get(to_throw.item).map_or(2, |w| w.0 * 2));
            }

            if is_inflictor {
                entities.delete(to_throw.item).expect("Unable to delete thrown entity");
            } else {
                backpack.remove(to_throw.item).expect("Unable to remove thrown item from backpack");
                let Point {x, y} = to_throw.target;
                pos.insert(to_throw.item, Position { x, y }).expect("Unable to place thrown item in position");
            }
        }
        intentthrow.clear();
    }
}