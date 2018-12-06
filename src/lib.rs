#[macro_use]
extern crate serde_derive;

pub use utils::get_world::get_world;

mod cli;
mod item;
mod pathway;
mod properties;
mod room;
mod utils;
mod world;
