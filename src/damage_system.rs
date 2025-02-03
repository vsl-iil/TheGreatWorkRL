use rltk::console;
use specs::prelude::*;

use crate::{components::{CombatStats, Name, Player, SufferDamage}, gamelog::GameLog};

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
        let mut log = ecs.fetch_mut::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 { 
                let player = players.get(entity);
                match player {
                    None => { 
                        if let Some(victim_name) = names.get(entity) {
                            log.entries.push(format!("{} dies!", &victim_name.name));
                        }
                        dead.push(entity);
                    },
                    Some(_p) => log.entries.push("You are dead!".to_string()),
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete dead entity");
    }
}