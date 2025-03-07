use damage_system::DamageSystem;
use gamelog::GameLog;
use gui::draw_ui;
use inventory_system::{InventorySystem, ItemDropSystem, ItemThrowSystem, ItemUseSystem};
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use monster_ai_system::{BossAI, MonsterAI};
use rltk::{GameState, Point, RandomNumberGenerator, Rltk};
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
use staineffect_system::StainEffect;
use trap_system::TrapSystem;
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
mod random_table;
mod staineffect_system;
mod trap_system;


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
            RunState::ShowHelp => {
                let result = gui::keybinds_menu(ctx);
                match result {
                    gui::ItemMenuResult::Cancel
                        => newrunstate = RunState::AwaitingInput,
                    _   => {}
                }
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
                                newrunstate = RunState::ShowTargeting { range: item_ranged.range, item, targettype: TargetType::Use }
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
            RunState::ShowTargeting { range, item, targettype }
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
                            if targettype == TargetType::Use {
                                let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                                intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: target.1 }).expect("Unable to insert use intent");
                            } else {
                                {
                                    let mut intent = self.ecs.write_storage::<WantsToThrowItem>();
                                    intent.insert(*self.ecs.fetch::<Entity>(), WantsToThrowItem { item, target: target.1.unwrap() }).expect("Unable to insert throw intent");
                                }
                            }
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
                                        => {
                                            // TODO save before quit + add "continue" in addition to "new game" during playthrough
                                            ::std::process::exit(0);
                                        }
                                }
                            }
                    }
            },
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::LoadGame }
            },
            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            },
            RunState::ShowThrowItem => {
                let result = gui::throw_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Selected => {
                        let item = result.1.unwrap();
                        let ws = self.ecs.read_storage::<Weight>();
                        let weight = ws.get(item).map_or(0, |w| w.0);
                        newrunstate = RunState::ShowTargeting { range: 6-weight, item, targettype: TargetType::Throw };
                    },
                    gui::ItemMenuResult::Cancel    
                        => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse
                        => {},
                }
            }
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
        let mut throw = ItemThrowSystem {};
        throw.run_now(&self.ecs);
        let mut boss = BossAI {};
        boss.run_now(&self.ecs);

        let runstate;
        {
            let runstwriter = self.ecs.fetch::<RunState>();
            runstate = *runstwriter;
        }
        if runstate == RunState::PlayerTurn {
            let mut trap = TrapSystem {};
            trap.run_now(&self.ecs);
            let mut stain = StainEffect {};
            stain.run_now(&self.ecs);
        }

        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player_entity = self.ecs.fetch::<Entity>();
        let player = self.ecs.read_storage::<Player>();
        let inbackpack = self.ecs.read_storage::<InBackpack>();

        let mut to_delete: Vec<Entity> = vec![];
        for entity in entities.join() {
            let mut should_delete = true;

            if let Some(_player) = player.get(entity) {
                should_delete = false;
            }

            if let Some(backpack_item) = inbackpack.get(entity) {
                if backpack_item.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        let to_delete = self.entities_to_remove_on_level_change();
        self.ecs.delete_entities(&to_delete).expect("Unable to delete entities");

        let new_depth;
        let mut worldmap;
        {
            let mut worldmap_res = self.ecs.write_resource::<Map>();
            new_depth = worldmap_res.depth + 1;
            *worldmap_res = Map::new_map_rooms_and_corridors(new_depth);
            worldmap = worldmap_res.clone();
        }

        for room in worldmap.rooms.clone().iter() {
            spawner::spawn_room(&mut self.ecs, room, &mut worldmap, new_depth);
        }

        {   // Костыли мои костыли
            let mut worldmap_res = self.ecs.write_resource::<Map>();
            *worldmap_res = worldmap.clone();
        }

        let (player_x, player_y) = worldmap.rooms[0].center();
        let mut player_pos = self.ecs.write_resource::<Point>();
        *player_pos = Point::new(player_x, player_y);
        let mut pos_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        
        if let Some(player_pos_comp) = pos_components.get_mut(*player_entity) {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        let mut viewsheds = self.ecs.write_storage::<Viewshed>();

        if let Some(player_vs) = viewsheds.get_mut(*player_entity) {
            player_vs.dirty = true;
        }

        let mut gamelog = self.ecs.write_resource::<GameLog>();
        gamelog.entries.push("You descend to a next level, and take a moment to heal.".to_owned());

        if new_depth == LEVELNUM-1 {
            gamelog.entries.push("You feel foul presence the level below.".to_owned());
        }

        let mut stats = self.ecs.write_storage::<CombatStats>();

        if let Some(player_stats) = stats.get_mut(*player_entity) {
            player_stats.hp = i32::max(player_stats.hp, player_stats.max_hp / 2);
        }

    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowHelp,
    ShowInventory,
    ShowDropItem,
    ShowTargeting{ range: i32, item: Entity, targettype: TargetType },
    MainMenu{ menu_selection: gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    ShowThrowItem
}

#[derive(PartialEq, Clone, Copy)]
pub enum TargetType {
    Use,
    Throw
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("The Great Work")
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
    gs.ecs.register::<Teleport>();
    gs.ecs.register::<Weight>();
    gs.ecs.register::<Puddle>();
    gs.ecs.register::<Potion>();
    gs.ecs.register::<WantsToThrowItem>();
    gs.ecs.register::<LingeringEffect>();
    gs.ecs.register::<InstantHarm>();
    gs.ecs.register::<Explosion>();
    gs.ecs.register::<Bomber>();
    gs.ecs.register::<Boss>();
    gs.ecs.register::<MacGuffin>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let mut map = Map::new_map_rooms_and_corridors(1);
    let (player_x, player_y) = map.rooms[0].center();
    
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    gs.ecs.insert(Point::new(player_x, player_y));
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
    gs.ecs.insert(player_entity);

    for room in map.rooms.clone().iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room, &mut map, 1);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(gamelog::GameLog { entries: vec!["Welcome to the dungeon of doom!".to_string()] });
    gs.ecs.insert(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });

    rltk::main_loop(context, gs)

}
