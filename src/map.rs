use crate::game_object::*;
use crate::vector2::Vector2;
use colored::*;
use std::collections::HashMap;

pub struct Map {
    pub map_size: Vector2,
    pub objects: HashMap<GameObjectID, GameObject>,
    next_id: usize,
    pub positions_hashmap: HashMap<Vector2, GameObjectID>,
    pub ground_icon: ColoredString,

    pub moveable_components: HashMap<GameObjectID, MoveableComponent>,
    pub input_components: HashMap<GameObjectID, InputComponent>,
    pub event_components: HashMap<GameObjectID, EventComponent>,
    pub stats_components: HashMap<GameObjectID, StatsComponent>,

    pub camera_operator: GameObjectID, // this fella is gonna control the camera a.k.a. the player
    pub current_event_id: Option<GameObjectID>,
}
impl Map {
    pub fn new(map_size: Vector2, ground_icon: ColoredString) -> Self {
        Self {
            map_size,
            objects: HashMap::new(),
            next_id: 0,
            positions_hashmap: HashMap::new(),
            ground_icon,
            moveable_components: HashMap::new(),
            input_components: HashMap::new(),
            event_components: HashMap::new(),
            stats_components: HashMap::new(),
            camera_operator: 0,
            current_event_id: None,
        }
    }

    pub fn delete_object(&mut self, id: GameObjectID) {
        let Some(object) = self.objects.get(&id) else {
            return;
        };
        self.positions_hashmap.remove(&object.position);
        self.objects.remove(&id);
        self.moveable_components.remove(&id);
        self.input_components.remove(&id);
        self.event_components.remove(&id);
        self.stats_components.remove(&id);
    }

    pub fn insert_object(
        &mut self,
        position: Vector2,
        icon: ColoredString,
    ) -> Option<GameObjectID> {
        if self.positions_hashmap.contains_key(&position) {
            println!("{} coordinate is already occupied!", position);
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;

        let object: GameObject = GameObject { id, icon, position };

        self.positions_hashmap.insert(object.position, object.id);
        self.objects.insert(id, object);
        return Some(id);
    }
    pub fn insert_object_with_id(
        &mut self,
        id: GameObjectID,
        position: Vector2,
        icon: ColoredString,
    ) -> Option<GameObjectID> {
        if self.positions_hashmap.contains_key(&position) {
            println!("{} coordinate is already occupied!", position);
            return None;
        }
        let object = GameObject { id, icon, position };
        self.positions_hashmap.insert(object.position, object.id);
        self.objects.insert(id, object);
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        return Some(id);
    }

    pub fn insert_moveable_component(&mut self, id: GameObjectID) {
        Self::insert_component(MoveableComponent, &mut self.moveable_components, id);
    }
    pub fn insert_input_component(&mut self, id: GameObjectID) {
        Self::insert_component(InputComponent, &mut self.input_components, id);
    }
    pub fn insert_stats_component(&mut self, id: GameObjectID, stats: StatsComponent) {
        Self::insert_component(stats, &mut self.stats_components, id);
    }
    pub fn insert_event_component(&mut self, id: GameObjectID, events: Vec<EventStep>) {
        Self::insert_component(
            EventComponent {
                events,
                current_index: 0,
            },
            &mut self.event_components,
            id,
        );
    }
    fn insert_component<T>(
        what_component: T,
        component_hashmap: &mut HashMap<GameObjectID, T>,
        id: GameObjectID,
    ) {
        if component_hashmap.contains_key(&id) {
            //println!("Gameobject with {} id already has that Component", id);
            return;
        }

        component_hashmap.insert(id, what_component);
    }

    pub fn is_position_occupied(&self, position: &Vector2) -> bool {
        return self.positions_hashmap.contains_key(position);
    }

    pub fn is_out_of_bounds(&self, next_position: Vector2) -> bool {
        return next_position.x < 0
            || next_position.x >= self.map_size.x + 1
            || next_position.y < 0
            || next_position.y >= self.map_size.y + 1;
    }

    pub fn change_object_position(&mut self, id: GameObjectID, new_position: Vector2) -> bool {
        if self.is_position_occupied(&new_position) || self.is_out_of_bounds(new_position) {
            return false;
        }
        let Some(object) = self.objects.get_mut(&id) else {
            return false;
        };
        self.positions_hashmap.remove(&object.position);
        object.position = new_position;
        self.positions_hashmap.insert(new_position, id);
        return true;
    }

    pub fn get_event_around_this_position(&mut self, position: Vector2) -> Option<GameObjectID> {
        for y in -1..2 {
            for x in -1..2 {
                if x == 0 && y == 0 {
                    continue;
                }
                let current_pos = Vector2::new(x, y) + position;

                if let Some(id) = self.positions_hashmap.get(&current_pos)
                    && self.event_components.contains_key(id)
                {
                    return Some(*id);
                }
            }
        }
        return None;
    }
}
