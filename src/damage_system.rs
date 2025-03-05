use specs::prelude::*;

use crate::{components::{Boss, CombatStats, Name, Player, SufferDamage}, gamelog::GameLog, map::{Map, TileType}};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (stats, damage) in (&mut stats, &damage).join() {
            for dmg in damage.amount.iter() {
                stats.hp = stats.hp.saturating_sub(*dmg);
            }
        }

        damage.clear();
    }
}

pub fn clean_up_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = vec![];
    let mut is_boss_dead = false;
    {
        let names = ecs.read_storage::<Name>();
        let boss = ecs.read_storage::<Boss>();
        
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let mut log = ecs.fetch_mut::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 { 
                let player = players.get(entity);
                match player {
                    None => { 
                        is_boss_dead = boss.get(entity).is_some();
                        if !is_boss_dead {
                            if let Some(victim_name) = names.get(entity) {
                                log.entries.push(format!("{} dies!", &victim_name.name));
                            }
                        }
                        dead.push(entity);
                    },
                    Some(_p) => {
                        let msg_dead = "You are dead!".to_string();
                        if log.entries.iter().last().is_some_and(|msg| msg != &msg_dead) {
                            log.entries.push(msg_dead);
                        }
                    },
                }
            }
        }
    }

    if is_boss_dead { boss_dead(ecs) }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete dead entity");
    }
}

pub fn boss_dead(ecs: &mut World) {
    {
        let mut map = ecs.write_resource::<Map>();
        
        for tile in map.tiles.iter_mut() {
            if *tile == TileType::FinalDoor {
                *tile = TileType::Floor;
            }
        }
    }
    let mut log = ecs.write_resource::<GameLog>();
    log.entries.push(String::new());
    log.entries.push("\"You are a fool... You'll never leave...\"".to_owned());
    log.entries.push("The Cursed Alchemist dies!".to_owned());
    log.entries.push("You hear a rumbling sound; the door to the chamber opens!".to_owned());
    
}