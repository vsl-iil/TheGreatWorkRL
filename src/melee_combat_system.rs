use specs::prelude::*;
use crate::{components::{CombatStats, Name, Strength, SufferDamage, WantsToMelee}, gamelog::GameLog};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        ReadStorage<'a, Strength>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, mut wants_melee, mut inflict_dmg, names, combat_stats, strength) = data;

        for (entity, wants_melee, name, stats) in (&entities, &mut wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();
                    let modifier = if strength.contains(entity) { 2 } else { 1 };
                    let damage = i32::max(0, stats.power * modifier - target_stats.defence);

                    if damage == 0 {
                        log.entries.push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.entries.push(format!("{} hurts {} for {} hp", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut inflict_dmg, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}