use serde::Deserialize;

use crate::entity::Entity;

// An object to be interacted with by the user
#[derive(Debug, Deserialize)]
pub struct Armor {
    name: String,
    desc: String,
    inspect: String,
    ac: u32,
}

impl Armor {
    pub fn ac(&self) -> u32 {
        self.ac
    }
}

impl Entity for Armor {
    fn name(&self) -> &str {
        &self.name
    }

    fn desc(&self) -> &str {
        &self.desc
    }

    fn inspect(&self) -> &str {
        &self.inspect
    }
}
