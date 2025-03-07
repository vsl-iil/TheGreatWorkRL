use specs::prelude::*;

use crate::{components::{Explosion, InstantHarm, LingeringEffect, Position, ProvidesHealing, Puddle, Teleport}, map::Map};

pub struct TrapSystem {}

impl<'a> System<'a> for TrapSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (WriteStorage<'a, Puddle>,
                       Entities<'a>,
                       ReadStorage<'a, Position>,
                       WriteExpect<'a, Map>,
                    
                       WriteStorage<'a, ProvidesHealing>,
                       WriteStorage<'a, Teleport>,
                       WriteStorage<'a, LingeringEffect>,
                       WriteStorage<'a, InstantHarm>,
                       WriteStorage<'a, Explosion>
                       );

    fn run(&mut self, data: Self::SystemData) {
        let (mut puddles, entities, pos, map, mut heal, mut tp, mut linger, mut harm, mut explode) = data;

        for(ent, puddle, pos) in (&entities, &mut puddles, &pos).join() {
            puddle.lifetime -= 1;
            let Position { x, y } = *pos;
            for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                if *mob == ent { continue; }
                // INFLICTS
                // Heal
                if let Some(healing) = heal.get(ent) {
                    if !heal.contains(*mob) {
                        heal.insert(*mob, *healing).expect("Unable to insert heal inflict on entity");
                    }
                }

                // Teleport
                if let Some(teleporting) = tp.get(ent) {
                    if !tp.contains(*mob) {
                        tp.insert(*mob, *teleporting).expect("Unable to insert teleport inflict on entity");
                    }
                }

                // Lingering
                if let Some(lingering) = linger.get(ent) {
                    if !linger.contains(*mob) {
                        linger.insert(*mob, *lingering).expect("Unable to insert lingering inflict on entity");
                    }
                }

                // Instant harm
                if let Some(harming) = harm.get(ent) {
                    if !harm.contains(*mob) {
                        harm.insert(*mob, *harming).expect("Unable to insert harm inflict on entity");
                    }
                }

                // Exploding
                if let Some(exploding) = explode.get(ent) {
                    if !explode.contains(*mob) {
                        explode.insert(*mob, *exploding).expect("Unable to insert explosion inflict on entity");
                    }
                }
            }
            if puddle.lifetime == 0 {
                entities.delete(ent).expect("Unable to delete puddle");
            }
        }
    }
}