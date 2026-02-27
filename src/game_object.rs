use crate::vector2::Vector2;
use colored::*;

pub type GameObjectID = usize; // -1 means the object should not be real.. yeah
pub struct InputComponent; // anything with this component should be moved with keyboard input
pub struct MoveableComponent; // this allows an object to move another object with this component

pub struct GameObject {
    pub id: GameObjectID,
    pub icon: ColoredString,
    pub position: Vector2, // ALWAYS CHANGE POSITION FROM MAP FUNCTION, when the position changed, the hashmap on the map struct that point to the id should be changed, with that we can see if we should render the object
}
