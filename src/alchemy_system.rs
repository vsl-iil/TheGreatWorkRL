use std::collections::HashMap;

use rand::{seq::SliceRandom, SeedableRng};
use rltk::RGB;
use specs::prelude::*;

use crate::{components::{Confusion, Consumable, Explosion, InBackpack, InstantHarm, Invulnerability, Item, LingerType, LingeringEffect, Name, Potion, ProvidesHealing, Renderable, Strength, Teleport, WantsToMixPotions, Weight}, gamelog::GameLog, AlchemySeed};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PotionEffect {
    Heal(ProvidesHealing),
    Teleport(Teleport),
    Confusion(Confusion),
    Harm(InstantHarm),
    Linger(LingeringEffect),
    Explosion(Explosion),
    Invulnerability(Invulnerability),
    Strength(Strength)
}

pub struct AlchemySystem {}

impl<'a> System<'a> for AlchemySystem {
    type SystemData = ( WriteStorage<'a, WantsToMixPotions>,
                        Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Entity>    ,
                        WriteStorage<'a, Renderable>,
                        WriteStorage<'a, Potion>,
                        WriteStorage<'a, Item>,
                        WriteStorage<'a, Consumable>,
                        WriteStorage<'a, InBackpack>,
                        WriteStorage<'a, Name>,
                        WriteStorage<'a, Weight>,
                        ReadExpect<'a, AlchemySeed>,
                        
                        WriteStorage<'a, ProvidesHealing>,
                        WriteStorage<'a, Teleport>,
                        WriteStorage<'a, Confusion>,
                        WriteStorage<'a, InstantHarm>,
                        WriteStorage<'a, LingeringEffect>,
                        WriteStorage<'a, Explosion>,
                        WriteStorage<'a, Invulnerability>,
                        WriteStorage<'a, Strength>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut intentmix, entities, mut log, playerentity, mut renders, mut potions, mut items, mut consumables, mut inbackpack, mut names, mut weight, seed,   mut heal, mut tp, mut confusion, mut harm, mut linger, mut explosion, mut invuln, mut strength) = data;

        for intent in (&mut intentmix).join() {
            let WantsToMixPotions { first, second } = intent;
            if !potions.contains(*first) || !potions.contains(*second) {
                log.entries.push("You cannot mix that.".to_owned());
                continue;
            }
            entities.delete(*first).expect("Unable to delete first mix component");
            entities.delete(*second).expect("Unable to delete second mix component");

            // special case
            // heal + harm combo
            if  heal.contains(*first) && harm.contains(*second) ||
                harm.contains(*first) && heal.contains(*second) {
                // BOOOOM!!!

                // entities.delete(*first).expect("Unable to delete first mix component");
                // entities.delete(*second).expect("Unable to delete second mix component");
                log.entries.push("The mix violently explodes!".to_owned());

                explosion.insert(*playerentity, crate::components::Explosion { maxdmg: 20, radius: 5 })
                         .expect("Unable to explode the player");

                continue;
            }

            let mut effects_first: Vec<PotionEffect> = vec![];
            let mut effects_second: Vec<PotionEffect> = vec![];

            // INFLICTS
            let mut contains: u8 = 0;
            use PotionEffect::*;
            heal     .get(*first).map(|h| { effects_first.push(Heal(*h)); contains |= 1});
            tp       .get(*first).map(|t| { effects_first.push(Teleport(*t)); contains |= 2});
            confusion.get(*first).map(|c| { effects_first.push(Confusion(*c)); contains |= 4});
            harm     .get(*first).map(|h| { effects_first.push(Harm(*h)); contains |= 8});
            linger   .get(*first).map(|l| { effects_first.push(Linger(*l)); contains |= 16});
            explosion.get(*first).map(|e| { effects_first.push(Explosion(*e)); contains |= 32});

            heal     .get(*second).map(|h| { effects_second.push(Heal(*h)); contains |= 1});
            tp       .get(*second).map(|t| { effects_second.push(Teleport(*t)); contains |= 2});
            confusion.get(*second).map(|c| { effects_second.push(Confusion(*c)); contains |= 4});
            harm     .get(*second).map(|h| { effects_second.push(Harm(*h)); contains |= 8});
            linger   .get(*second).map(|l| { effects_second.push(Linger(*l)); contains |= 16});
            explosion.get(*second).map(|e| { effects_second.push(Explosion(*e)); contains |= 32});

            let mut color = renders.get(*first).map_or(RGB::named(rltk::GREEN), |c| c.fg);
            // entities.delete(*first).expect("Unable to delete first mix component");
            // entities.delete(*second).expect("Unable to delete second mix component");

            let new_potion = entities.create();
            items.insert(new_potion, Item {}).expect("Unable to insert item in mix");
            potions.insert(new_potion, Potion {}).expect("Unable to insert potion in mix");
            consumables.insert(new_potion, Consumable {}).expect("Unable to insert consumable in mix");
            
            let mut name: Vec<String> = Vec::new();
            
            let specials = generate_combos(seed.0);

            if specials.contains_key(&contains) {
                effects_first = vec![*specials.get(&contains).unwrap()];
            } else {
                effects_first.append(&mut effects_second);
            }

            effects_first.sort();
            effects_first = effects_first.iter().fold(vec![], |mut acc, effect| {
                if acc.len() > 0 && *acc.last().unwrap() == *effect {
                    let new_effect = acc.pop();
                    match new_effect {
                        None => return vec![],
                        Some(new_effect) => match (effect, new_effect) {
                            (&Heal(mut h1), Heal(h2)) => {
                                h1.heal_amount += h2.heal_amount;
                                acc.push(Heal(h1));
                            }
                            (&Harm(mut h1), Harm(h2)) => {
                                h1.dmg = ((h1.dmg + h2.dmg) as f32 * 0.6).round() as i32;
                                acc.push(Harm(h1));
                            }
                            (&Explosion(mut e1), Explosion(e2)) => {
                                e1.maxdmg = ((e1.maxdmg + e2.maxdmg) as f32 * 0.6).round() as i32;
                                acc.push(Explosion(e1));
                            }

                            (&Linger(mut l1), Linger(l2)) => {
                                if l2.etype == LingerType::Fire {
                                    l1.etype = LingerType::Fire;
                                }
                                l1.duration = ((l1.duration + l2.duration) as f32 * 0.6).round() as i32;
                                acc.push(Linger(l1));
                            }
                            (&Confusion(mut c1), Confusion(c2)) => {
                                c1.turns = ((c1.turns + c2.turns) as f32 * 0.6).round() as i32;
                                acc.push(Confusion(c1));
                            }

                            (_, popped) => { acc.push(popped) }
                        }
                    }
                } else {
                    acc.push(*effect);
                }
                return acc;
            });
            for effect in effects_first {
                match effect {
                    Heal(h) => { 
                        heal.insert(new_potion, h).expect("Unable to insert heal in mix"); 
                        color = mix_colors(color, RGB::named(rltk::MAGENTA));
                        name.push("Health".to_owned());
                    },
                    Teleport(t) => { 
                        tp.insert(new_potion, t).expect("Unable to insert tp in mix"); 
                        color = mix_colors(color, RGB::named(rltk::VIOLET));
                        name.push("Teleport".to_owned());
                    },
                    Confusion(c) => { 
                        confusion.insert(new_potion, c).expect("Unable to insert confusion in mix"); 
                        color = mix_colors(color, RGB::named(rltk::PINK));
                        name.push("Confusion".to_owned());
                    },
                    Harm(h) => { 
                        harm.insert(new_potion, h).expect("Unable to insert harm in mix"); 
                        color = mix_colors(color, RGB::named(rltk::DARKRED));
                        name.push("Harm".to_owned());
                    },
                    Linger(l) => { 
                        linger.insert(new_potion, l).expect("Unable to insert linger in mix"); 
                        let color2 = match l.etype {
                            crate::components::LingerType::Fire => {
                                name.push("Fire".to_owned());
                                RGB::named(rltk::RED)
                            },
                            crate::components::LingerType::Poison => {
                                name.push("Poison".to_owned());
                                RGB::named(rltk::GREEN)
                            },
                        };
                        color = mix_colors(color, color2);
                    },
                    Explosion(e) => { 
                        name.push("Explosion".to_owned());
                        explosion.insert(new_potion, e).expect("Unable to insert explosion in mix"); 
                        color = mix_colors(color, RGB::named(rltk::ORANGE));
                    },
                    // special cases
                    Invulnerability(invul) => {
                        name.push("Invulnerability".to_owned());
                        invuln.insert(new_potion, invul).expect("Unable to insert invul in mix");
                        color = RGB::named(rltk::GOLD);
                    }
                    Strength(strong) => {
                        name.push("Strength".to_owned());
                        strength.insert(new_potion, strong).expect("Unable to insert strength in mix");
                        color = RGB::named(rltk::BLUE);
                    }
                }
            }

            renders.insert(new_potion, Renderable { 
                glyph: rltk::to_cp437('¡'), 
                fg: color, 
                bg: RGB::named(rltk::BLACK), 
                render_order: 2 
            }).expect("Unable to insert renderable in mix");
        
            inbackpack.insert(new_potion, InBackpack { owner: *playerentity }).expect("Unable to insert mix in backpack");
            let new_weight = weight.get(*first).map_or(1, |w| w.0) + weight.get(*second).map_or(1, |w| w.0);
            weight.insert(new_potion, Weight(new_weight)).expect("Unable to insert mix weight");

            name.dedup();
            let mut name = name.join(" + ");
            name.push_str(" potion");
            names.insert(new_potion, Name { name }).expect("Unable to name mix");
        }
    
        intentmix.clear();
    }
}

fn mix_colors(color1: RGB, color2: RGB) -> RGB {
    rltk::RgbLerp::new(color1, color2, 3)
                  .skip(1)
                  .next()
                  .unwrap_or(RGB::named(rltk::GREEN))
}

fn generate_combos(seed: u64) -> HashMap<u8, PotionEffect> {
    let mut hashmap = HashMap::new();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut combos: Vec<u8> = (1..=5).map(|p| u8::pow(2, p as u32)).collect();

    combos.shuffle(&mut rng);

    let mut all_except_tp = combos.iter().filter(|n| **n != 2);
    let mut random_pair = combos.choose_multiple(&mut rng, 2);
    hashmap.insert(all_except_tp.next().unwrap() | 2, PotionEffect::Invulnerability(Invulnerability { turns: 3 }));
    hashmap.insert(all_except_tp.next().unwrap() | 2, PotionEffect::Strength(Strength { turns: 3 }));
    hashmap.insert(random_pair.next().unwrap() | random_pair.next().unwrap(), PotionEffect::Heal(ProvidesHealing { heal_amount: 3 }));

    hashmap

}