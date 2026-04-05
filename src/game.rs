use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute, terminal};
use rand;
use std::io::{Write, stdout};
use std::panic;
use std::path::Path;
use std::{env, i32, io};

use crate::game_object::{
    COMBAT_SELECTIONS, CombatPhase, EnemyAttack, EventCondition, GameEvent, GameObjectID,
    Projectile, TurnResult,
};
use crate::level::{data_to_map, load_map, load_measurements};
use crate::map::*;
use crate::renderer::{Renderer, ScreenMeasurements};
use crate::vector2::*;

use colored::control;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Normal,
    Dialogue,
    Combat,
    Menu,
}

pub struct Game {
    pub map: Map,
    pub camera: Vector2,
    pub audio_manager: AudioManager,
    pub state: GameState,
    pub renderer: Renderer,
}

impl Game {
    pub fn new() -> Self {
        Game {
            map: Map::new(
                Vector2::new(100, 100),
                "#".custom_color(CustomColor::new(0, 255, 0)),
            ),
            camera: Vector2::zero(),
            audio_manager: generate_audio_manager().expect("Failed to initialize audio"),
            state: GameState::Normal,
            renderer: Renderer::new(ScreenMeasurements::new(
                Vector2::new(50, 20),
                Vector2::new(5, 3),
                5,
                2,
                2,
                50,
                7,
                9,
                30,
                5,
                1,
                3,
            )),
        }
    }

    pub fn setup_renderer(&mut self) {
        if let Some(measurements) = load_measurements(&"./measurements.ron") {
            self.renderer = Renderer::new(measurements);
        }
    }
    pub fn setup_map(&mut self) {
        if let Some(map) = load_map(&"./map.ron") {
            self.map = data_to_map(&map);
        }
    }

    pub fn process_input(&mut self, key: KeyCode) -> bool {
        if key == KeyCode::Char('q') {
            println!("Quitting... \r");
            return false;
        }
        match &self.state {
            GameState::Normal => match key {
                KeyCode::Up => self.move_objects(Vector2::new(0, -1)),
                KeyCode::Down => self.move_objects(Vector2::new(0, 1)),
                KeyCode::Left => self.move_objects(Vector2::new(-1, 0)),
                KeyCode::Right => self.move_objects(Vector2::new(1, 0)),
                KeyCode::Char('e') => {
                    let _ = self.trigger_event_nearby();
                }
                _ => {}
            },
            GameState::Dialogue => match key {
                KeyCode::Up => {
                    let _ = self.change_dialogue_selection(-1);
                }
                KeyCode::Down => {
                    let _ = self.change_dialogue_selection(1);
                }
                KeyCode::Char('e') => {
                    let _ = self.progress_event();
                }
                _ => {}
            },
            GameState::Combat => match key {
                KeyCode::Left => {
                    let _ = self.change_combat_selection(-1);
                }
                KeyCode::Right => {
                    let _ = self.change_combat_selection(1);
                }
                KeyCode::Up => {
                    let _ = self.combat_movement(-1);
                }
                KeyCode::Down => {
                    let _ = self.combat_movement(1);
                }
                KeyCode::Char('e') => {
                    let _ = self.select_combat_choice();
                }
                _ => {}
            },
            _ => {}
        }
        return true;
    }

    fn change_dialogue_selection(&mut self, direction: i32) -> Option<()> {
        let event_id = self.map.current_event_id?;
        let event = self.map.event_components.get_mut(&event_id)?;

        if let GameEvent::Dialogue(ref mut dialogue) = event.events[event.current_index].event {
            let len = dialogue.selections.len() as i32;
            if len > 0 {
                let new_index = (dialogue.current_selection as i32 + direction + len) % len;
                dialogue.current_selection = new_index as usize;
            }
        }

        return Some(());
    }
    fn change_combat_selection(&mut self, direction: i32) -> Option<()> {
        let event_id = self.map.current_event_id?;
        let event = self.map.event_components.get_mut(&event_id)?;

        if let GameEvent::Combat(ref mut combat) = event.events[event.current_index].event {
            let len = COMBAT_SELECTIONS.len() as i32;
            if len > 0 {
                let new_index = (combat.current_selection as i32 + direction + len) % len;
                combat.current_selection = new_index as usize;
            }
        }

        return Some(());
    }
    fn combat_movement(&mut self, direction: i32) -> Option<()> {
        let event_id = self.map.current_event_id?;
        let event = self.map.event_components.get_mut(&event_id)?;

        if let GameEvent::Combat(ref mut combat) = event.events[event.current_index].event {
            if matches!(combat.current_phase, CombatPhase::EnemyAttack(_)) {
                combat.player_row = (combat.player_row as i32 + direction).clamp(0, 2) as usize;
            }
        }
        Some(())
    }
    fn select_combat_choice(&mut self) -> Option<()> {
        let event_id = self.map.current_event_id?;
        let event = self.map.event_components.get_mut(&event_id)?;

        if let GameEvent::Combat(ref mut combat) = event.events[event.current_index].event {
            if !matches!(combat.current_phase, CombatPhase::PlayerTurn) {
                return Some(());
            }
            let player_stats = self.map.stats_components.get(&self.map.camera_operator)?;
            let enemy_stats = self.map.stats_components.get(&event_id)?;
            match COMBAT_SELECTIONS[combat.current_selection] {
                "Fight" => {
                    let damage = player_stats.calculate_damage(enemy_stats);

                    self.map
                        .stats_components
                        .get_mut(&event_id)?
                        .take_damage(damage); // ehhh fuck it

                    combat.current_phase = CombatPhase::TurnResult(TurnResult::WasPlayersTurn);
                    self.renderer.combat_message = format!("You dealt {} damage.", damage);
                }
                "Item" => {}
                "Run" => {
                    if player_stats.agility > enemy_stats.agility {
                        self.state = GameState::Normal;
                    } else if player_stats.agility <= enemy_stats.agility
                        && rand::random::<f32>() < 0.30
                    {
                        self.state = GameState::Normal;
                    } else {
                        self.renderer.combat_message = String::from("Can't run away.");
                        combat.current_phase = CombatPhase::TurnResult(TurnResult::WasPlayersTurn);
                    }
                }
                _ => {}
            }
        }

        return Some(());
    }

    fn progress_event(&mut self) -> Option<()> {
        let event_id = self.map.current_event_id?;
        let event = self.map.event_components.get_mut(&event_id)?;

        match event.events[event.current_index].requirement {
            EventCondition::None => 'none: {
                match &event.events[event.current_index].event {
                    GameEvent::Dialogue(dialogue) => {
                        self.state = GameState::Normal;
                        if dialogue.selections_pointing_event.is_empty() {
                            let Some(next_index) = event.events[event.current_index].next_event
                            else {
                                self.map.current_event_id = None;
                                break 'none;
                            };
                            event.current_index = next_index;
                            self.trigger_event(event_id);
                            break 'none;
                        }

                        let Some(next_index) =
                            dialogue.selections_pointing_event[dialogue.current_selection]
                        else {
                            let Some(next_index) = event.events[event.current_index].next_event
                            else {
                                self.map.current_event_id = None;
                                break 'none;
                            };
                            event.current_index = next_index;
                            self.trigger_event(event_id);
                            break 'none;
                        };
                        event.current_index = next_index;
                        self.trigger_event(event_id);
                    }
                    GameEvent::Combat(combat) => {
                        if combat.delete_when_defeated {
                            self.map.delete_object(event_id);
                            self.state = GameState::Normal;
                            self.map.current_event_id = None;
                            break 'none;
                        }
                        let Some(next_index) = event.events[event.current_index].next_event else {
                            self.state = GameState::Normal;
                            self.map.current_event_id = None;
                            break 'none;
                        };
                        self.map.stats_components.get_mut(&event_id)?.heal_to_max();
                        event.current_index = next_index;
                        self.trigger_event(event_id);
                    }
                    GameEvent::TriggerObjectEvent(_) => {}
                    GameEvent::None => {}
                }
            }
        }

        return Some(());
    }

    fn trigger_event_nearby(&mut self) -> Option<()> {
        let Some(camera_object) = self.map.objects.get(&self.map.camera_operator) else {
            return None;
        };
        let pos = camera_object.position;
        let id = self.map.get_event_around_this_position(pos)?;
        self.trigger_event(id);
        Some(())
    }

    fn trigger_event(&mut self, id: GameObjectID) -> Option<()> {
        let event = self.map.event_components.get_mut(&id)?;

        if event.events[event.current_index].is_triggered
            && !event.events[event.current_index].repeat
        {
            return Some(());
        }

        match &event.events[event.current_index].event {
            GameEvent::Dialogue(_) => {
                self.state = GameState::Dialogue;
                self.map.current_event_id = Some(id);
            }
            GameEvent::Combat(combat) => {
                self.state = GameState::Combat;
                self.map.current_event_id = Some(id);

                if !combat.turn_order_decided {
                    self.decide_turn_order();
                }
            }
            GameEvent::TriggerObjectEvent(_) => {}
            GameEvent::None => {}
        }

        if let Some(event) = self.map.event_components.get_mut(&id) {
            event.events[event.current_index].is_triggered = true;
        }

        Some(())
    }

    fn decide_turn_order(&mut self) {
        let Some(event_id) = self.map.current_event_id else {
            return;
        };
        let player_ag = self
            .map
            .stats_components
            .get(&self.map.camera_operator)
            .map(|s| s.agility)
            .unwrap_or(1);
        let enemy_ag = self
            .map
            .stats_components
            .get(&event_id)
            .map(|s| s.agility)
            .unwrap_or(1);

        let player_goes_first = player_ag > enemy_ag || rand::random::<f32>() < 0.30;

        let Some(event) = self.map.event_components.get_mut(&event_id) else {
            return;
        };
        let GameEvent::Combat(ref mut combat) = event.events[event.current_index].event else {
            return;
        };
        combat.player_goes_first = player_goes_first;
        combat.turn_order_decided = true;

        if !player_goes_first {
            self.renderer.combat_message = String::from("Enemy acts");
            combat.current_phase = CombatPhase::TurnResult(TurnResult::WasPlayersTurn);
        } else {
            self.renderer.combat_message = String::from("You act");
            combat.current_phase = CombatPhase::TurnResult(TurnResult::WasEnemiesTurn);
        }
    }

    fn move_objects(&mut self, direction: Vector2) {
        let ids: Vec<usize> = self.map.input_components.keys().cloned().collect();

        for id in ids {
            let Some(object) = self.map.objects.get(&id) else {
                continue;
            };
            let next_position: Vector2 = object.position + direction;

            if self.map.is_out_of_bounds(next_position) {
                continue;
            }

            if let Some(moveable_id) = self.map.positions_hashmap.get(&next_position).cloned()
                && self.map.moveable_components.contains_key(&moveable_id)
            {
                if self
                    .map
                    .change_object_position(moveable_id, direction + next_position)
                {
                    self.map.change_object_position(id, next_position);
                }
            } else {
                self.map.change_object_position(id, next_position);
            }
        }

        let Some(camera_object) = self.map.objects.get(&self.map.camera_operator) else {
            return;
        };
        let pos = camera_object.position;

        let rel_x = pos.x - self.camera.x;
        if direction.x < 0 && rel_x < self.renderer.measurements.screen_margins.x {
            self.camera.x += direction.x;
        } else if direction.x > 0
            && rel_x
                >= self.renderer.measurements.screen_size.x
                    - self.renderer.measurements.screen_margins.x
        {
            self.camera.x += direction.x;
        }

        let rel_y = pos.y - self.camera.y;
        if direction.y < 0 && rel_y < self.renderer.measurements.screen_margins.y {
            self.camera.y += direction.y;
        } else if direction.y > 0
            && rel_y
                >= self.renderer.measurements.screen_size.y
                    - self.renderer.measurements.screen_margins.y
        {
            self.camera.y += direction.y;
        }
    }

    pub fn tick(&mut self, delta_ms: usize) {
        let Some(event_id) = self.map.current_event_id else {
            return;
        };
        let Some(event) = self.map.event_components.get_mut(&event_id) else {
            return;
        };
        match &mut event.events[event.current_index].event {
            GameEvent::Combat(combat) => match &mut combat.current_phase {
                CombatPhase::TurnResult(turn_result) => {
                    combat.turn_result_timer += delta_ms;
                    if combat.turn_result_timer >= combat.turn_result_time {
                        if matches!(turn_result, TurnResult::CombatEnded) {
                            self.progress_event();
                            return;
                        }

                        let Some(enemy_stats) = self.map.stats_components.get(&event_id) else {
                            return;
                        };
                        let Some(player_stats) =
                            self.map.stats_components.get(&self.map.camera_operator)
                        else {
                            return;
                        };

                        self.renderer.combat_message = String::from("");
                        if enemy_stats.is_dead() {
                            self.renderer.combat_message = String::from("Enemy lost.");
                            combat.current_phase = CombatPhase::TurnResult(TurnResult::CombatEnded);
                        } else if player_stats.is_dead() {
                            self.renderer.combat_message = String::from("You lost.");
                            combat.current_phase = CombatPhase::TurnResult(TurnResult::CombatEnded);
                        } else {
                            match turn_result {
                                TurnResult::WasPlayersTurn => {
                                    combat.current_phase =
                                        CombatPhase::EnemyAttack(EnemyAttack::new(combat));
                                }
                                TurnResult::WasEnemiesTurn => {
                                    combat.current_phase = CombatPhase::PlayerTurn;
                                }
                                _ => {}
                            }
                        }
                        combat.turn_result_timer = 0
                    }
                }
                CombatPhase::EnemyAttack(enemy_attack) => {
                    enemy_attack.move_timer += delta_ms;
                    enemy_attack.next_spawn_timer += delta_ms;

                    if enemy_attack.next_spawn_timer >= combat.projectile_spawn_time
                        && enemy_attack.projectile_count > 0
                    {
                        enemy_attack.projectiles.push(Projectile {
                            x: 0,
                            row: rand::random_range(0..3),
                            damage: combat.projectile_damage,
                        });
                        enemy_attack.next_spawn_timer = 0;
                        enemy_attack.projectile_count -= 1;
                    }
                    if enemy_attack.move_timer >= combat.projectile_move_time {
                        for projectile in &mut enemy_attack.projectiles {
                            projectile.x += 1;
                        }

                        let base = self.renderer.measurements.combat_character_padding_x
                            + self.renderer.measurements.combat_characters_distance
                            + 1;
                        enemy_attack.projectiles.retain(|projectile| {
                            if projectile.row == combat.player_row
                                && projectile.x
                                    == self.renderer.measurements.combat_characters_distance + 1
                            {
                                enemy_attack.damage_dealt += projectile.damage;
                                return false;
                            }
                            if projectile.x >= base {
                                return false;
                            }
                            true
                        });

                        enemy_attack.move_timer = 0;
                    }

                    if enemy_attack.projectile_count == 0 && enemy_attack.projectiles.is_empty() {
                        if enemy_attack.damage_dealt > 0 {
                            if let Some(stats) =
                                self.map.stats_components.get_mut(&self.map.camera_operator)
                            {
                                stats.take_damage(enemy_attack.damage_dealt);
                            }
                            self.renderer.combat_message =
                                format!("Enemy dealt {} damage.", enemy_attack.damage_dealt);
                        } else {
                            self.renderer.combat_message = format!("Took no damage.",);
                        }
                        combat.current_phase = CombatPhase::TurnResult(TurnResult::WasEnemiesTurn);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn generate_audio_manager() -> Result<AudioManager, Box<dyn std::error::Error>> {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    Ok(manager)
}

fn handle_crash(info: &panic::PanicHookInfo) {
    let mut stdout = stdout();
    let _ = execute!(stdout, cursor::MoveTo(0, 0));
    let _ = stdout.flush();
    let _ = execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show);
    let _ = disable_raw_mode();
    println!("Something went wrong");
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("Cause: {}", s);
    }
}

pub fn run() -> io::Result<()> {
    panic::set_hook(Box::new(|info| {
        handle_crash(info);
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    control::set_override(true);

    let mut game = Game::new();
    if cfg!(debug_assertions) {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let folder_string = &args[1];
            let folder_path = Path::new(folder_string);
            if folder_path.is_dir() {
                let measurements_path = folder_path.join("measurements.ron");
                let map_path = folder_path.join("map.ron");
                if let Some(m_str) = measurements_path.to_str() {
                    if let Some(measurements) = load_measurements(m_str) {
                        game.renderer = Renderer::new(measurements);
                    }
                }
                if let Some(map_str) = map_path.to_str() {
                    if let Some(map) = load_map(map_str) {
                        game.map = data_to_map(&map);
                    }
                }
            }
        };
    } else {
        game.setup_renderer();
        game.setup_map();
    }

    let mut frame_number: i32 = 0;
    let mut last_frame = Instant::now();

    loop {
        let delta_ms = last_frame.elapsed().as_millis() as usize;
        last_frame = Instant::now();

        execute!(stdout, cursor::MoveTo(0, 0))?;

        if cfg!(debug_assertions) {
            print!("{}\r\n", frame_number);
        }
        frame_number += 1;

        game.renderer.render(&game.map, &game.camera, &game.state);

        stdout.flush()?;

        game.tick(delta_ms);

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(key_event) = event::read()?
        {
            if game.process_input(key_event.code) == false && key_event.kind == KeyEventKind::Press
            {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(32)); // 30 fps
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
