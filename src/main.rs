use damage_system::DamageSystem;
use gui::draw_ui;
use inventory_system::{InventorySystem, ItemDropSystem, PotionUseSystem};
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use monster_ai_system::MonsterAI;
use rltk::{GameState, Point, Rltk};
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
mod inventory_system;
mod gui;
mod gamelog;
mod spawner;

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();

        draw_map(&self.ecs, ctx);

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::PreRun;
            }
            RunState::ShowInventory => {
                match gui::show_inventory(self, ctx) {
                    gui::ItemMenuResult::Selected(item)
                        => {
                            let mut intent = self.ecs.write_storage::<WantsToDrinkPotion>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToDrinkPotion { potion: item }).expect("Unable to insert want to drink");
                            newrunstate = RunState::PlayerTurn;
                        }
                    gui::ItemMenuResult::Cancel 
                        => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                }
            }
            RunState::ShowDropItem => {
                match gui::drop_menu(self, ctx) {
                    gui::ItemMenuResult::Selected(item) 
                        => {
                            let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item }).expect("Unable to insert drop intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    gui::ItemMenuResult::Cancel
                        => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                }
            }
        }

        {
            let mut runwriter = self.ecs.fetch_mut::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::clean_up_dead(&mut self.ecs);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        // let player_pos = self.ecs.fetch::<Position>();

        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        for (pos, render) in data {
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
        let mut inventory = InventorySystem {};
        inventory.run_now(&self.ecs);
        let mut potion = PotionUseSystem {};
        potion.run_now(&self.ecs);
        let mut drop = ItemDropSystem {};
        drop.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem
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
    gs.ecs.register::<Item>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToDrinkPotion>();
    gs.ecs.register::<WantsToDropItem>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    gs.ecs.insert(Point::new(player_x, player_y));
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
    gs.ecs.insert(player_entity);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(gamelog::GameLog { entries: vec!["Welcome to the dungeon of doom!".to_string()] });
    gs.ecs.insert(RunState::PreRun);

    rltk::main_loop(context, gs)

}
