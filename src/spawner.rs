use rltk::{RandomNumberGenerator, RGB};
use specs::{prelude::*, saveload::{MarkedBuilder, SimpleMarker}};

use crate::{components::{AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item, Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializeMe, Teleport, Viewshed}, rect::Rect};

pub const MAX_MONSTERS: i32 = 4;
pub const MAX_ITEMS: i32 = 2;

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

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { ork(ecs, x, y) }
        _ => { goblin(ecs, x, y) }
    }
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

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<(i32, i32)> = vec![];
    let mut item_spawn_points: Vec<(i32, i32)> = vec![];
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS+1)-1;
        let num_items = rng.roll_dice(1, MAX_ITEMS+1)-1;

        for _ in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));

                if !monster_spawn_points.contains(&(x, y)) {
                    monster_spawn_points.push((x, y));
                    added = true;
                }
            }
        }

        for _ in 0..num_items {
            let mut added = false;
            while !added {
                let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
                let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));

                if !item_spawn_points.contains(&(x, y)) {
                    item_spawn_points.push((x, y));
                    added = true;
                }
            }
        }
    }

    for coords in monster_spawn_points.iter() {
        random_monster(ecs, coords.0, coords.1);
    }
    for coords in item_spawn_points.iter() {
        random_item(ecs, coords.0, coords.1);
    }
}

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 5);
    }

    match roll {
        1 => healing_potion(ecs, x, y),
        2 => fireball_scroll(ecs, x, y),
        3 => confusion_scroll(ecs, x, y),
        4 => teleport_scroll(ecs, x, y),
        _ => magic_missile_scroll(ecs, x, y),
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