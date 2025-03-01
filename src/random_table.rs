use rltk::RandomNumberGenerator;

#[derive(Clone, Copy)]
pub enum SpawnEntry {
    None,
    Goblin,
    Ork,
    HealingPotion,
    FireballScroll,
    ConfusionScroll,
    TeleportScroll,
    MissileScroll,
}

pub struct RandomEntry {
    name: SpawnEntry,
    weight: i32,
}

impl RandomEntry {
    pub fn new(name: SpawnEntry, weight: i32) -> Self {
        RandomEntry { name, weight }
    }
}

#[derive(Default)]
pub struct RandomTable {
    entries: Vec<RandomEntry>,
    total_weight: i32,
}

impl RandomTable {
    pub fn new() -> Self {
        RandomTable { entries: vec![], total_weight: 0 }
    }

    pub fn add(mut self, name: SpawnEntry, weight: i32) -> Self {
        self.total_weight += weight;
        self.entries.push(RandomEntry::new(name, weight));
        self
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> SpawnEntry {
        if self.total_weight == 0 { return SpawnEntry::None }
        let mut roll = rng.roll_dice(1, self.total_weight)-1;
        let mut index: usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].name;
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        SpawnEntry::None
    }
}