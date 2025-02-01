use specs::prelude::*;

use crate::{components::{BlocksTile, Position}, map::Map};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, pos, blockers, ent) = data;

        map.populate_blocked();
        map.clear_content_index();

        for (entity, pos) in (&ent, &pos).join() {
            let idx = map.xy_idx(pos.x, pos.y);

            if let Some(_p) = blockers.get(entity) {
                map.blocked[idx] = true;
            }

            map.tile_content[idx].push(entity);
        }
    }
}
