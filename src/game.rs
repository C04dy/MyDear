use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute, terminal};
use std::io::{Write, stdout};
use std::{i32, io};

use crate::map::*;
use crate::vector2::*;

use kira::{
    AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use std::time::Duration;

pub struct Game {
    pub map: Map,
    pub camera: Vector2,
    pub screen_size: Vector2,
    pub screen_margins: Vector2,
    pub audio_manager: AudioManager,
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    control::set_override(true);

    let mut game: Game = Game {
        map: Map::new(
            Vector2::new(500, 500),
            String::from("○"),
            CustomColor::new(0, 255, 0),
        ),
        camera: Vector2::zero(),
        screen_size: Vector2::new(50, 20),
        screen_margins: Vector2::new(5, 3),
        audio_manager: generate_audio_manager().expect("Failed to initialize audio"),
    };

    if let Some(id) = game.map.insert_object(
        Vector2::new(6, 5),
        "♥︎".custom_color(CustomColor::new(255, 0, 0)),
    ) {
        game.map.insert_input_component(id);
        game.map.camera_operator = id;
    }

    if let Some(id) = game.map.insert_object(
        Vector2::new(7, 3),
        "1".custom_color(CustomColor::new(255, 255, 255)),
    ) {
        game.map.insert_moveable_component(id);
    }

    if let Some(id) = game.map.insert_object(
        Vector2::new(3, 3),
        "♥︎".custom_color(CustomColor::new(180, 0, 0)),
    ) {
        game.map.insert_input_component(id);
    }

    let mut frame_number: i32 = 0;
    loop {
        execute!(stdout, cursor::MoveTo(0, 0))?;

        print!("{}\r\n\r\n", frame_number);
        frame_number += 1;

        render(&game);

        stdout.flush()?;

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            if process_input(code, &mut game) == false {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(32)); // 30 fps
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}

fn generate_audio_manager() -> Result<AudioManager, Box<dyn std::error::Error>> {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    Ok(manager)
}

fn process_input(key: KeyCode, game: &mut Game) -> bool {
    if key == KeyCode::Char('q') {
        println!("Quitting... \r");
        return false;
    }
    match key {
        KeyCode::Up => move_objects(Vector2::new(0, -1), game),
        KeyCode::Down => move_objects(Vector2::new(0, 1), game),
        KeyCode::Left => move_objects(Vector2::new(-1, 0), game),
        KeyCode::Right => move_objects(Vector2::new(1, 0), game),
        //KeyCode::Char('e') => process_input(0, 0, &mut game),
        _ => {}
    }
    return true;
}

fn move_objects(direction: Vector2, game: &mut Game) {
    let ids: Vec<usize> = game.map.input_components.keys().cloned().collect();

    for id in ids {
        let next_position: Vector2 = game.map.objects[id].position + direction;

        if game.map.is_out_of_bounds(next_position) {
            return;
        }

        if let Some(moveable_id) = game.map.positions_hashmap.get(&next_position)
            && game.map.moveable_components.contains_key(moveable_id)
        {
            if game.map.change_object_position(
                *moveable_id,
                Vector2::new(direction.x, direction.y) + next_position,
            ) {
                game.map.change_object_position(id, next_position);
            }
        } else {
            game.map.change_object_position(id, next_position);
        }
    }

    let rel_x = game.map.objects[game.map.camera_operator].position.x - game.camera.x;
    if direction.x < 0 && rel_x < game.screen_margins.x {
        game.camera.x += direction.x;
    } else if direction.x > 0 && rel_x >= game.screen_size.x - game.screen_margins.x {
        game.camera.x += direction.x;
    }

    let rel_y = game.map.objects[game.map.camera_operator].position.y - game.camera.y;
    if direction.y < 0 && rel_y < game.screen_margins.y {
        game.camera.y += direction.y;
    } else if direction.y > 0 && rel_y >= game.screen_size.y - game.screen_margins.y {
        game.camera.y += direction.y;
    }
}

fn render(game: &Game) {
    let capacity = (game.screen_size.x * game.screen_size.y * 15) as usize;
    let mut buffer = String::with_capacity(capacity);

    for y in 0..game.screen_size.y {
        for x in 0..game.screen_size.x {
            let current_point = get_point_from_world_to_screen(&game.camera, &Vector2::new(x, y));
            if game.map.is_out_of_bounds(current_point) {
                buffer.push_str(" ");
                continue;
            }
            if let Some(id) = game.map.positions_hashmap.get(&current_point) {
                buffer.push_str(&game.map.objects[*id].icon.to_string());
            } else {
                buffer.push_str(&game.map.ground_icon.to_string());
            }
        }
        buffer.push_str("\r\n");
    }

    print!("{}", buffer);
}

fn get_point_from_world_to_screen(game_origin: &Vector2, screen_coordinate: &Vector2) -> Vector2 {
    return game_origin + screen_coordinate;
}
