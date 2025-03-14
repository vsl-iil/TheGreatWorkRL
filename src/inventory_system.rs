use rltk::{Point, RandomNumberGenerator, RGB};
use specs::prelude::*;

use crate::{components::{Agitated, AreaOfEffect, CombatStats, Confusion, Consumable, Explosion, InBackpack, InflictsDamage, InstantHarm, Invulnerability, LingeringEffect, MacGuffin, Name, Position, Potion, ProvidesHealing, Puddle, Renderable, Strength, SufferDamage, Teleport, Viewshed, WantsToDropItem, WantsToPickupItem, WantsToThrowItem, WantsToUseItem, Weight}, gamelog::GameLog, map::Map, particle_system::ParticleBuilder};

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>,
                        ReadStorage<'a, MacGuffin>,
                        WriteExpect<'a, bool>
                        // WriteExpect<'a, Map>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut log, mut wants_pickup, mut pos, name, mut backpack, boss, mut player_won) = data;

        for pickup in wants_pickup.join() {
            pos.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to insert backpack entry");
            
            if pickup.collected_by == *player_entity {
                if boss.contains(pickup.item) {
                    log.entries.push("You obtained the Philosopher's Stone!".to_owned());
                    *player_won = true;
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
                        WriteStorage<'a, InstantHarm>,
                        WriteStorage<'a, LingeringEffect>,
                        WriteStorage<'a, Explosion>,
                        WriteStorage<'a, Invulnerability>,
                        WriteStorage<'a, Strength>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, Consumable>,
                        WriteStorage<'a, CombatStats>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteExpect<'a, Map>,
                        WriteExpect<'a, ParticleBuilder>
                    );

 fn run(&mut self, data: Self::SystemData) {
    let (player_entity, mut gamelog, entities, mut want_use, names, mut viewsheds, healing, damaging, mut confusion, teleport, mut harm, mut linger, mut explosion, mut invuln, mut strength, mut playerpos, aoe, mut suffering, consumables, mut combat_stats, mut rng, mut map, mut pbuilder) = data;

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

        let item_harms = harm.get(usable.item).copied();
        match item_harms {
            None => {},
            Some(item_harms) => {
                for target in targets.iter() {
                    harm.insert(*target, item_harms).expect("Unable to harm used");
                }
            }
        }

        let item_lingers = linger.get(usable.item).copied();
        match item_lingers {
            None => {},
            Some(item_lingers) => {
                for target in targets.iter() {
                    linger.insert(*target, item_lingers).expect("Unable to linger used");
                }
            }
        }

        let item_explodes = explosion.get(usable.item).copied();
        match item_explodes {
            None => {},
            Some(item_explodes) => {
                for target in targets.iter() {
                    explosion.insert(*target, item_explodes).expect("Unable to explode used");
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
                        if let Some(pos) = playerpos.get(entity) {
                            pbuilder.request(pos.x, pos.y, RGB::named(rltk::RED), RGB::named(rltk::BLACK), rltk::to_cp437('♥'), 200.0);
                        }
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
                    if let Some(pos) = playerpos.get(entity) {
                        pbuilder.request(pos.x, pos.y, RGB::named(rltk::PINK), RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
                    }
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

        let item_gives_invul = invuln.get(usable.item).copied();
        match item_gives_invul {
            None => {},
            Some(invul) => {
                for target in targets.iter() {
                    if combat_stats.contains(*target) {
                        invuln.insert(*target, invul).expect("Unable to insert invulnerabilty on target");
                        if *target == *player_entity {
                            gamelog.entries.push("You are invulnerable!".to_owned());
                        }
                    }
                }
            }
        }

        let item_gives_strength = strength.get(usable.item).copied();
        match item_gives_strength {
            None => {},
            Some(strong) => {
                for target in targets.iter() {
                    if combat_stats.contains(*target) {
                        strength.insert(*target, strong).expect("Unable to insert strength on target");
                        if *target == *player_entity {
                            gamelog.entries.push("You feel stronger!".to_owned());
                        }
                    }
                }
            }
        }

        if consumables.contains(usable.item) {
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
            // entities.create();
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
                        WriteStorage<'a, Confusion>,
                        WriteStorage<'a, Invulnerability>,
                        WriteStorage<'a, Strength>,
                        ReadStorage<'a, Potion>,
                        WriteStorage<'a, Renderable>,
                        WriteStorage<'a, Puddle>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteExpect<'a, ParticleBuilder>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut intentthrow, map, mut backpack, mut pos, mut suffer, weight, mut agitate,   mut healing, mut teleport, mut linger, mut harm, mut explosion, mut confusion, mut invuln, mut strength, potions, mut render, mut puddle, mut rng, mut pbuilder) = data;

        for to_throw in (&mut intentthrow).join() {
            let Point {x, y} = to_throw.target;

            //  ================== PUDDLES =================
            let mut puddles: Vec<Entity> = vec![];
            let is_potion;
            {
                is_potion = potions.contains(to_throw.item);
            }
            if is_potion {
                let mut random_coords: Vec<(i32, i32)> = vec![(0, 0)];
                {
                    let all_combinations = (-1..=1).flat_map(|x| (1..=1).map(move |y| (x, y))).collect::<Vec<(i32, i32)>>();
                    for _ in 0..rng.roll_dice(1, 4)+2 {
                        let choice = rng.random_slice_index(&all_combinations);
                        if choice.is_none() { break; }
                        random_coords.push(*all_combinations.get(choice.unwrap()).unwrap());
                    }
                }

                for (dx, dy) in random_coords {
                    let puddle = entities.create();
                    pos.insert(puddle, Position { x: x+dx, y: y+dy }).expect("Unable to insert puddle coords");
                    puddles.push(puddle);
                }
            }
            // INFLICTS
            // Список эффектов: ProvidesHealing, Teleport, Lingering, Harm, Explosion

            // Heal
            if let Some(&heal) = healing.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    healing.insert(*mob, heal).expect("Unable to apply healing inflict to entity");
                }
                for pd in puddles.iter() {
                    healing.insert(*pd, heal).expect("Unable to insert puddle heal");
                }
            }

            // Teleport
            if let Some(&tp) = teleport.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    teleport.insert(*mob, tp).expect("Unable to apply teleport inflict to entity");
                }
                for pd in puddles.iter() {
                    teleport.insert(*pd, tp).expect("Unable to insert puddle tp");
                }
            }

            // Lingering
            if let Some(&lingering) = linger.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    linger.insert(*mob, lingering).expect("Unable to apply lingering inflict to entity");
                }
                for pd in puddles.iter() {
                    linger.insert(*pd, lingering).expect("Unable to insert puddle linger");
                }
            }

            // Instant damage
            if let Some(&dmg) = harm.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    harm.insert(*mob, dmg).expect("Unable to apply harm inflict to entity");
                }
                for pd in puddles.iter() {
                    harm.insert(*pd, dmg).expect("Unable to insert puddle dmg");
                }
            }

            // Explosion
            if let Some(&boom) = explosion.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    explosion.insert(*mob, boom).expect("Unable to apply explosion inflict to entity");
                }
                for pd in puddles.iter() {
                    explosion.insert(*pd, boom).expect("Unable to insert puddle explosion");
                }
            }

            // Confusion
            if let Some(&confuse) = confusion.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    confusion.insert(*mob, confuse).expect("Unable to confuse entity");
                }
                for pd in puddles.iter() {
                    confusion.insert(*pd, confuse).expect("Unable to insert puddle confuse");
                }
            }

            // Invuln
            if let Some(&invul) = invuln.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    invuln.insert(*mob, invul).expect("Unable to make entity invulnerable");
                }
                for pd in puddles.iter() {
                    invuln.insert(*pd, invul).expect("Unable to insert puddle invul");
                }
            }

            // Strength
            if let Some(&strong) = strength.get(to_throw.item) {
                for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                    strength.insert(*mob, strong).expect("Unable to make entity strong");
                }
                for pd in puddles.iter() {
                    strength.insert(*pd, strong).expect("Unable to insert puddle strength");
                }
            }

            for mob in map.tile_content[map.xy_idx(x, y)].iter().filter(|e| !potions.contains(**e) && !puddle.contains(**e)) {
                if !agitate.contains(*mob) {
                    pbuilder.request(x, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), rltk::to_cp437('!'), 200.0);
                }
                // Эй, кто в меня кинул?!
                agitate.insert(*mob, Agitated { turns: 2 }).expect("Unable to agitate enemy after throw.");
            }

            let color = render.get(to_throw.item).map_or(RGB::named(rltk::GREEN), |r| r.fg);
            
            for pd in puddles.iter() {
                render.insert(*pd, Renderable { 
                    glyph: rltk::to_cp437(' '), 
                    fg: RGB::named(rltk::BLACK), 
                    bg: color, 
                    render_order: 10 
                }).expect("Unable to insert renderable puddle");

                puddle.insert(*pd, Puddle { lifetime: 3 }).expect("Unable to insert puddle lifetime");
            }

            // damage based on weight
            if let Some(target) = map.tile_content[map.xy_idx(x, y)].iter().filter(|e| !puddle.contains(**e) ).next() {
                SufferDamage::new_damage(&mut suffer, *target, weight.get(to_throw.item).map_or(1, |w| w.0));
            } 

            if is_potion {
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