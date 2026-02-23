use crate::game;
use crate::game::Game;
use crate::vector2;
use crate::vector2::Vector2;
use crate::game_object::GameObject;
use crate::game_object::GameObjectType;
use std::collections::HashMap;
use colored::*;

pub struct Map
{
    pub map_size: Vector2,
    pub objects: HashMap<Vector2, GameObject>,
    pub ceilings: HashMap<Vector2, GameObject>,
    pub ceiling_keys: HashMap<i32, Vec<Vector2>>,
    pub ground_icon: ColoredString,
    pub current_dialogue_position: Vector2,
}
impl Map 
{
    pub fn new(_map_size: Vector2, _ground_icon: String, _ground_color: CustomColor) -> Self
    {
        Self {
            map_size: _map_size,
            objects: HashMap::new(),
            ceilings: HashMap::new(),
            ceiling_keys: HashMap::new(),
            ground_icon: _ground_icon.custom_color(_ground_color),
            current_dialogue_position: Vector2::zero(),
        }
    }
    
    pub fn insert_object(&mut self, object: GameObject)
    {
        if object.object_type == GameObjectType::CEILING
        {
            self.insert_ceiling(object);
            return;
        }
        
        if self.objects.contains_key(&object.position)
        {
            println!("{} coordinate is already occupied!", object.position);
            return;
        }

        self.objects.insert(object.position, object);
    }
    
    pub fn insert_ceiling(&mut self, object: GameObject)
    {
        if self.ceilings.contains_key(&object.position)
        {
            println!("{} coordinate is already occupied!", object.position);
            return;
        }
        if object.object_type != GameObjectType::CEILING
        {
            println!("insert a ceiling");
            return;
        }
        if object.get_ceiling_group_id() < 0
        {
            println!("set the ceilings group id before inserting");
            return;
        }

        let object_key = object.position;
        let group_id = object.get_ceiling_group_id(); 

        self.ceilings.insert(object_key, object);
        self.ceiling_keys.entry(group_id).or_insert(Vec::new()).push(object_key);
    }

    pub fn get_ceiling_by_id(&self, group_id: i32) -> Vec<&GameObject>
    {
        self.ceiling_keys.get(&group_id)
        .map(|positions| {
            positions.iter()
                .filter_map(|pos| self.objects.get(pos))
                .collect()
        })
        .unwrap_or_default()
    }

    pub fn get_ceiling_from_position(&self, position: Vector2) -> Option<&GameObject>
    {
        return self.ceilings.get(&position);
    }

    pub fn trigger_if_there_is_dialogue_at_this_position(&mut self, position: Vector2) -> bool
    {
        for y in -1..2 {
            for x in -1..2 {
                if x == 0 && y == 0
                {
                    continue;
                }
                let current_pos = Vector2::new(x, y) + position;
                
                if let Some(object) = self.objects.get_mut(&current_pos) && object.has_dialogues()
                {
                    object.current_dialogue_selection_index = 0;
                    self.current_dialogue_position = current_pos;
                    return true;
                }
            }
        } 
        return false;
    }

    pub fn get_object_from_position(&self, position: Vector2) -> Option<&GameObject>
    {
        return self.objects.get(&position);
    }
    
    pub fn is_position_occupied(&self, position: Vector2) -> bool
    {
        if self.get_object_from_position(position).is_some()
        {
            return true;
        }
        return false;
    }
    
    pub fn is_out_of_bounds(&self, next_position: Vector2) -> bool
    {
        return next_position.x < 0 || next_position.x >= self.map_size.x + 1 || next_position.y < 0 || next_position.y >= self.map_size.y + 1;    
    }
    
    pub fn push_object(&mut self, current_pos: Vector2, direction_x: i32, direction_y: i32) -> bool
    {
        let next_pos = Vector2::new(current_pos.x + direction_x, current_pos.y + direction_y);

        if self.is_out_of_bounds(next_pos) || self.objects.contains_key(&next_pos) 
        {
            return false;
        }

        let Some(mut object) = self
            .objects
            .remove(&current_pos)
        else {
            return false;
        };

        object.position = next_pos;
        self.objects.insert(next_pos, object);
        return true;
    }
}
