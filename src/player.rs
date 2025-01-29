use rltk::{VirtualKeyCode, Rltk};
use specs::prelude::*;
use super::{Position, Player, TileType, Map, State};
use std::cmp::{min, max};

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let dest = map.xy_idx(pos.x + dx, pos.y + dy);
        if map.tiles[dest] != TileType::Wall {
            pos.x = min(map.width, max(0, pos.x + dx));
            pos.y = min(map.height, max(0, pos.y + dy));
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
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
            _ => {}
        }
    }
}