use rltk::{Point, RandomNumberGenerator, Rltk, VirtualKeyCode};
use specs::prelude::*;
use crate::{components::{CombatStats, Confusion, InBackpack, Item, Viewshed, WantsToMelee, WantsToPickupItem, Weight}, gamelog::GameLog, map::TileType, RunState};

use super::{Position, Player, Map, State};
use std::cmp::{min, max};

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut confusion = ecs.write_storage::<Confusion>();

    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    let mut dx = dx;
    let mut dy = dy;

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {

        // Player is confused
        if let Some(confused) = confusion.get_mut(entity) {
            confused.turns -= 1;
            if confused.turns < 1 {
                confusion.remove(entity);
            }

            let mut rng = ecs.write_resource::<RandomNumberGenerator>();
            dx = rng.roll_dice(1, 3) - 2;
            dy = rng.roll_dice(1, 3) - 2;
        }

        if pos.x + dx < 1 || pos.x + dx > map.width-1 || pos.y + dy < 1 || pos.y + dy > map.height-1 { return; }
        let dest = map.xy_idx(pos.x + dx, pos.y + dy);

        for potential_target in map.tile_content[dest].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_t) = target {
                wants_to_melee.insert(entity, WantsToMelee { target: *potential_target }).expect("Add target failed");
                return;
            }
        }

        if !map.blocked[dest] {
            pos.x = min(map.width, max(0, pos.x + dx));
            pos.y = min(map.height, max(0, pos.y + dy));
            viewshed.dirty = true;

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        } 
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 
                => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 
                => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 
                => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 
                => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad7 
                => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad9 
                => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad1 
                => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 
                => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Period => {
                if try_next_level(&gs.ecs) {
                    return RunState::NextLevel;
                } else {
                    return RunState::AwaitingInput;
                }
            }
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space
                // => return skip_turn(&mut gs.ecs),
                => return RunState::PlayerTurn,
            
            VirtualKeyCode::G | VirtualKeyCode::Comma 
                => get_item(&mut gs.ecs),
            VirtualKeyCode::I
                => return RunState::ShowInventory,
            VirtualKeyCode::D
                => return RunState::ShowDropItem,
            VirtualKeyCode::Escape
                => return RunState::SaveGame,
            VirtualKeyCode::Slash
                => return RunState::ShowHelp,
            VirtualKeyCode::T 
                => return RunState::ShowThrowItem,
            VirtualKeyCode::M
                => return RunState::ShowMix(None),
            #[cfg(debug_assertions)]
            VirtualKeyCode::N
                => return RunState::NextLevel,
            _ => return RunState::AwaitingInput 
        }
    }

    RunState::PlayerTurn
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, pos) in (&entities, &items, &positions).join() {
        if pos.x == player_pos.x && pos.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There's nothing to pick up.".to_string()),
        Some(item) => {
            let backpack = ecs.read_storage::<InBackpack>();
            let weight = ecs.read_storage::<Weight>();
            let mut total_weight = 0;
            for (_inpack, weight) in (&backpack, &weight).join() {
                total_weight += weight.0;
            }
            if total_weight + weight.get(item).map_or(1, |w| w.0) > 12 {
                gamelog.entries.push("You are overburdened!".to_owned());
            } else {
                let mut pickup = ecs.write_storage::<WantsToPickupItem>();
                pickup.insert(item, WantsToPickupItem { collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
            }
        }
    }
}

// no food - no heal
// fn skip_turn(ecs: &mut World) -> RunState {
    // let mut can_heal = true;
    // let player_entity = ecs.fetch::<Entity>();
    // let viewshed = ecs.read_storage::<Viewshed>();
    // let player_viewshed = viewshed.get(*player_entity);
    // let monsters = ecs.read_storage::<Monster>();
    // let map = ecs.fetch::<Map>();

    // if let Some(viewshed) = player_viewshed {
    //     for tile in viewshed.visible_tiles.iter() {
    //         let idx = map.xy_idx(tile.x, tile.y);
    //         for entity_id in map.tile_content[idx].iter() {
    //             can_heal &= !monsters.contains(*entity_id);
    //         }
    //     }
    // }

    // if can_heal {
    //     if let Some(stats) = ecs.write_storage::<CombatStats>().get_mut(*player_entity) {
    //         stats.hp = min(stats.hp + 1, stats.max_hp);
    //     }
    // }

    // RunState::PlayerTurn
// }

pub fn try_next_level(ecs: &World) -> bool {
    let Point {x: player_x, y: player_y} = *ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_x, player_y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There's no way down.".to_owned());
        false
    }
}
