mod game;
mod game_object;
mod map;
mod vector2;

fn main() {
    if let Err(e) = crate::game::run() {
        eprintln!("Game exited with error: {}", e);
        std::process::exit(1);
    }
}