use damage_system::DamageSystem;
use gui::draw_ui;
use inventory_system::{InventorySystem, ItemDropSystem, ItemUseSystem};
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use monster_ai_system::MonsterAI;
use rltk::{GameState, Point, Rltk};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

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
mod saveload_system;

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();

        match newrunstate {
            RunState::MainMenu {..} => {},
            _ => {
                    draw_map(&self.ecs, ctx);
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
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel
                        => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected
                        => {
                            let item = result.1.unwrap();
                            let is_ranged = self.ecs.read_storage::<Ranged>();
                            if let Some(item_ranged) = is_ranged.get(item) {
                                newrunstate = RunState::ShowTargeting { range: item_ranged.range, item }
                            } else {
                                let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                                intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: None }).expect("Unable to insert want to use");
                                newrunstate = RunState::PlayerTurn;
                            }
                        }
                }
            }
            RunState::ShowDropItem => {
                match gui::drop_menu(self, ctx) {
                    (gui::ItemMenuResult::Selected, item) 
                        => {
                            let item = item.unwrap();
                            let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item }).expect("Unable to insert drop intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    (gui::ItemMenuResult::Cancel, _)
                        => newrunstate = RunState::AwaitingInput,
                    (gui::ItemMenuResult::NoResponse, _) => {}
                }
            }
            RunState::ShowTargeting { range, item }
                => {
                    let mut radius = 0;
                    {
                        let aoe = self.ecs.read_storage::<AreaOfEffect>();
                        if let Some(r) = aoe.get(item) {
                            radius = r.radius + 1;
                        }
                    }
                    let target = gui::ranged_target(self, ctx, range, radius);
                    match target.0 {
                        gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                        gui::ItemMenuResult::NoResponse => {},
                        gui::ItemMenuResult::Selected => {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: target.1 }).expect("Unable to insert use intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                },
            RunState::MainMenu {..}
                => {
                    let result = gui::main_menu(self, ctx);
                    match result {
                        gui::MainMenuResult::NoSelection { selected }
                            => newrunstate = RunState::MainMenu { menu_selection: selected },
                        gui::MainMenuResult::Selected { selected }
                            => {
                                match selected {
                                    gui::MainMenuSelection::NewGame
                                        => newrunstate = RunState::PreRun,
                                    gui::MainMenuSelection::LoadGame
                                        => {
                                            saveload_system::load_game(&mut self.ecs);
                                            newrunstate = RunState::AwaitingInput;
                                            saveload_system::delete_save();
                                        },
                                    gui::MainMenuSelection::Quit
                                        => ::std::process::exit(0),
                                }
                            }
                    }
            },
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::LoadGame }
            },
        }

        {
            let mut runwriter = self.ecs.fetch_mut::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::clean_up_dead(&mut self.ecs);

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
        let mut itemuse = ItemUseSystem {};
        itemuse.run_now(&self.ecs);
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
    ShowDropItem,
    ShowTargeting{ range: i32, item: Entity },
    MainMenu{ menu_selection: gui::MainMenuSelection },
    SaveGame
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
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Agitated>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

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
    gs.ecs.insert(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });

    rltk::main_loop(context, gs)

}
