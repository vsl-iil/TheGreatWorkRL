use rltk::{GameState, Rltk, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
use specs_derive::Component;
use rand;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
struct RandomMover {}

#[derive(Component, Debug)]
struct Player {}

struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();

        self.run_systems();

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut rw = RandomWalker{};
        rw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

struct RandomWalker {}

impl<'a> System<'a> for RandomWalker {
    type SystemData = (ReadStorage<'a, RandomMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (shall_move, mut pos): Self::SystemData) {
        for (_shall_move, pos) in (&shall_move, &mut pos).join() {
            let dx = rand::random::<i32>() % 3 - 1;
            let dy = rand::random::<i32>() % 3 - 1;

            pos.x = (pos.x + dx).rem_euclid(80);
            pos.y = (pos.y + dy).rem_euclid(50);

        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new()
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<RandomMover>();
    gs.ecs.register::<Player>();

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 4, y: i * 2 })
            .with(Renderable {
                glyph: rltk::to_cp437('g'),
                fg: RGB::named(rltk::GREEN),
                bg: RGB::named(rltk::BLACK),
            })
            .with(RandomMover{})
            .build();
    }

    rltk::main_loop(context, gs)

}
