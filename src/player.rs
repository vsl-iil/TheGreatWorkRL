use rltk::{console, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use crate::{components::{CombatStats, Viewshed, WantsToMelee}, RunState};

use super::{Position, Player, TileType, Map, State};
use std::cmp::{min, max};

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut combat_stats = ecs.write_storage::<CombatStats>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
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
            _ => { return RunState::AwaitingInput }
        }
    }

    return RunState::PlayerTurn;
}