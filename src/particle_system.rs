use rltk::{Rltk, RGB};
use specs::prelude::*;

use crate::components::{ParticleLifetime, Position, Renderable};

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime: f32
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder { requests: Vec::new() }
    }

    pub fn request(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, glyph: rltk::FontCharType, lifetime: f32) {
        self.requests.push(ParticleRequest { x, y, fg, bg, glyph, lifetime })
    }
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Renderable>,
                        WriteStorage<'a, ParticleLifetime>,
                        WriteExpect<'a, ParticleBuilder>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renders, mut lifetimes, mut pbuilder) = data;

        for particle in pbuilder.requests.iter() {
            let ParticleRequest { x, y, fg, bg, glyph, lifetime } = *particle;
            let p = entities.create();
            positions.insert(p, Position { x, y }).expect("Unable to insert particle position");
            renders.insert(p, Renderable { glyph, fg, bg, render_order: 0 }).expect("Unable to insert particle render");
            lifetimes.insert(p, ParticleLifetime { lifetime_ms: lifetime }).expect("Unable to insert particle lifetime");
        }

        pbuilder.requests.clear();
    }
}

pub fn cull_dead_particles(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles: Vec<Entity> = Vec::new();
    {
        let mut lifetimes = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();

        for (ent, particle) in (&entities, &mut lifetimes).join() {
            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(ent);
            }
        }
    }

    for particle in dead_particles {
        ecs.delete_entity(particle).expect("Particle won't die");
    }
}