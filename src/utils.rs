use std::ops::{Add, Rem, Sub};

pub fn wrap_add_reverse<T>(value: T, add_value: T, max_value: T) -> T
where
    T: Add<Output = T> + Rem<Output = T> + Copy,
{
    (value + add_value) % max_value
}
pub fn wrap_add<T>(value: T, add_value: T, max_value: T, min_value: T) -> T
where
    T: Add<Output = T> + Rem<Output = T> + Copy + PartialEq,
{
    if value == max_value {
        min_value
    } else {
        value + add_value
    }
}

pub fn wrap_remove<T>(value: T, remove_value: T, max_value: T, min_value: T) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Rem<Output = T> + Copy + PartialEq,
{
    if value == min_value {
        max_value
    } else {
        value - remove_value
    }
}
