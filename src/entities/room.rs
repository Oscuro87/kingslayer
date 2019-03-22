use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::entities::{Enemy, Item, Pathway};

// A section of the world connected by paths
#[derive(Serialize, Deserialize)]
pub struct Room {
    name: String,
    desc: String,
    paths: HashMap<String, Pathway>,
    enemies: HashMap<String, Box<Enemy>>,
    items: HashMap<String, Box<Item>>,
}

impl Room {
    // compiles all descriptions in the Room for printing
    pub fn desc(&self) -> String {
        let mut desc = format!("{}\n{}", self.name, self.desc);
        for x in self.paths.iter() {
            desc.push_str(&format!("\n{}", &(x.1).desc()));
        }
        for x in self.enemies.iter() {
            desc.push_str(&format!("\n{}", &x.1.desc()));
        }
        for x in self.items.iter() {
            desc.push_str(&format!("\n{}", &x.1.desc()));
        }
        desc
    }

    pub fn paths(&self) -> &HashMap<String, Pathway> {
        &self.paths
    }
    pub fn paths_mut(&mut self) -> &mut HashMap<String, Pathway> {
        &mut self.paths
    }

    pub fn enemies(&self) -> &HashMap<String, Box<Enemy>> {
        &self.enemies
    }
    pub fn enemies_mut(&mut self) -> &mut HashMap<String, Box<Enemy>> {
        &mut self.enemies
    }

    pub fn items(&self) -> &HashMap<String, Box<Item>> {
        &self.items
    }
    pub fn items_mut(&mut self) -> &mut HashMap<String, Box<Item>> {
        &mut self.items
    }
}