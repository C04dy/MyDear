use std::collections::HashMap;

use crate::vector2::Vector2;
use colored::*;

struct Dialogue {
    pub text: String,
    pub selections: Vec<String>,
    pub selections_pointing_index: Vec<i32>,
}

pub enum EventType {
    Dialogue = 0,
    Fight = 1,
}

pub struct Event {
    pub event_type: EventType,
    pub current_dialogue_selection_index: usize,
    pub current_dialogue_index: usize,
    dialogues: Vec<Dialogue>,
    owner_name: String,
}
impl Event {
    pub fn new(_type: EventType, owner_name: &str) -> Self {
        Self {
            event_type: _type,
            current_dialogue_selection_index: 0,
            current_dialogue_index: 0,
            dialogues: Vec::new(),
            owner_name: owner_name.to_string(),
        }
    }

    pub fn add_dialogue(
        &mut self,
        text: &str,
        selections: Vec<String>,
        selections_pointing_index: Vec<i32>,
    ) {
        self.dialogues.push(Dialogue {
            text: format!("{}: {}", self.owner_name, text),
            selections: selections,
            selections_pointing_index: selections_pointing_index,
        });
    }

    pub fn get_current_dialogue_text(&self) -> &String {
        return &self.dialogues[self.current_dialogue_index].text;
    }

    pub fn get_current_dialogue_selections(&self) -> &Vec<String> {
        return &self.dialogues[self.current_dialogue_index].selections;
    }

    pub fn get_current_dialogue_selections_pointing_index(&self) -> i32 {
        return self.dialogues[self.current_dialogue_index].selections_pointing_index
            [self.current_dialogue_selection_index];
    }

    pub fn get_current_dialogue_selections_length(&self) -> usize {
        return self.dialogues[self.current_dialogue_index].selections.len();
    }
}

pub struct GameObjectStats {
    pub strength: usize,
    pub agility: usize,
    pub max_health: usize,
}
impl GameObjectStats {
    pub fn new(strength: usize, agility: usize, max_health: usize) -> Self {
        Self {
            strength,
            agility,
            max_health,
        }
    }
    pub fn empty() -> Self {
        Self { 
            strength: 0,
            agility: 0,
            max_health: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum GameObjectType {
    MOVEABLE = 0,
    STATIC = 1,
    PLAYER = 2,
    CEILING = 3,
}

#[derive(PartialEq)]
pub enum GameObjectMetaData {
    EMPTY,
    INT(i32),
}

pub struct GameObject {
    pub name: String,
    pub position: Vector2,
    pub icon: ColoredString,
    pub object_type: GameObjectType,
    pub meta_data: HashMap<String, GameObjectMetaData>,
    pub events: Vec<Event>,
    pub current_event_index: usize,
    pub stat: GameObjectStats,
    pub health: usize,
    ceiling_group_id: i32,
}
impl GameObject {
    pub fn new(
        _name: String,
        _position: Vector2,
        _icon: String,
        _object_type: GameObjectType,
        _object_color: CustomColor,
        _object_stats: GameObjectStats,
    ) -> Self {
        Self {
            name: _name,
            position: _position,
            icon: _icon.custom_color(_object_color),
            object_type: _object_type,
            meta_data: HashMap::new(),
            events: Vec::new(),
            current_event_index: 0,
            stat: _object_stats,
            health: _object_stats.max_health,
            ceiling_group_id: -1,
        }
    }

    pub fn add_event(&mut self, _type: EventType) -> Option<&mut Event> {
        self.events.push(Event::new(_type, &self.name));
        return self.events.last_mut();
    }

    pub fn set_position(&mut self, _position: Vector2) {
        self.position = _position;
    }

    pub fn set_ceiling_group_id(&mut self, _id: i32) {
        self.ceiling_group_id = _id;
    }

    pub fn get_ceiling_group_id(&self) -> i32 {
        return self.ceiling_group_id;
    }

    pub fn has_event(&self) -> bool {
        !self.events.is_empty() && (self.current_event_index) < self.events.len()
    }
}
