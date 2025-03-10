use specs::prelude::*;
use specs_derive::*;

use std::convert::Infallible;
type NoError = Infallible;
use serde::{Serialize, Deserialize};
use specs::saveload::{Marker, ConvertSaveload};
use rltk::{Point, RGB};

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone, Copy, Debug)]
pub struct Bomber { 
    pub effect: Entity 
}

#[derive(Component, ConvertSaveload, Clone, Copy, Debug)]
pub struct Lobber {
    pub turns: u32,
    pub targetpos: Option<Point>
}

#[derive(Component, Debug, ConvertSaveload, Clone, Copy)]
pub struct Boss {
    pub state: BossState,
    pub targetpos: Option<Point>
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct MacGuffin {}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum BossState {
    ThrowingPotions(i32),
    ClosingIn(i32),
    GainingDistance(i32)
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defence: i32,
    pub power: i32
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage!");
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Item {}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Consumable {}

/// Лужа зелья
#[derive(Component, Debug, ConvertSaveload, Clone, Copy)]
pub struct Puddle { 
    pub lifetime: i32 
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Potion {}
//
// ---=== Эффекты мобов / эффекты зелий ===---
//

#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

/// Safe teleport will always transport you to an empty place.
/// Unsafe may teleport you inside a wall.
#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Teleport {
    pub safe: bool
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LingerType { Fire, Poison }

// Продолжительный наносящий урон эффект (огонь, отравление)
#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LingeringEffect {
    pub etype: LingerType,
    pub duration: i32,
    pub dmg: i32
}

// Моментальный урон. Имеет особое взаимодействие с зельем лечения...
#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstantHarm {
    pub dmg: i32,
}

// Boom!
#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Explosion {
    pub maxdmg: i32,
    pub radius: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Invulnerability {
    pub turns: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Strength {
    pub turns: i32
}

// TODO polymorph???

// ============================================

#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToThrowItem {
    pub item: Entity,
    pub target: Point
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToMixPotions {
    pub first: Entity,
    pub second: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Confusion {
    pub turns: i32
}

/// Enemy is awake and active
#[derive(Component, Debug, ConvertSaveload)]
pub struct Agitated {
    pub turns: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Weight(pub i32);

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32
}

pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map : super::map::Map
}
