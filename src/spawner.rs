use std::collections::HashMap;

use rltk::{RandomNumberGenerator, RGB};
use specs::{prelude::*, saveload::{MarkedBuilder, SimpleMarker}};

use crate::{components::{AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item, Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializeMe, Teleport, Viewshed}, random_table::{RandomTable, SpawnEntry}, rect::Rect};

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
            power: 7
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawntable = room_table(map_depth);
    let mut spawn_points: HashMap<(i32, i32), SpawnEntry> = HashMap::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        #[allow(clippy::map_entry)]
        for _ in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));

                if !spawn_points.contains_key(&(x, y)) {
                    spawn_points.insert((x, y), spawntable.roll(&mut rng));
                    added = true;
                }

                tries += 1;
            }
        }
    }

    for spawn in spawn_points.iter() {
        match spawn.1 {
            SpawnEntry::Goblin  
                => goblin(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::Ork 
                => ork(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::MissileScroll
                => magic_missile_scroll(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::HealingPotion
                => healing_potion(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::ConfusionScroll
                => confusion_scroll(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::FireballScroll
                => fireball_scroll(ecs, spawn.0.0, spawn.0.1),
            SpawnEntry::TeleportScroll
                => teleport_scroll(ecs, spawn.0.0, spawn.0.1),
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
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

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

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Scroll of confusion".to_string()})
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn teleport_scroll(ecs: &mut World, x: i32, y: i32) {
    let safe;
    {
        let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
        safe = rng.roll_dice(1, 6) != 1;
    }

    ecs
        .create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('?'),
            fg: RGB::named(rltk::VIOLET),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name { name: "Scroll of teleportation".to_string()})
        .with(Item {})
        .with(Consumable {})
        .with(Teleport { safe })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
                // Enemies
                .add(SpawnEntry::Goblin, 12)
                .add(SpawnEntry::Ork, 1 + map_depth)
                // Items
                .add(SpawnEntry::HealingPotion, 7)
                .add(SpawnEntry::FireballScroll, 2 + map_depth)
                .add(SpawnEntry::ConfusionScroll, 2 + map_depth)
                .add(SpawnEntry::TeleportScroll, 2)
                .add(SpawnEntry::MissileScroll, 4)
}