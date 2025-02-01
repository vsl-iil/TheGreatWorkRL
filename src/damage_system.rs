use rltk::console;
use specs::prelude::*;

use crate::components::{CombatStats, Name, Player, SufferDamage};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn clean_up_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = vec![];
    {
        let names = ecs.read_storage::<Name>();
        
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 { 
                let player = players.get(entity);
                match player {
                    None => { 
                        let name = names.get(entity).unwrap_or(&Name { name: "Unnamed".to_string() }).name.clone();
                        console::log(format!("{} dies!", &name));
                        dead.push(entity);
                    },
                    Some(_p) => console::log("You are dead!"),
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete dead entity");
    }
}