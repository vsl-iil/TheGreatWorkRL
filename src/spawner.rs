use std::collections::HashMap;

use rltk::{to_cp437, RandomNumberGenerator, RGB};
use specs::{prelude::*, saveload::{MarkedBuilder, SimpleMarker}};

use crate::{components::{BlocksTile, Bomber, Boss, CombatStats, Confusion, Consumable, Explosion, InstantHarm, Item, LingerType, LingeringEffect, Lobber, MacGuffin, Monster, Name, Player, Position, Potion, ProvidesHealing, Renderable, SerializeMe, Teleport, Viewshed, Weight}, map::{self, Map, TileType, MAPWIDTH}, random_table::{RandomTable, SpawnEntry}, rect::Rect};

pub const MAX_MONSTERS: i32 = 4;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: vec![], range: 8, dirty: true })
        .with(Name { name: "Rogue".to_string() })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defence: 5,
            power: 5
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn ork(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), "Ork"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), "Goblin"); }

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Viewshed { visible_tiles: vec![], range: 8, dirty: true })
        .with(Monster {})
        .with(Name { name: name.to_string() })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 10,
            hp: 10,
            defence: 1,
            power: 8
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn lobber(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('a'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Viewshed { visible_tiles: vec![], range: 8, dirty: true })
        .with(Lobber { turns: 4, targetpos: None })
        .with(Name { name: "Thrall".to_owned() })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 6,
            hp: 6,
            defence: 0,
            power: 6
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn bomber(ecs: &mut World, x: i32, y: i32) {
    let potion;
    let color;
    {
        let choice;
        let choice_color;
        {
            let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
            choice = rng.roll_dice(1, 16);
            choice_color = rng.roll_dice(1, 2);
        }

        let mut potion_build = ecs
            .create_entity()
            .with(Item {})
            .with(Potion {});

        match choice {
            1..=4 => {
                potion_build = potion_build.with(Explosion { maxdmg: 10, radius: 4});
                color = RGB::named(rltk::ORANGE);
            }
            5..=12 => {
                potion_build = potion_build.with(InstantHarm { dmg: 5 });
                color = RGB::named(rltk::DARKRED);
            }
            _ => {
                let etype = match choice_color {
                    1 => {
                        color = RGB::named(rltk::RED);
                        LingerType::Fire
                    },
                    _ => {
                        color = RGB::named(rltk::GREEN);
                        LingerType::Poison
                    },
                };
                potion_build = potion_build.with(LingeringEffect { etype, duration: 3, dmg: 3 });
            }
        }

        potion_build = potion_build.with(Renderable { 
            glyph: rltk::to_cp437('!'), 
            fg: color,
            bg: RGB::named(rltk::BLACK), 
            render_order: 2 
        });

        potion = potion_build.build();
    }

    ecs 
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: color,
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Viewshed { visible_tiles: vec![], range: 8, dirty: true })
        .with(Monster {})
        .with(Bomber { effect: potion })
        .with(Name { name: "Living potion".to_owned() })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 5,
            hp: 5,
            defence: 0,
            power: 0
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn finalboss(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('A'),
            fg: RGB::named(rltk::VIOLET),
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Viewshed { visible_tiles: vec![], range: 12, dirty: true })
        .with(Boss { state: crate::components::BossState::ClosingIn(10), targetpos: None })
        .with(Name { name: "The Cursed Alchemist".to_string() })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 70,
            hp: 70,
            defence: 2,
            power: 12
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn spawn_mcguffin(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('☼'), 
            fg: RGB::named(rltk::GOLD),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Item {})
        .with(MacGuffin {})
        .with(Weight(3))
        .with(Name { name: "The Philosopher's Stone".to_owned() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect, map: &mut Map, map_depth: i32) {
    let spawntable; 
    let mut spawn_points: HashMap<(i32, i32), SpawnEntry> = HashMap::new();
    let mut boss_coords = None;
    let mut mcguffin_coords = None;

    {
        if map_depth == map::LEVELNUM {
            spawntable = boss_table();

            for (i, tile) in map.tiles.iter_mut().enumerate() {
                if *tile == TileType::BossSpawner {
                    *tile = TileType::Floor;
                    boss_coords = Some(((i % MAPWIDTH) as i32, (i / MAPWIDTH) as i32));
                } else if *tile == TileType::MacGuffinSpawner {
                    *tile = TileType::Floor;
                    mcguffin_coords = Some(((i % MAPWIDTH) as i32, (i / MAPWIDTH) as i32));
                }
            }
        } else {
            spawntable = room_table(map_depth);
        }

        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        #[allow(clippy::map_entry)]
        for _ in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));

                if !spawn_points.contains_key(&(x, y)) && !map.blocked[map.xy_idx(x, y)] {
                    spawn_points.insert((x, y), spawntable.roll(&mut rng));
                    added = true;
                }

                tries += 1;
            }
        }
    }

    if let Some((x, y)) = boss_coords {
        finalboss(ecs, x, y);
    }
    if let Some((x, y)) = mcguffin_coords {
        spawn_mcguffin(ecs, x, y);
    }

    for spawn in spawn_points.iter() {
        let (x, y) = (spawn.0.0, spawn.0.1);
        match spawn.1 {
            SpawnEntry::Goblin  
                => goblin(ecs, x, y),
            SpawnEntry::Ork 
                => ork(ecs, x, y),
            SpawnEntry::Bomber
                => bomber(ecs, x, y),
            SpawnEntry::Lobber
                => lobber(ecs, x, y),
            // SpawnEntry::MissileScroll
            //     => magic_missile_scroll(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::HealingPotion
                => healing_potion(ecs, x, y),
            SpawnEntry::ConfusionPotion
                => confusion_potion(ecs, x, y),
            // SpawnEntry::FireballScroll
            //     => fireball_scroll(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::TeleportPotion
                => teleport_potion(ecs, x, y),
            SpawnEntry::LingeringPotion
                => lingering_potion(ecs, x, y),
            SpawnEntry::HarmingPotion
                => harm_potion(ecs, x, y),
            SpawnEntry::ExplosionPotion
                => explosion_potion(ecs, x, y),
            SpawnEntry::None
                => {},
        }
    }
}

fn healing_potion(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Health potion".to_string() })
        .with(Item {})
        .with(Potion {})
        .with(Consumable {})
        .with(Weight(1))
        .with(ProvidesHealing { heal_amount: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
/*
fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::LIGHT_YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Scroll of magic missile".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 7 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Scroll of fireball".to_string()})
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
*/
fn confusion_potion(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Potion of Confusion".to_string()})
        .with(Item {})
        .with(Potion {})
        .with(Consumable {})
        .with(Weight(1))
        // .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn teleport_potion(ecs: &mut World, x: i32, y: i32) {
    let safe;
    {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        safe = rng.roll_dice(1, 6) != 1;
    }

    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::VIOLET),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Potion of Teleportation".to_string()})
        .with(Item {})
        .with(Potion {})
        .with(Consumable {})
        .with(Weight(1))
        .with(Teleport { safe })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn lingering_potion(ecs: &mut World, x: i32, y: i32) {
    let name: &str;
    let color: (u8, u8, u8);
    let etype: LingerType;
    {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        etype = match rng.roll_dice(1, 2) {
            1 => {
                name = "Potion of Fire";
                color = rltk::RED;
                LingerType::Fire
            }, 
            _ => {
                name = "Potion of Poison";
                color = rltk::GREEN;
                LingerType::Poison
            }
        };
    }

    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(color),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: name.to_string() })
        .with(Item {})
        .with(Potion {})
        .with(Consumable {})
        .with(Weight(1))
        .with(LingeringEffect { etype, duration: 5, dmg: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn harm_potion(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::VIOLET_RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Potion of Harm".to_string() })
        .with(Item {})
        .with(Potion {})
        .with(Consumable {})
        .with(Weight(1))
        .with(InstantHarm { dmg: 7 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn explosion_potion(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Potion of Explosion".to_string() })
        .with(Item {})
        .with(Potion {})
        .with(Consumable {})
        .with(Weight(1))
        .with(Explosion { maxdmg: 10, radius: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn room_table(map_depth: i32) -> RandomTable {
    // #[cfg(debug_assertions)]
    // return RandomTable::new()
    //             // Enemies
    //             .add(SpawnEntry::Goblin, 30 + map_depth)
    //             .add(SpawnEntry::Ork, 5 + map_depth)
    //             // Items
    //             .add(SpawnEntry::HealingPotion, 7)
    //             .add(SpawnEntry::LingeringPotion, 20 + map_depth)
    //             .add(SpawnEntry::HarmingPotion, 20 + map_depth)
    //             .add(SpawnEntry::ExplosionPotion, 20 + map_depth / 2)
    //             // .add(SpawnEntry::FireballScroll, 2 + map_depth)
    //             .add(SpawnEntry::ConfusionPotion, 20 + map_depth)
    //             .add(SpawnEntry::TeleportPotion, 20 + map_depth);
    //             // .add(SpawnEntry::MissileScroll, 4)

    // #[cfg(not(debug_assertions))]
    RandomTable::new()
                // Enemies
                .add(SpawnEntry::Goblin, 12)
                .add(SpawnEntry::Ork, 1 + map_depth)
                .add(SpawnEntry::Bomber, -3 + map_depth)
                .add(SpawnEntry::Lobber, (map_depth + 4) / 2)
                // Items
                .add(SpawnEntry::HealingPotion, 7)
                .add(SpawnEntry::LingeringPotion, 2 + map_depth)
                .add(SpawnEntry::HarmingPotion, 4 + map_depth)
                .add(SpawnEntry::ExplosionPotion, 3 + map_depth / 2)
                // .add(SpawnEntry::FireballScroll, 2 + map_depth)
                .add(SpawnEntry::ConfusionPotion, 2 + map_depth)
                .add(SpawnEntry::TeleportPotion, 1 + map_depth / 2)
                // .add(SpawnEntry::MissileScroll, 4)
}

fn boss_table() -> RandomTable {
    RandomTable::new()
                .add(SpawnEntry::HealingPotion, 5)
                .add(SpawnEntry::LingeringPotion, 7)
                .add(SpawnEntry::HarmingPotion, 10)
                .add(SpawnEntry::ExplosionPotion, 3)
                .add(SpawnEntry::ConfusionPotion, 5)
                .add(SpawnEntry::TeleportPotion, 2)
                .add(SpawnEntry::Bomber, 2)
                .add(SpawnEntry::Lobber, 2)
}