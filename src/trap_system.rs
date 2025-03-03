use specs::prelude::*;

use crate::{components::{Name, Position, ProvidesHealing, Puddle, Stained, Teleport}, map::Map};

pub struct TrapSystem {}

impl<'a> System<'a> for TrapSystem {
    type SystemData = (WriteStorage<'a, Puddle>,
                    //    ReadExpect<'a, Entity>,
                       Entities<'a>,
                       ReadStorage<'a, Position>,
                       WriteExpect<'a, Map>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, Stained>,
                    
                       WriteStorage<'a, ProvidesHealing>,
                       WriteStorage<'a, Teleport>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut puddles, entities, pos, map, names, mut stained, mut heal, mut tp) = data;

        for(ent, puddle, pos) in (&entities, &mut puddles, &pos).join() {
            puddle.lifetime -= 1;
            let Position { x, y } = *pos;
            for mob in map.tile_content[map.xy_idx(x, y)].iter() {
                if *mob == ent { continue; }
                // INFLICTS
                // Heal
                if let Some(healing) = heal.get(ent) {
                    if heal.get(*mob).is_none() {
                        heal.insert(*mob, *healing).expect("Unable to insert heal inflict on entity");
                    }
                }
                // Teleport
                if let Some(teleporting) = tp.get(ent) {
                    if tp.get(*mob).is_none() {
                        tp.insert(*mob, *teleporting).expect("Unable to insert teleport inflict on entity");
                    }
                }

                if stained.get(*mob).is_none() {
                    stained.insert(*mob, Stained {}).expect("Unable to puddle stain entity");
                }
            }
            if puddle.lifetime == 0 {
                entities.delete(ent).expect("Unable to delete puddle");
            }
        }
    }
}