use std::fmt;
use std::ops::{Add, Div, Index, IndexMut};

use colored::{Colorize, CustomColor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: i32,
    pub y: i32,
}
impl Vector2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn to_colored_string(
        &self,
        x_color: CustomColor,
        y_color: CustomColor,
        punctuation_color: CustomColor,
    ) -> String {
        format!(
            "{}{}{}{}{}",
            "(".custom_color(punctuation_color).to_string(),
            self.x.to_string().custom_color(x_color).to_string(),
            ", ".custom_color(punctuation_color).to_string(),
            self.y.to_string().custom_color(y_color).to_string(),
            ")".custom_color(punctuation_color).to_string(),
        )
    }
}
impl Index<usize> for Vector2 {
    type Output = i32;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of bounds"),
        }
    }
}
impl IndexMut<usize> for Vector2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Index out of bounds"),
        }
    }
}
impl Div<i32> for &Vector2 {
    type Output = Vector2;

    fn div(self, other: i32) -> Self::Output {
        Vector2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Div<i32> for Vector2 {
    type Output = Vector2;

    fn div(self, other: i32) -> Self::Output {
        Vector2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Add for &Vector2 {
    type Output = Vector2;

    fn add(self, other: &Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
