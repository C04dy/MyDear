use std::collections::HashMap;

use crate::vector2::Vector2;
use colored::*;


struct Dialogue
{
    pub text: String,
    pub selections: Vec<String>,
    pub selections_pointing_index: Vec<i32>,
}

#[derive(PartialEq)]
pub enum GameObjectType 
{
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

pub struct GameObject
{
    pub name: String,
    pub position: Vector2,
    pub icon: ColoredString,
    pub object_type: GameObjectType,
    pub meta_data: HashMap<String, GameObjectMetaData>,
    pub current_dialogue_index: usize,
    pub current_dialogue_selection_index: usize,
    dialogues: Vec<Dialogue>,
    ceiling_group_id: i32,
}
impl GameObject
{
    pub fn new(_name: String, _position: Vector2, _icon: String, _object_type: GameObjectType, _object_color: CustomColor) -> Self 
    {
        Self 
        {
            name: _name,
            position: _position,
            icon: _icon.custom_color(_object_color),
            object_type: _object_type,
            meta_data: HashMap::new(),
            dialogues: Vec::new(),
            current_dialogue_index: 0,
            current_dialogue_selection_index: 0,
            ceiling_group_id: -1,
        }
    }

    pub fn get_current_dialogue_text(&self) -> &String
    {
        return &self.dialogues[self.current_dialogue_index].text;
    }

    pub fn get_current_dialogue_selections(&self) -> &Vec<String>
    {
        return &self.dialogues[self.current_dialogue_index].selections;
    }
    
    pub fn get_current_dialogue_selections_pointing_index(&self) -> i32
    {
        return self.dialogues[self.current_dialogue_index].selections_pointing_index[self.current_dialogue_selection_index];
    }
    
    pub fn get_current_dialogue_selections_length(&self) -> usize
    {
        return self.dialogues[self.current_dialogue_index].selections.len();
    }

    pub fn add_dialogue(&mut self, text: &str, selections: Vec<String>, selections_pointing_index: Vec<i32>)
    {
        self.dialogues.push(Dialogue { text: format!("{}: {}", self.name, text), selections: selections, selections_pointing_index: selections_pointing_index});
    }

    pub fn set_position(&mut self, _position: Vector2)
    {
        self.position = _position;
    }
    
    pub fn set_ceiling_group_id(&mut self, _id: i32)
    {
        self.ceiling_group_id = _id;
    }
    
    pub fn get_ceiling_group_id(&self) -> i32
    {
        return self.ceiling_group_id;
    }

    pub fn has_dialogues(&self) -> bool
    {
        !self.dialogues.is_empty() 
        && (self.current_dialogue_index as usize) < self.dialogues.len()
    }
}
