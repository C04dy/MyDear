#![allow(unused_variables)]
#![allow(dead_code)]
#![warn(unused_imports)]

#[cfg(feature = "editor")]
mod editor;
#[cfg(not(feature = "editor"))]
mod game;
mod game_object;
mod level;
mod map;
mod renderer;
#[cfg(feature = "editor")]
mod utils;
mod vector2;

fn main() {
    #[cfg(feature = "editor")]
    if let Err(e) = editor::run() {
        eprintln!("Editor exited with error: {}", e);
        std::process::exit(1);
    }

    #[cfg(not(feature = "editor"))]
    if let Err(e) = game::run() {
        eprintln!("Game exited with error: {}", e);
        std::process::exit(1);
    }
}
