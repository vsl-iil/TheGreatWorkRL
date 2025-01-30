use monster_ai_system::MonsterAI;
use rltk::{GameState, Point, Rltk, RGB};
use specs::prelude::*;

mod components;
use components::*;
mod map;
use map::*;
mod player;
use player::*;
mod rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;

pub struct State {
    pub ecs: World,
    pub runstate: RunState
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();

        //let map = self.ecs.fetch::<Vec<TileType>>();
        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    Paused,
    Running
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    // context.with_post_scanlines(false);

    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();

    let mut rng = rltk::RandomNumberGenerator::new();

    let map = Map::new_map_rooms_and_corridors(80, 50);
    let (player_x, player_y) = map.rooms[0].center();
    
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        
        let glyph: rltk::FontCharType;
        let name: String;

        let roll = rng.roll_dice(1, 2);

        match roll {
            1 => { glyph = rltk::to_cp437('g'); name = "Goblin".to_string(); }
            _ => { glyph = rltk::to_cp437('o'); name = "Ork".to_string(); }
        }
        
        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed { visible_tiles: vec![], range: 8, dirty: true })
            .with(Monster {})
            .with(Name { name: format!("{name} #{i}") })
            .build();
    }
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(map);

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: vec![], range: 8, dirty: true })
        // .with(Name { name: "Rogue".to_string() })
        .build();

    rltk::main_loop(context, gs)

}
