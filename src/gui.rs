use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

use crate::{components::{CombatStats, InBackpack, Name, Player, Position, Potion, Viewshed, Weight}, gamelog::GameLog, map::{Map, MAPWIDTH}, RunState, State};

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection }
}

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, MAPWIDTH-1, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);
        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    let log = ecs.fetch::<GameLog>();

    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 { ctx.print(2, y, s); }
        y += 1;
    }

    let map = ecs.fetch::<Map>();
    let depth = format!("Floor {}", map.depth);
    ctx.print_color(2, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &depth);

    draw_tooltip(ecs, ctx);
}

fn draw_tooltip(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }

    let mut tooltip: Vec<String> = vec![];
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        if mouse_pos.0 > map.width / 2 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(rltk::WHITE), RGB::named(rltk::DARKBLUE), s);
                let padding = (width - s.len() as i32) - 1;
                for _i in 0..padding {
                    ctx.print_color(arrow_pos.x - 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::DARKBLUE), " ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::DARKBLUE), "->".to_string());
        }
        else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::DARKBLUE), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(rltk::WHITE), RGB::named(rltk::DARKBLUE), " ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::DARKBLUE), "<-".to_string());
        }

    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let weight = gs.ecs.read_storage::<Weight>();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count() as i32;

    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y-2, 31, count+3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Inventory");
    ctx.print_color(40, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Weight");
    ctx.print_color(18, y+count + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Press ESC to close");

    let mut usable: Vec<Entity> = vec![];
    let mut items = (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity).collect::<Vec<_>>();
    items.sort_by(|a, b| a.2.name.cmp(&b.2.name));
    for (j, (entity, _pack, name)) in items.into_iter().enumerate() {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, truncate_str(name.name.to_string()));
        ctx.print(45, y, weight.get(entity).map_or(1, |w| w.0));
        usable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            rltk::VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    return (ItemMenuResult::Selected, Some(usable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        }
    }
}

pub fn drop_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let weight = gs.ecs.read_storage::<Weight>();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count() as i32;

    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y-2, 31, count+3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Drop which item?");
    ctx.print_color(40, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Weight");
    ctx.print_color(18, y+count + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Press ESC to close");

    let mut droppable: Vec<Entity> = vec![];
    let mut items = (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity).collect::<Vec<_>>();
    items.sort_by(|a, b| a.2.name.cmp(&b.2.name));
    for (j, (entity, _pack, name)) in items.into_iter().enumerate() {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, truncate_str(name.name.to_string()));
        ctx.print(45, y, weight.get(entity).map_or(1, |w| w.0));
        droppable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            rltk::VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    return (ItemMenuResult::Selected, Some(droppable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None) 
            }
        }
    }
}

pub fn ranged_target(gs: &mut State, ctx: &mut Rltk, range: i32, radius: i32) -> (ItemMenuResult, Option<rltk::Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();
    let mut gamelog = gs.ecs.fetch_mut::<GameLog>();

    let msg = "Select a target...".to_string();
    if gamelog.entries.iter().last().is_some_and(|m| m != &msg) {
        gamelog.entries.push(msg);
    }

    // Подсветим доступные видимые клетки
    let mut available_cells = vec![];
    let visible = viewsheds.get(*player_entity);

    if let Some(visible) = visible {
        // Мы что-то видим!
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(rltk::BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        // Мы слепы!
        return (ItemMenuResult::Cancel, None);
    }

    // Отрисовка курсора
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() { 
        // Мышь указывает на доступную клетку?
        valid_target |= idx.x == mouse_pos.0 && idx.y == mouse_pos.1;
    }

    if valid_target && radius > 0 {
        for idx in available_cells.iter() {
        let distance = rltk::DistanceAlg::Pythagoras.distance2d(rltk::Point { x: mouse_pos.0, y: mouse_pos.1 }, **idx);
            if distance <= radius as f32 && distance > 0.0 {
                ctx.set_bg(idx.x, idx.y, RGB::named(rltk::ORANGERED));
            }
        }
    }

    if ctx.key.is_some_and(|k| k == VirtualKeyCode::Escape) { 
        return (ItemMenuResult::Cancel, None); 
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point { x: mouse_pos.0, y: mouse_pos.1 }));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();
    let game_exists = crate::saveload_system::does_save_exist();

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "The Great Work");

    if let RunState::MainMenu { menu_selection: mut selection } = *runstate {
        if !game_exists {
            selection = MainMenuSelection::NewGame;
        }
        #[cfg(target_arch = "wasm32")]
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::BLACK), RGB::named(rltk::YELLOW), " Continue ");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), " Continue ");
        }

        #[cfg(not(target_arch = "wasm32"))]
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::BLACK), RGB::named(rltk::YELLOW), " New Game ");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), " New Game ");
        }

        if game_exists {
            if selection == MainMenuSelection::LoadGame {
                ctx.print_color_centered(26, RGB::named(rltk::BLACK), RGB::named(rltk::YELLOW), " Load Game ");
            } else {
                ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), " Load Game ");
            }
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(28, RGB::named(rltk::BLACK), RGB::named(rltk::YELLOW), " Quit ");
        } else {
            ctx.print_color_centered(28, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), " Quit ");
        }


        match ctx.key {
            None => return MainMenuResult::NoSelection { selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame  => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit     => newselection = MainMenuSelection::LoadGame,
                        }
                        if newselection == MainMenuSelection::LoadGame && !game_exists {
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection { selected: newselection }
                    },
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame  => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit     => newselection = MainMenuSelection::NewGame,
                        }
                        if newselection == MainMenuSelection::LoadGame && !game_exists {
                            newselection = MainMenuSelection::Quit;
                        }
                        return MainMenuResult::NoSelection { selected: newselection }
                    },
                    VirtualKeyCode::Return => return MainMenuResult::Selected { selected: selection },
                    _ => return MainMenuResult::NoSelection { selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

pub fn keybinds_menu(ctx: &mut Rltk) -> ItemMenuResult {
    macro_rules! formstr {
        ($key:literal, $desc:literal) => {
            format!("{:^5} - {}", $key, $desc)
        };
    }

    ctx.draw_box(15, 5, 50, 22, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color_centered(5, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Keybinds");
    ctx.print_color(18, 27, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Press ESC to close");

    #[cfg(not(target_arch = "wasm32"))]
    let strings: Vec<String> = vec![
        formstr!("←↑↓→", "move"),
        formstr!(".", "descend to next level"),
        formstr!("g | ,", "pick up an item"),
        formstr!("space", "wait a turn"),
        formstr!("i", "open inventory"),
        formstr!("d", "drop an item"),
        formstr!("t", "throw an item"),
        formstr!("m", "mix potions"),
        formstr!("esc", "pause"),
        formstr!("/", "help"),
    ];

    #[cfg(target_arch = "wasm32")]
    let strings: Vec<String> = vec![
        formstr!("←↑↓→", "move"),
        formstr!(">", "descend to next level"),
        formstr!("g | ,", "pick up an item"),
        formstr!("space", "wait a turn"),
        formstr!("i", "open inventory"),
        formstr!("d", "drop an item"),
        formstr!("t", "throw an item"),
        formstr!("m", "mix potions"),
        formstr!("esc", "pause"),
        formstr!("?", "help"),
    ];

    for (i, s) in strings.iter().enumerate() {
        ctx.print_color(16, 7+2*i, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), s);
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(rltk::VirtualKeyCode::Escape)
             => ItemMenuResult::Cancel,
        Some(_) => ItemMenuResult::NoResponse
    }
}

pub fn throw_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let weight = gs.ecs.read_storage::<Weight>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count() as i32;

    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y-2, 31, count+3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Throw which item?");
    ctx.print_color(40, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Weight");
    ctx.print_color(18, y+count + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Press ESC to close");

    let mut throwable: Vec<Entity> = vec![];
    let mut items = (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity).collect::<Vec<_>>();
    items.sort_by(|a, b| a.2.name.cmp(&b.2.name));
    for (j, (entity, _pack, name)) in items.into_iter().enumerate() {
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, truncate_str(name.name.to_string()));
        ctx.print(45, y, weight.get(entity).map_or(1, |w| w.0));
        throwable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            rltk::VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    return (ItemMenuResult::Selected, Some(throwable[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None) 
            }
        }
    }
}

pub fn mix_potions(gs: &mut State, ctx: &mut Rltk, selected: Option<Entity>) -> (ItemMenuResult, Option<Entity>, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let potions = gs.ecs.read_storage::<Potion>();
    let entities = gs.ecs.entities();
    let weight = gs.ecs.read_storage::<Weight>();

    let inventory = (&backpack, &names, &potions).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count() as i32;

    let mut y = 25 - (count / 2);
    ctx.draw_box(15, y-2, 31, count+3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    if selected.is_none() {
        ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Choose first ingredient...");
    } else {
        ctx.print_color(18, y-2, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Choose second ingredient...");
    }
    ctx.print_color(18, y+count + 1, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Press ESC to close");

    let mut mixable: Vec<Entity> = vec![];
    let mut items = (&entities, &backpack, &names, &potions).join().filter(|item| item.1.owner == *player_entity).collect::<Vec<_>>();
    items.sort_by(|a, b| a.2.name.cmp(&b.2.name));
    for (j, (entity, _pack, name, _potion)) in items.into_iter().enumerate() {

        let (fg, bg, glyph) = if selected.is_some_and(|sel| sel == entity) {
            (RGB::named(rltk::BLACK), RGB::named(rltk::YELLOW), 65+j as rltk::FontCharType)
        } else {
            (RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), 97+j as rltk::FontCharType)
        };
        ctx.set(17, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437('('));
        ctx.set(18, y, fg, bg, glyph);
        ctx.set(19, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(')'));

        ctx.print(21, y, truncate_str(name.name.to_string()));
        ctx.print(45, y, weight.get(entity).map_or(1, |w| w.0));
        mixable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, selected, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                (ItemMenuResult::Cancel, None, None)
            },
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    let selection = mixable[selection as usize];
                    if selected.is_some() {
                        if selected.unwrap() == selection {
                            return (ItemMenuResult::Selected, None, None);
                        }
                        return (ItemMenuResult::Selected, selected, Some(selection))
                    } else {
                        return (ItemMenuResult::Selected, Some(selection), None)
                    }
                }
                (ItemMenuResult::NoResponse, selected, None)
            }
        }
    }
}

pub fn gameover(ctx: &mut Rltk) -> ItemMenuResult {
    ctx.draw_box(35, 20, 10, 3, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color_centered(22, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "You died!");

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(VirtualKeyCode::Escape | VirtualKeyCode::Return) 
             => ItemMenuResult::Cancel,
        Some(_) => ItemMenuResult::NoResponse
    }
}

pub fn winscreen(ctx: &mut Rltk) -> ItemMenuResult {
    ctx.draw_box(15, 20, 50, 11, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    ctx.print_color_centered(22, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "You found the Philosopher's stone!");
    ctx.print_color_centered(24, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "You hear the passage to the surface");
    ctx.print_color_centered(25, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "closing behind you. You begin to wonder");
    ctx.print_color_centered(26, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "if the fate of the Cursed Alchemist");
    ctx.print_color_centered(27, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "awaits you as well...");
    ctx.print_color_centered(29, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Thank you for playing!");
    ctx.print_color_centered(31, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "visilii for 7DRL 2025");

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(VirtualKeyCode::Escape | VirtualKeyCode::Return) 
             => ItemMenuResult::Cancel,
        Some(_) => ItemMenuResult::NoResponse
    }
}

fn truncate_str(name: String) -> String {
    match name.char_indices().nth(20) {
        None => name,
        Some((idx, _)) => {
            let mut n = name[..idx].to_string();
            n.push_str("...");

            n
        }
    }
}
