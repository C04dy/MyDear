use crate::{
    game_object::{
        Combat, CombatPhase, Dialogue, EventComponent, EventCondition, EventStep, GameEvent,
        GameObjectID, InputComponent, MoveableComponent, StatsComponent,
    },
    map::Map,
    renderer::ScreenMeasurements,
    vector2::Vector2,
};
use colored::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

// Temporary objects for loading, will not live in the runtime
#[derive(Serialize, Deserialize)]
pub struct MapData {
    pub map_size: Vector2,
    pub objects: HashMap<GameObjectID, ObjectData>,
    pub ground_icon: String,
    pub ground_color: ColorData,
    pub moveable_components: HashMap<GameObjectID, MoveableComponent>,
    pub input_components: HashMap<GameObjectID, InputComponent>,
    pub event_components: HashMap<GameObjectID, EventComponentData>,
    pub stats_components: HashMap<GameObjectID, StatsComponent>,
    pub camera_operator: GameObjectID,
}
#[derive(Serialize, Deserialize)]
pub struct ObjectData {
    pub position: Vector2,
    pub icon: String,
    pub icon_color: ColorData,
}
#[derive(Serialize, Deserialize)]
pub struct EventComponentData {
    pub events: Vec<EventStepData>,
    pub current_index: usize,
}
#[derive(Serialize, Deserialize)]
pub struct EventStepData {
    pub event: GameEventData,
    pub requirement: EventCondition,
    pub repeat: bool,
    pub is_triggered: bool,
    pub next_event: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub enum GameEventData {
    None,
    Dialogue(DialogueData),
    Combat(CombatData),
    TriggerObjectEvent(GameObjectID),
}

#[derive(Serialize, Deserialize)]
pub struct DialogueData {
    pub text: String,
    pub selections: Vec<String>,
    pub selections_pointing_event: Vec<Option<usize>>,
    pub current_selection: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CombatData {
    pub player_goes_first: bool,
    pub turn_order_decided: bool,
    pub turn_result_time: usize,
    pub projectile_icon: String,
    pub projectile_icon_color: ColorData,
    pub projectile_damage: usize,
    pub projectile_count: usize,
    pub projectile_move_time: usize,
    pub projectile_spawn_time: usize,
}
#[derive(Serialize, Deserialize)]
pub struct ColorData {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RecentProjects {
    pub paths: Vec<String>,
}

pub fn get_data_path() -> Option<std::path::PathBuf> {
    ProjectDirs::from("com", "YourName", "MyDear").map(|dirs| {
        let path = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&path).ok();
        path
    })
}

pub fn load_recent_projects() -> RecentProjects {
    let Some(dir) = get_data_path() else {
        return RecentProjects::default();
    };
    let path = dir.join("recent_projects.ron");
    let Ok(s) = std::fs::read_to_string(path) else {
        return RecentProjects::default();
    };
    ron::from_str(&s).unwrap_or_default()
}

pub fn save_recent_projects(data: &RecentProjects) -> std::io::Result<()> {
    let Some(dir) = get_data_path() else {
        return Ok(());
    };
    let s = ron::to_string(data).unwrap();
    std::fs::write(dir.join("recent_projects.ron"), s)
}

pub fn add_recent_project(path: &str) -> RecentProjects {
    let mut recent = load_recent_projects();
    recent.paths.retain(|p| p != path);
    recent.paths.insert(0, path.to_string());
    let _ = save_recent_projects(&recent);
    return recent;
}

pub fn remove_recent_project(path: &str) -> RecentProjects {
    let mut recent = load_recent_projects();
    recent.paths.retain(|p| p != path);
    let _ = save_recent_projects(&recent);
    return recent;
}

fn data_to_color(data: &ColorData) -> CustomColor {
    CustomColor::new(data.r, data.g, data.b)
}

pub fn save_map(data: &MapData, path: &str, map_name: &str) -> std::io::Result<()> {
    match ron::to_string(data) {
        Ok(str) => std::fs::write(Path::new(path).join(map_name), str),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}

pub fn save_measurements(data: &ScreenMeasurements, path: &str) -> std::io::Result<()> {
    match ron::to_string(data) {
        Ok(str) => std::fs::write(Path::new(path).join("measurements.ron"), str),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}
pub fn load_map(path: &str) -> Option<MapData> {
    match std::fs::read_to_string(path) {
        Ok(s) => Some(ron::from_str(&s).unwrap()),
        Err(_) => None,
    }
}
pub fn load_measurements(path: &str) -> Option<ScreenMeasurements> {
    match std::fs::read_to_string(path) {
        Ok(s) => Some(ron::from_str(&s).unwrap()),
        Err(_) => None,
    }
}

/// Converts MapData to Map IGNORE THIS FUNCTION
pub fn data_to_map(data: &MapData) -> Map {
    let mut map = Map::new(
        data.map_size,
        data.ground_icon
            .custom_color(data_to_color(&data.ground_color)),
    );
    map.camera_operator = data.camera_operator;
    data.objects.iter().for_each(|(&id, obj)| {
        map.insert_object_with_id(
            id,
            obj.position,
            obj.icon.custom_color(data_to_color(&obj.icon_color)),
        );
    });
    data.moveable_components.keys().for_each(|&id| {
        map.insert_moveable_component(id);
    });
    data.input_components.keys().for_each(|&id| {
        map.insert_input_component(id);
    });
    data.stats_components.iter().for_each(|(&id, s)| {
        map.insert_stats_component(id, s.clone());
    });
    data.event_components.iter().for_each(|(&id, ec)| {
        let events = ec
            .events
            .iter()
            .map(|step| EventStep {
                event: match &step.event {
                    GameEventData::None => GameEvent::None,
                    GameEventData::Dialogue(d) => GameEvent::Dialogue(Dialogue::new(
                        d.text.clone(),
                        d.selections.clone(),
                        d.selections_pointing_event.clone(),
                        d.current_selection,
                    )),
                    GameEventData::Combat(c) => GameEvent::Combat(Combat::new(
                        CombatPhase::PlayerTurn,
                        c.player_goes_first,
                        c.turn_order_decided,
                        c.turn_result_time,
                        c.projectile_icon
                            .custom_color(data_to_color(&c.projectile_icon_color)),
                        c.projectile_damage,
                        c.projectile_count,
                        c.projectile_move_time,
                        c.projectile_spawn_time,
                        false,
                    )),
                    GameEventData::TriggerObjectEvent(id) => GameEvent::TriggerObjectEvent(*id),
                },
                requirement: step.requirement.clone(),
                repeat: step.repeat,
                is_triggered: step.is_triggered,
                next_event: step.next_event,
            })
            .collect();

        let mut ec_runtime = EventComponent::new(events);
        ec_runtime.current_index = ec.current_index;
        map.event_components.insert(id, ec_runtime);
    });

    return map;
}
/// Converts Map to MapData
pub fn map_to_data(map: &Map) -> MapData {
    let objects = map
        .objects
        .iter()
        .map(|(&id, obj)| {
            let color = match &obj.icon.fgcolor {
                Some(Color::TrueColor { r, g, b }) => ColorData {
                    r: *r,
                    g: *g,
                    b: *b,
                },
                _ => ColorData {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            };
            (
                id,
                ObjectData {
                    position: obj.position,
                    icon: obj.icon.input.clone(),
                    icon_color: color,
                },
            )
        })
        .collect();

    let event_components = map
        .event_components
        .iter()
        .map(|(&id, ec)| {
            (
                id,
                EventComponentData {
                    events: ec
                        .events
                        .iter()
                        .map(|step| EventStepData {
                            requirement: step.requirement.clone(),
                            repeat: step.repeat,
                            is_triggered: step.is_triggered,
                            next_event: step.next_event,
                            event: match &step.event {
                                GameEvent::Dialogue(d) => GameEventData::Dialogue(DialogueData {
                                    text: d.text.clone(),
                                    selections: d.selections.clone(),
                                    selections_pointing_event: d.selections_pointing_event.clone(),
                                    current_selection: d.current_selection,
                                }),
                                GameEvent::Combat(c) => GameEventData::Combat(CombatData {
                                    player_goes_first: c.player_goes_first,
                                    turn_order_decided: c.turn_order_decided,
                                    turn_result_time: c.turn_result_time,
                                    projectile_damage: c.projectile_damage,
                                    projectile_count: c.projectile_count,
                                    projectile_move_time: c.projectile_move_time,
                                    projectile_spawn_time: c.projectile_spawn_time,
                                    projectile_icon: c.projectile_icon.input.clone(),
                                    projectile_icon_color: match &c.projectile_icon.fgcolor {
                                        Some(Color::TrueColor { r, g, b }) => ColorData {
                                            r: *r,
                                            g: *g,
                                            b: *b,
                                        },
                                        _ => ColorData {
                                            r: 255,
                                            g: 255,
                                            b: 255,
                                        },
                                    },
                                }),
                                GameEvent::TriggerObjectEvent(id) => {
                                    GameEventData::TriggerObjectEvent(*id)
                                }
                                GameEvent::None => GameEventData::None,
                            },
                        })
                        .collect(),
                    current_index: ec.current_index,
                },
            )
        })
        .collect();

    let ground_color = match &map.ground_icon.fgcolor {
        Some(Color::TrueColor { r, g, b }) => ColorData {
            r: *r,
            g: *g,
            b: *b,
        },
        _ => ColorData {
            r: 255,
            g: 255,
            b: 255,
        },
    };

    MapData {
        map_size: map.map_size,
        objects,
        ground_icon: map.ground_icon.input.clone(),
        ground_color,
        moveable_components: map.moveable_components.clone(),
        input_components: map.input_components.clone(),
        event_components,
        stats_components: map.stats_components.clone(),
        camera_operator: map.camera_operator,
    }
}
