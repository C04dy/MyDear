use std::ops::{Add, Div};
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct Vector2 
{
    pub x: i32,
    pub y: i32,
}
impl Vector2 
{
    pub fn new(x: i32, y: i32) -> Self 
    {
        Self { x, y }
    }
    pub fn zero() -> Self
    {
        Self { x: 0, y: 0 }
    }
}
impl Div<i32> for &Vector2
{
    type Output = Vector2;

    fn div(self, other: i32) -> Self::Output {
        Vector2 
        {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Add for Vector2
{
    type Output = Vector2;

    fn add(self, other: Vector2) -> Self {
        Self 
        {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Add for &Vector2 
{
    type Output = Vector2;

    fn add(self, other: &Vector2) -> Vector2 
    {
        Vector2 
        {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl fmt::Display for Vector2
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        write!(f, "({}, {})", self.x, self.y)
    }
}
