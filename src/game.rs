use rand::prelude::*;
use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::{i32, io};

use crate::game_object::*;
use crate::map::*;
use crate::vector2::*;

use kira::{
    AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use std::time::Duration;

#[derive(PartialEq)]
pub enum GameState {
    Normal = 0,
    Dialogue = 1,
    Cutscene = 2,
    Fight = 3,
}
pub struct Game {
    pub player: GameObject,
    pub map: Map,
    pub camera: Vector2,
    pub screen_size: Vector2,
    pub screen_margins: Vector2,
    pub game_state: GameState,
    pub audio_manager: AudioManager,
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;

    let mut game: Game = Game {
        player: GameObject::new(
            String::from("Player"),
            Vector2::new(6, 5),
            String::from("♥︎"),
            GameObjectType::PLAYER,
            CustomColor::new(255, 0, 0),
            GameObjectStats::new(1, 1)
        ),
        map: Map::new(
            Vector2::new(500, 500),
            String::from("○"),
            CustomColor::new(0, 255, 0),
            vec![String::from("Fight"), String::from("Run")],
        ),
        camera: Vector2::zero(),
        screen_size: Vector2::new(50, 20),
        screen_margins: Vector2::new(5, 3),
        game_state: GameState::Normal,
        audio_manager: generate_audio_manager().expect("Failed to initialize audio"),
    };
    game.player.meta_data.insert(
        "CeilingIDPlayerIsOnBelow".to_string(),
        GameObjectMetaData::INT(-1),
    );

    let moveable_box: GameObject = GameObject::new(
        String::from("Box"),
        Vector2::new(7, 3),
        String::from("1"),
        GameObjectType::MOVEABLE,
        CustomColor::new(255, 255, 255),
        GameObjectStats::empty(),
    );
    game.map.insert_object(moveable_box);

    let mut enemy: GameObject = GameObject::new(
        String::from("Enemy"),
        Vector2::new(3, 3),
        String::from("♥︎"),
        GameObjectType::STATIC,
        CustomColor::new(180, 0, 0),
        GameObjectStats::new(1, 2),
    );
    enemy.add_event(EventType::Fight);

    game.map.insert_object(enemy);

    for y in 0..9 {
        for x in 0..13 {
            if y > 0 && x > 0 && x < 12 {
                continue;
            }
            let wall: GameObject = GameObject::new(
                String::from("Wall"),
                Vector2::new(1 + x, 1 + y),
                String::from("#"),
                GameObjectType::STATIC,
                CustomColor::new(255, 255, 255),
                GameObjectStats::empty(),
            );
            game.map.insert_object(wall);
        }
    }
    for y in 0..11 {
        for x in 0..15 {
            let mut ceiling = GameObject::new(
                String::from("Ceiling"),
                Vector2::new(0 + x, 0 + y),
                String::from("█"),
                GameObjectType::CEILING,
                CustomColor::new(0, 0, 255),
                GameObjectStats::empty(),
            );
            ceiling.set_ceiling_group_id(0);
            game.map.insert_object(ceiling);
        }
    }


    if let Some(ceiling) = game.map.get_ceiling_from_position(game.player.position) {
        game.player
            .meta_data
            .entry("CeilingIDPlayerIsOnBelow".to_string())
            .and_modify(|metadata| {
                *metadata = GameObjectMetaData::INT(ceiling.get_ceiling_group_id())
            })
            .or_insert(GameObjectMetaData::INT(ceiling.get_ceiling_group_id()));
    }

    let mut frame_number: i32 = 0;
    loop {
        print!("{}\r\n", frame_number);
        frame_number += 1;
        print!("{}", game.player.position);
        print!("\r\n");

        render(&game);

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up => process_input(0, -1, &mut game),
                    KeyCode::Down => process_input(0, 1, &mut game),
                    KeyCode::Left => process_input(-1, 0, &mut game),
                    KeyCode::Right => process_input(1, 0, &mut game),
                    KeyCode::Char('e') => process_input(0, 0, &mut game),
                    KeyCode::Char('q') => {
                        println!("Quitting... \r");
                        break;
                    }
                    _ => {}
                }
            }
        }

        std::thread::sleep(Duration::from_millis(32)); // 30 fps
        clearscreen::clear().expect("failed to clear screen");
    }
    disable_raw_mode()?;
    Ok(())
}

fn generate_audio_manager() -> Result<AudioManager, Box<dyn std::error::Error>> {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    Ok(manager)
}

fn process_input(direction_x: i32, direction_y: i32, game: &mut Game) {
    if game.game_state == GameState::Dialogue {
        let Some(object) = game.map.objects.get_mut(&game.map.current_event_position) else {
            return;
        };
        let Some(event) = object.events.get_mut(object.current_event_index) else {
            return;
        };

        match direction_x {
            -1 => {
                if event.get_current_dialogue_selections_length() == 0 {
                    return;
                }
                event.current_dialogue_selection_index =
                    ((event.current_dialogue_selection_index as i32) - 1).clamp(
                        0,
                        (event.get_current_dialogue_selections_length() as i32) - 1,
                    ) as usize
            }
            0 => {
                if direction_y != 0 {
                    return;
                }

                if event.get_current_dialogue_selections_length() == 0 {
                    game.game_state = GameState::Normal;
                    return;
                }

                let next_dialogue_index = event.get_current_dialogue_selections_pointing_index();

                if next_dialogue_index == -1 {
                    game.game_state = GameState::Normal;
                    return;
                }

                event.current_dialogue_index = next_dialogue_index as usize
            }
            1 => {
                if event.get_current_dialogue_selections_length() == 0 {
                    return;
                }
                event.current_dialogue_selection_index = (event.current_dialogue_selection_index
                    + 1)
                .clamp(0, event.get_current_dialogue_selections_length() - 1)
            }
            _ => {}
        }
        return;
    } else if game.game_state == GameState::Fight {
        let Some(enemy) = game.map.objects.get(&game.map.current_event_position) else {
            return;
        };

        match direction_x {
            -1 => {
                game.map.current_fight_selection_index =
                    ((game.map.current_fight_selection_index as i32) - 1)
                        .clamp(0, (game.map.fight_selections.len() as i32) - 1)
                        as usize
            }
            0 => {
                if direction_y != 0 {
                    return;
                }

                match game.map.current_fight_selection_index {
                    0 => { // Fight
                        
                    }
                    1 => { // Run
                        if game.player.stat.agility <= enemy.stat.agility && rand::random_range(1..101) > 35 {
                            game.map.fight_move_queue = 1;
                            return;
                        }
                        
                        game.game_state = GameState::Normal;
                        return;
                    }
                    _ => {}
                }
            }
            1 => {
                game.map.current_fight_selection_index =
                    ((game.map.current_fight_selection_index as i32) + 1)
                        .clamp(0, (game.map.fight_selections.len() as i32) - 1)
                        as usize
            }
            _ => {}
        }
        return;
    }

    if direction_x == 0
        && direction_y == 0
        && let Some(event) = game
            .map
            .get_if_there_is_event_at_this_position(game.player.position)
    {
        match event.event_type {
            EventType::Dialogue => game.game_state = GameState::Dialogue,
            EventType::Fight => {
                game.game_state = GameState::Fight;
            }
        }
    }

    let next_x = game.player.position.x + direction_x;
    let next_y = game.player.position.y + direction_y;

    if game.map.is_out_of_bounds(Vector2::new(next_x, next_y)) {
        return;
    }

    if let Some(object) = game.map.objects.get(&Vector2::new(next_x, next_y)) {
        match object.object_type {
            GameObjectType::STATIC => return,
            GameObjectType::MOVEABLE => {
                if game
                    .map
                    .push_object(object.position, direction_x, direction_y)
                    == false
                {
                    return;
                }
            }
            GameObjectType::PLAYER => {}
            GameObjectType::CEILING => {}
        }
    }

    game.player.position.x = next_x;
    game.player.position.y = next_y;

    if let Some(ceiling) = game.map.get_ceiling_from_position(game.player.position) {
        game.player
            .meta_data
            .entry("CeilingIDPlayerIsOnBelow".to_string())
            .and_modify(|metadata| {
                *metadata = GameObjectMetaData::INT(ceiling.get_ceiling_group_id())
            })
            .or_insert(GameObjectMetaData::INT(ceiling.get_ceiling_group_id()));
    } else {
        game.player
            .meta_data
            .entry("CeilingIDPlayerIsOnBelow".to_string())
            .and_modify(|metadata| *metadata = GameObjectMetaData::INT(-1))
            .or_insert(GameObjectMetaData::INT(-1));
    }

    let rel_x = game.player.position.x - game.camera.x;
    if direction_x < 0 && rel_x < game.screen_margins.x {
        game.camera.x += direction_x;
    } else if direction_x > 0 && rel_x >= game.screen_size.x - game.screen_margins.x {
        game.camera.x += direction_x;
    }

    let rel_y = game.player.position.y - game.camera.y;
    if direction_y < 0 && rel_y < game.screen_margins.y {
        game.camera.y += direction_y;
    } else if direction_y > 0 && rel_y >= game.screen_size.y - game.screen_margins.y {
        game.camera.y += direction_y;
    }
}

fn render(game: &Game) {
    let mut dialogue_rendered: bool = false;
    let mut dialogue_selection_rendered: bool = false;
    let mut rendered_fight_selections: bool = false;

    for y in 0..game.screen_size.y {
        for x in 0..game.screen_size.x {
            match game.game_state {
                GameState::Normal => {}
                GameState::Fight => {
                    if y == game.screen_size.y / 2 {
                        if x == game.screen_margins.x {
                            print!("{}", game.player.icon);
                        } else if x == game.screen_size.x - game.screen_margins.x {
                            if let Some(enemy) =
                                game.map.objects.get(&game.map.current_event_position)
                            {
                                print!("{}", enemy.icon);
                            }
                        } else {
                            print!(" ");
                        }
                    } else if y == game.screen_size.y - 5 {
                        print_fight_padding();
                    } else if y == game.screen_size.y - 4 && !rendered_fight_selections {
                        let selections = &game.map.fight_selections;

                        let output: String = selections
                            .iter()
                            .enumerate()
                            .map(|(i, text)| {
                                if i == game.map.current_fight_selection_index {
                                    text.custom_color(CustomColor::new(255, 0, 0)).to_string()
                                } else {
                                    text.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        print!("{}", output);

                        rendered_fight_selections = true;
                    } else {
                        print!(" ");
                    }
                    continue;
                }
                GameState::Cutscene => {}
                GameState::Dialogue => {}
            }

            let current_point = get_point_from_world_to_screen(&game.camera, &Vector2::new(x, y));
            if current_point.x > game.map.map_size.x
                || current_point.x < 0
                || current_point.y > game.map.map_size.y
                || current_point.y < 0
            {
                print!(" ");
                continue;
            }

            if game.player.position == current_point {
                print!("{}", game.player.icon);
            }
            // render the ceilings first
            else if let Some(ceiling) = game.map.get_ceiling_from_position(current_point)
                && !matches!(game.player.meta_data.get("CeilingIDPlayerIsOnBelow"), Some(GameObjectMetaData::INT(id)) if *id == ceiling.get_ceiling_group_id())
            {
                print!("{}", ceiling.icon);
            } else if let Some(object) = game.map.objects.get(&current_point) {
                print!("{}", object.icon);
            } else {
                print!("{}", game.map.ground_icon);
            }
        }

        match game.game_state {
            GameState::Normal => {}
            GameState::Fight => {}
            GameState::Cutscene => {}
            GameState::Dialogue => 'dialogue: {
                print_dialogue_padding();
                let Some(object) = game.map.objects.get(&game.map.current_event_position) else {
                    break 'dialogue;
                };

                let Some(event) = object.events.get(object.current_event_index) else {
                    break 'dialogue;
                };

                if !dialogue_rendered {
                    print!("{}", event.get_current_dialogue_text());
                    dialogue_rendered = true;
                    break 'dialogue;
                }

                if !dialogue_selection_rendered {
                    let selections = event.get_current_dialogue_selections();

                    let output: String = selections
                        .iter()
                        .enumerate()
                        .map(|(i, text)| {
                            if i == event.current_dialogue_selection_index {
                                text.custom_color(CustomColor::new(255, 0, 0)).to_string()
                            } else {
                                text.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    print!("{}", output);
                    dialogue_selection_rendered = true;
                }
            }
        }
        print!("\r\n");
    }
}

fn print_dialogue_padding() {
    print!("{}", " ".repeat(5));
    print!("|   ");
}

fn print_fight_padding() {
    print!("{}", "_");
}

fn get_point_from_world_to_screen(game_origin: &Vector2, screen_coordinate: &Vector2) -> Vector2 {
    return game_origin + screen_coordinate;
}
