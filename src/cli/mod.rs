use std::{
    cell::{Cell, RefCell},
    convert::TryInto,
    fs::{self, File},
    io::{self, BufReader, Read, Write},
};

use serde::{Deserialize, Serialize};

use crate::{
    entity::{Element, Enemy, Entity, Item},
    input::{read_line, CmdTokens, Lexer, Parser},
    player::Player,
    types::{Action, Class, CmdResult, Race},
    world::World,
};

/// The Cli type provides a simple way to interface into the mechanics of Kingslayer with custom worlds
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Cli {
    #[serde(default)]
    running: Cell<bool>,
    #[serde(default)]
    last_cmd_res: RefCell<CmdResult>,
    #[serde(default)]
    last_cmd: RefCell<CmdTokens>,
    #[serde(default)]
    player: RefCell<Box<Player>>,
    world: RefCell<Box<World>>,
}

impl Cli {
    /// Construct from a RON file
    pub fn from_file(path: &str) -> Self {
        let metadata = fs::metadata(path).expect("Error getting metadata from file");
        let world_file = File::open(path).expect("Unable to open world file");
        let mut world_file_reader = BufReader::new(world_file);
        let mut data = String::with_capacity(metadata.len().try_into().unwrap());
        world_file_reader
            .read_to_string(&mut data)
            .expect("Unable to read string from world file");

        ron::de::from_str(&data).expect("Error creating world from RON file.")
    }

    /// Construct from a string containing RON
    pub fn from_ron_str(ron: &str) -> Self {
        ron::de::from_str(ron).expect("Error creating world from string.")
    }

    /// Prompts the user for input from stdin
    pub fn prompt(&self, prompt: &str) -> String {
        loop {
            if self.last_cmd_res.borrow().has_request() {
                print!("\n{}: ", prompt);
            } else {
                print!("\n{}> ", prompt);
            }
            io::stdout().flush().expect("Error flushing stdout");

            let input = read_line();
            if !input.is_empty() && input.len() < 100 {
                return input;
            } else {
                println!("Excuse me?");
            }
        }
    }

    /// Returns a helpful list of game commands
    pub fn help() -> CmdResult {
        CmdResult::new(
            Action::Passive,
            "Typical format (but the game is quite lenient): 
    <action> [object] [preposition] [object]

    some prepositions: in, inside, from, on, with

Some available commands:

    Explore around the world
        go, enter       move in a direction or through a listed entrance
            Directions can be short like:
                n, s, e, w, ne, nw, se, sw, u, d
            or long like:
               north, south, east, west,
               northeast, northwest, southeast, southwest,
               up, down, (any other listed entrance)
        
        r               repeat last command
        l, look         look around the room
        open | close    open/close an item or pathway

    Manipulate items found in the world
        take            put an item from the room into your inventory
        drop            drop an item from your inventory into the room
        i, inventory    print the contents of your inventory
        x, examine      show additional information about an item
        draw, equip     use an item from your inventory as your default weapon
        don, put on     don a set of armor to increase your armor class
        kill            attack an enemy with your main hand or a chosen weapon

    Manage your character
        heal            replenish some HP
        increase        increase a chosen ability score by 1 if stat points are available
        c | stats          display information on the state of your character"
                .to_owned(),
        )
    }

    /// Start a typical game for the command line
    pub fn start(&self) {
        if !self.running.get() {
            self.create_character();
        }

        println!("Type \"help\" if you are unfamiliar with text-based games.\n");
        println!("{}", self.ask("l"));

        self.running.set(true);
        while self.running.get() && self.player.borrow().is_alive() {
            println!("{}", self.ask(&self.prompt("")));
        }
    }

    pub fn create_character(&self) {
        self.player
            .borrow_mut()
            .set_race(Race::select_race(&self.prompt(&Race::race_prompt())));

        self.player
            .borrow_mut()
            .set_class(Class::select_class(&self.prompt(&Class::class_prompt())));
    }

    /// Handle user input and return the results of commands and events
    pub fn ask(&self, input: &str) -> String {
        let command = Lexer::lex(input);

        let res = if let Some(last_cmd_res) = self.last_cmd_res.borrow().request_input() {
            let last_cmd_res = if last_cmd_res.obj().is_some() {
                last_cmd_res.with_obj_prep(command.verb_clone())
            } else {
                last_cmd_res.with_obj(command.verb_clone())
            };
            Parser::parse(
                last_cmd_res,
                &mut self.world.borrow_mut(),
                &mut self.player.borrow_mut(),
            )
        } else {
            match command.verb() {
                Some("save") => self.save(),
                Some("r") => Parser::parse(
                    self.last_cmd.borrow().clone(),
                    &mut self.world.borrow_mut(),
                    &mut self.player.borrow_mut(),
                ),
                _ => {
                    self.last_cmd.replace(command.clone());
                    Parser::parse(
                        command,
                        &mut self.world.borrow_mut(),
                        &mut self.player.borrow_mut(),
                    )
                }
            }
        };

        self.last_cmd_res.replace(res.clone());

        if res.is_active() {
            format!("{}{}", res.output(), self.combat())
        } else {
            res.output().to_owned()
        }
    }

    // manages actions taken by Enemies in the current room
    fn combat(&self) -> String {
        let mut events_str =
            String::with_capacity(50 * self.world.borrow().get_curr_room().enemies().len());

        for enemy in self.world.borrow_mut().get_curr_room_mut().enemies_mut() {
            if enemy.is_angry() && enemy.is_alive() {
                let enemy_damage = enemy.damage();

                events_str.push_str(
                    &self
                        .player
                        .borrow_mut()
                        .take_damage(enemy.name(), enemy_damage),
                );

                self.player.borrow_mut().engage_combat();
            }
            if !enemy.is_alive() {
                events_str.push_str(&format!("\nYou gained {} XP.\n", enemy.xp()));
                self.player.borrow_mut().disengage_combat();
                self.player.borrow_mut().gain_xp(enemy.xp());
            }
        }
        self.world.borrow_mut().clear_dead_enemies();

        if !self.player.borrow().is_alive() {
            events_str.push_str("\n\nYou died.");
        } else {
            events_str.push_str(&self.player.borrow_mut().level_up());
        }
        events_str.shrink_to_fit();
        events_str
    }

    pub fn add_element(&self, room: &str, el: Element) {
        self.world.borrow_mut().add_element(room, el)
    }

    pub fn add_item(&self, room: &str, item: Item) {
        self.world.borrow_mut().add_item(room, item)
    }

    pub fn spawn_enemy(&self, room: &str, enemy: Enemy) {
        self.world.borrow_mut().spawn_enemy(room, enemy)
    }

    fn save(&self) -> CmdResult {
        let saved = ron::ser::to_string(&self).expect("Error serializing world save file.");

        if let Ok(mut file) = File::create("world.save.ron") {
            if let Ok(()) = file.write_all(saved.as_bytes()) {
                CmdResult::new(Action::Passive, String::from("Saved to 'world.save.ron'."))
            } else {
                CmdResult::new(Action::Passive, String::from("Error saving world."))
            }
        } else {
            CmdResult::new(Action::Passive, String::from("Error saving world."))
        }
    }
}
