use rltk::{to_cp437, Algorithm2D, BaseMap, FontCharType, Point, RandomNumberGenerator, Rltk, RGB};
use serde::{Deserialize, Serialize};
use specs::{Entity, World};

use super::rect::*;
use std::cmp::{min, max};

pub const MAPWIDTH:  usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;
#[cfg(not(debug_assertions))]
pub const LEVELNUM: i32 = 8;
#[cfg(debug_assertions)]
pub const LEVELNUM: i32 = 8;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    BossSpawner,
    FinalDoor,
    MacGuffinSpawner
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub puddles: Vec<i32>,
    pub depth: i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * MAPWIDTH) + x as usize
    }

    pub fn new_map_rooms_and_corridors(new_depth: i32) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: vec![],
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![vec![]; MAPCOUNT],
            puddles: vec![0; MAPCOUNT],
            depth: new_depth
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 7;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        if new_depth == LEVELNUM {
            map.final_level(&mut rng);

            return map;
        }

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, MAPWIDTH as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, MAPHEIGHT as i32 - h - 1) - 1;

            let new_room = Rect::new(x, y, w, h);

            let mut room_ok = true;
            for other_room in map.rooms.iter() {
                room_ok &= !other_room.intersect(&new_room);

            }

            if room_ok {
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();

                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.apply_room_to_map(&new_room);
                map.rooms.push(new_room);
            }
        }

        let (stair_x, stair_y) = map.rooms.iter()
                                          .last()
                                          .expect("No rooms were generated?")
                                          .center();

        let idx = map.xy_idx(stair_x, stair_y);
        map.tiles[idx] = TileType::DownStairs;

        map
    }

    pub fn final_level(&mut self, rng: &mut RandomNumberGenerator) {
        let x = rng.roll_dice(1, (MAPWIDTH-33) as i32);
        let y = rng.roll_dice(1, (MAPHEIGHT-16) as i32);

        self.apply_final_lab(x, y);

        self.rooms.push(Rect { x1: x, x2: x+15, y1: y, y2: y+15 });
    }

    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1+1..=room.y2 {
            for x in room.x1+1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_final_lab(&mut self, x: i32, y: i32) {
        let room_bytes = include_bytes!("../room.bin");
        for (i, b) in room_bytes.iter().enumerate() {
            let idx = self.xy_idx(x + (i % 32) as i32, y + (i / 32) as i32);
            self.tiles[idx] = match b {
                1 => TileType::Wall,
                2 => TileType::BossSpawner,
                3 => TileType::FinalDoor,
                4 => TileType::MacGuffinSpawner,
                _ => TileType::Floor,
            };
        }
    }

    pub fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > (MAPWIDTH-1) as i32 
        || y < 1 || y > (MAPHEIGHT-1) as i32 {
            return false;
        }
        let idx = self.xy_idx(x, y);

        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = *tile == TileType::Wall || *tile == TileType::FinalDoor;
        }
    }

    pub fn clear_content_index(&mut self) {
        for tile in self.tile_content.iter_mut() {
            tile.clear();
        }
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk, map_depth: i32) {
    let map = ecs.fetch::<Map>();

    let mut x = 0;
    let mut y = 0;

    let tint = (
        f32::max(0.0, 0.1 - f32::powf(0.25 * map_depth as f32 - 1.25, 2.0)),
        f32::max(0.0, 0.1 - f32::powf(0.25 * map_depth as f32 - 0.45, 2.0)),
        f32::max(0.0, 0.1 - f32::powf(0.25 * map_depth as f32 - 0.85, 2.0)),
    );

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph: FontCharType;
            let mut fg: RGB;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.3, 0.3, 0.3);
                }
                TileType::Wall => {
                    // glyph = rltk::to_cp437('#');
                    glyph = wall_glyph(&*map, x, y);
                    fg = RGB::from_f32(0.8+tint.0, 0.8+tint.1, 0.8+tint.2);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    if map_depth == LEVELNUM-1 {
                        fg = RGB::named(rltk::RED);
                    } else {
                        fg = RGB::from_f32(0.8, 0.8, 0.95);
                    }
                },
                TileType::BossSpawner => {
                    glyph = rltk::to_cp437('A');
                    fg = RGB::from_f32(1.0, 1.0, 1.0);
                },
                TileType::FinalDoor => {
                    glyph = rltk::to_cp437('|');
                    fg = RGB::named(rltk::BROWN4);
                },
                TileType::MacGuffinSpawner => {
                    glyph = rltk::to_cp437('☼');
                    fg = RGB::named(rltk::GOLD);
                }
            }
            if !map.visible_tiles[idx] { 
                let mut darkest = f32::min(fg.b, f32::min(fg.g, fg.r));
                if darkest > 0.3 {
                    darkest -= 0.3;
                }
                fg.b = darkest;
                fg.g = darkest;
                fg.r = darkest;
            }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 { return to_cp437('#'); }
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y-1) { mask += 1; }
    if is_revealed_and_wall(map, x, y+1) { mask += 2; }
    if is_revealed_and_wall(map, x-1, y) { mask += 4; }
    if is_revealed_and_wall(map, x+1, y) { mask += 8; }

    match mask {
        0 => to_cp437('#'),
        1 => { 186 } // Wall only to the north
        2 => { 186 } // Wall only to the south
        3 => { 186 } // Wall to the north and south
        4 => { 205 } // Wall only to the west
        5 => { 188 } // Wall to the north and west
        6 => { 187 } // Wall to the south and west
        7 => { 185 } // Wall to the north, south and west
        8 => { 205 } // Wall only to the east
        9 => { 200 } // Wall to the north and east
        10 => { 201 } // Wall to the south and east
        11 => { 204 } // Wall to the north, south and east
        12 => { 205 } // Wall to the east and west
        13 => { 202 } // Wall to the east, west, and south
        14 => { 203 } // Wall to the east, west, and north
        15 => { 206 }  // ╬ Wall on all sides
        _ => { 35 } // We missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    map.revealed_tiles[map.xy_idx(x, y)] && map.tiles[map.xy_idx(x, y)] == TileType::Wall
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(MAPWIDTH, MAPHEIGHT)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall || self.tiles[idx] == TileType::FinalDoor
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % MAPWIDTH as i32;
        let y = idx as i32 / MAPWIDTH as i32;
        let w = MAPWIDTH;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0));}      // W
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0));}      // E
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0));}      // N
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0));}      // S
        if self.is_exit_valid(x-1, y+1) { exits.push((idx-1+w, 1.0));} // SW
        if self.is_exit_valid(x+1, y-1) { exits.push((idx+1-w, 1.0));} // NE
        if self.is_exit_valid(x-1, y-1) { exits.push((idx-1-w, 1.0));} // NW
        if self.is_exit_valid(x+1, y+1) { exits.push((idx+1+w, 1.0));} // SE

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = MAPWIDTH;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);

        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}