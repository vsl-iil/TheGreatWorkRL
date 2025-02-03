use damage_system::DamageSystem;
use gui::draw_ui;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
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
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;
mod gui;
mod gamelog;

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::PreRun;
            }
        }

        {
            let mut runwriter = self.ecs.fetch_mut::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::clean_up_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }

        draw_ui(&self.ecs, ctx);
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    context.with_post_scanlines(false);

    let mut gs = State {
        ecs: World::new(),
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let mut rng = rltk::RandomNumberGenerator::new();

    let map = Map::new_map_rooms_and_corridors();
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
            .with(Name { name: format!("{} #{}", name, i+1) })
            .with(BlocksTile {})
            .with(CombatStats {
                max_hp: 10,
                hp: 10,
                defence: 1,
                power: 7
            })
            .build();
    }
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(map);
    gs.ecs.insert(gamelog::GameLog { entries: vec!["Welcome to the dungeon of doom!".to_string()] });

    let player_entity = gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
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
        .build();

    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);

    rltk::main_loop(context, gs)

}
