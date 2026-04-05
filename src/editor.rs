use crate::{
    game_object::{EventCondition, EventStep, GameEvent, GameObjectID, StatsComponent},
    level::{
        add_recent_project, data_to_map, load_map, load_measurements, load_recent_projects,
        map_to_data, remove_recent_project, save_map, save_measurements,
    },
    map::Map,
    renderer::{Renderer, ScreenMeasurements},
    vector2::Vector2,
};
use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{self, Write, stdout},
    path::Path,
};
use std::{panic, time::Duration};

pub const OBJECT_EDIT_SELECTIONS: &[&str] =
    &["Position", "Icon", "Color", "Components", "Camera Operator"];
pub const FILE_SELECTIONS: &[&str] = &["New Project", "Open Project", "Recent Projects"];
pub const COMPONENT_SELECTIONS: &[&str] = &[
    "MoveableComponent",
    "InputComponent",
    "EventComponent",
    "StatsComponent",
];
pub const STATS_COMPONENT_SELECTIONS: &[&str] =
    &["strength", "agility", "defense", "luck", "max_health"];
pub const EDIT_SCREEN_MEASUREMENTS_SELECTIONS: &[&str] = &[
    "screen_size",
    "screen_margins",
    "dialogue_padding",
    "dialogue_text_padding",
    "dialogue_selection_text_padding",
    "dialogue_max_character_count",
    "combat_character_padding_y",
    "combat_character_padding_x",
    "combat_characters_distance",
    "combat_separator_padding_y",
    "combat_selection_separator_padding",
    "combat_health_padding_y",
];

pub const BROSWING_MESSAGE: &str =
    "←↑→↓:Move, e:Insert/Edit object, s:Save, CTRL+q:Quit, m:Edit screen measurements";
pub const EDITING_OBJECT_MESSAGE: &str =
    "↑↓:Move selection, ENTER:Select/DeSelect property, DELETE:Delete Object, ESC:Go back";
pub const EDITING_COMPONENT_MESSAGE: &str =
    "↑↓:Move selection, ENTER:Add component, DELETE:Remove component, ESC:Go back";
pub const EDITING_STATS_COMPONENT_MESSAGE: &str = "↑↓:Move selection, ←→:Change value, ESC:Go back";
pub const EDITING_EVENT_COMPONENT_MESSAGE: &str = "↑↓:Move selection, ←→:Change value, ENTER:Edit event, +:Add EventStep, DELETE:Delete EventStep, ESC:Go back";
pub const EDITING_EVENT_MESSAGE: &str =
    "↑↓:Move selection, ←→:Change value, ENTER:Edit, ESC:Go back";
pub const EDITING_EVENT_MESSAGE_DIALOGUE: &str = "↑↓:Move selection, ←→:Change value, ENTER:Edit, +:Add selection, -:Remove selection, ESC:Go back";
pub const EDITING_MEASUREMENTS_MESSAGE: &str =
    "↑↓:Move selection, ←→:Change value, ENTER:Select/DeSelect property, ESC:Go back";

pub enum EditorState {
    SelectingFile {
        file_selection: usize,
        file_input: String,
        file_message: String,
        recent_projects: Vec<String>,
        recent_selection: usize,
    },
    Browsing {
        cursor: Vector2,
    },
    EditingMeasurements {
        selection: usize,
        selections_selection: usize,
        selected: bool,
    },
    EditingObject {
        object_id: GameObjectID,
        selection: usize,
        edit_selection: usize,
        selected: bool,
    },
    SelectingComponent {
        object_id: GameObjectID,
        selection: usize,
    },
    EditingEventComponent {
        object_id: GameObjectID,
        current_step: usize,
        selection: usize,
    },
    EditingEvent {
        object_id: GameObjectID,
        current_step: usize,
        selection: usize,
        editing_selection: bool,
        selections_selection: usize,
    },
    EditingStatsComponent {
        object_id: GameObjectID,
        selection: usize,
    },
}

pub struct Editor {
    pub map: Map,
    pub camera: Vector2,
    pub renderer: Renderer,
    pub state: EditorState,
    current_folder: String,
    current_map: String,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            map: Map::new(
                Vector2::new(500, 500),
                "#".custom_color(CustomColor::new(0, 255, 0)),
            ),
            camera: Vector2::zero(),
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
            state: EditorState::SelectingFile {
                file_selection: 0,
                file_input: "".to_string(),
                file_message: "".to_string(),
                recent_projects: load_recent_projects().paths,
                recent_selection: 0,
            },
            current_folder: "".to_string(),
            current_map: "".to_string(),
        }
    }

    fn save(&self) -> std::io::Result<()> {
        save_map(
            &map_to_data(&self.map),
            &self.current_folder,
            &self.current_map,
        )?;
        save_measurements(&self.renderer.measurements, &self.current_folder)?;
        Ok(())
    }

    fn open_project(&mut self, path: &str) -> bool {
        let folder = if path.ends_with('/') {
            path.to_string()
        } else {
            path.to_string() + "/"
        };

        let map_path = folder.clone() + "map.ron";
        let measurements_path = folder.clone() + "measurements.ron";

        if !Path::new(&map_path).is_file() {
            return false;
        }
        if !Path::new(&measurements_path).is_file() {
            return false;
        }

        if let Some(measurements) = load_measurements(&measurements_path) {
            self.renderer = Renderer::new(measurements);
        }
        if let Some(map) = load_map(&map_path) {
            self.map = data_to_map(&map);
        }

        self.current_folder = folder;
        self.current_map = String::from("map.ron");
        add_recent_project(path);
        self.renderer.set_editor_message(BROSWING_MESSAGE);
        self.state = EditorState::Browsing {
            cursor: Vector2::new(
                self.renderer.measurements.screen_size.x / 2,
                self.renderer.measurements.screen_size.y / 2,
            ),
        };
        return true;
    }

    pub fn process_input(&mut self, key: KeyEvent) -> bool {
        if let EditorState::SelectingFile {
            file_selection,
            file_input,
            recent_projects,
            recent_selection,
            ..
        } = &self.state
        {
            if key.code == KeyCode::Enter {
                if file_input == "" && FILE_SELECTIONS[*file_selection] != "Recent Projects" {
                    return true;
                }
                let selection = FILE_SELECTIONS[*file_selection];
                let input = file_input.clone();
                let recent = recent_projects.get(*recent_selection).cloned();
                // all borrows of self.state dropped here
                match selection {
                    "New Project" => {
                        let path = Path::new(input.as_str());
                        let can_create = if path.exists() {
                            path.read_dir()
                                .map(|mut d| d.next().is_none())
                                .unwrap_or(false)
                        } else {
                            std::fs::create_dir_all(path).is_ok()
                        };

                        if can_create {
                            self.current_folder = input.clone() + "/";
                            self.current_map = String::from("map.ron");
                            match self.save() {
                                Ok(_) => {}
                                Err(e) => {
                                    if let EditorState::SelectingFile { file_message, .. } =
                                        &mut self.state
                                    {
                                        *file_message = format!("Couldnt Create Project: {}", e);
                                    }
                                    return true;
                                }
                            }

                            add_recent_project(&input);
                            self.renderer.set_editor_message(BROSWING_MESSAGE);
                            self.state = EditorState::Browsing {
                                cursor: Vector2::new(
                                    self.renderer.measurements.screen_size.x / 2,
                                    self.renderer.measurements.screen_size.y / 2,
                                ),
                            };
                        } else {
                            if let EditorState::SelectingFile { file_message, .. } = &mut self.state
                            {
                                *file_message = format!("Cannot create project at {}", input);
                            }
                        }
                    }
                    "Open Project" => {
                        if !self.open_project(&input) {
                            if let EditorState::SelectingFile { file_message, .. } = &mut self.state
                            {
                                *file_message = format!("Filepath {} is not valid", input);
                            }
                        }
                    }
                    "Recent Projects" => {
                        if let Some(path) = recent {
                            if !self.open_project(&path) {
                                if let EditorState::SelectingFile { file_message, .. } =
                                    &mut self.state
                                {
                                    *file_message = format!("Project at {} no longer exists", path);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                return true;
            }
        }

        match &mut self.state {
            EditorState::SelectingFile {
                file_selection,
                file_input,
                file_message,
                recent_projects,
                recent_selection,
            } => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                    return false;
                }
                match key.code {
                    KeyCode::Left => {
                        *file_selection =
                            (*file_selection + FILE_SELECTIONS.len() - 1) % FILE_SELECTIONS.len();
                    }
                    KeyCode::Right => {
                        *file_selection = (*file_selection + 1) % FILE_SELECTIONS.len();
                    }
                    KeyCode::Up => {
                        if FILE_SELECTIONS[*file_selection] == "Recent Projects"
                            && !recent_projects.is_empty()
                        {
                            *recent_selection = (*recent_selection + recent_projects.len() - 1)
                                % recent_projects.len();
                        }
                    }
                    KeyCode::Down => {
                        if FILE_SELECTIONS[*file_selection] == "Recent Projects"
                            && !recent_projects.is_empty()
                        {
                            *recent_selection = (*recent_selection + 1) % recent_projects.len();
                        }
                    }
                    KeyCode::Char(c) => {
                        if FILE_SELECTIONS[*file_selection] != "Recent Projects" {
                            file_input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if FILE_SELECTIONS[*file_selection] != "Recent Projects" {
                            file_input.pop();
                        }
                    }
                    KeyCode::Delete => {
                        if FILE_SELECTIONS[*file_selection] == "Recent Projects"
                            && !recent_projects.is_empty()
                        {
                            remove_recent_project(&recent_projects[*recent_selection]);
                            self.state = EditorState::SelectingFile {
                                file_selection: *file_selection,
                                file_input: file_input.clone(),
                                file_message: file_message.clone(),
                                recent_projects: load_recent_projects().paths,
                                recent_selection: *recent_selection,
                            };
                        }
                    }
                    _ => {}
                }
            }
            EditorState::Browsing { cursor } => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                    return false;
                }
                match key.code {
                    KeyCode::Up => self.camera.y -= 1,
                    KeyCode::Down => self.camera.y += 1,
                    KeyCode::Left => self.camera.x -= 1,
                    KeyCode::Right => self.camera.x += 1,
                    KeyCode::Delete => {}
                    KeyCode::Char('e') => {
                        let current_pos = self.camera + *cursor;
                        if let Some(object_id) = self.map.positions_hashmap.get(&current_pos) {
                            self.renderer.set_editor_message(EDITING_OBJECT_MESSAGE);
                            self.state = EditorState::EditingObject {
                                object_id: *object_id,
                                selection: 0,
                                edit_selection: 0,
                                selected: false,
                            }
                        } else {
                            self.map.insert_object(
                                current_pos,
                                "♥︎".custom_color(CustomColor::new(255, 0, 0)),
                            );
                        }
                    }
                    KeyCode::Char('s') => match self.save() {
                        Ok(_) => {}
                        Err(e) => {
                            if let EditorState::SelectingFile { file_message, .. } = &mut self.state
                            {
                                self.renderer.editor_message =
                                    format!("Couldnt Save Project: {}", e);
                            }
                            return true;
                        }
                    },
                    KeyCode::Char('m') => {
                        self.renderer
                            .set_editor_message(EDITING_MEASUREMENTS_MESSAGE);
                        self.state = EditorState::EditingMeasurements {
                            selection: 0,
                            selections_selection: 0,
                            selected: false,
                        }
                    }
                    _ => {}
                }
            }
            EditorState::EditingObject {
                object_id,
                selection,
                edit_selection,
                selected,
            } => match key.code {
                KeyCode::Up => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x,
                                        self.map.objects.get(object_id).unwrap().position.y - 1,
                                    ),
                                );
                            }
                            "Color" => {
                                if let Some(object) = self.map.objects.get_mut(object_id) {
                                    if let Some(Color::TrueColor { r, g, b }) =
                                        &mut object.icon.fgcolor
                                    {
                                        match *edit_selection {
                                            0 => *r = r.wrapping_add(1),
                                            1 => *g = g.wrapping_add(1),
                                            2 => *b = b.wrapping_add(1),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        *selection = (*selection + OBJECT_EDIT_SELECTIONS.len() - 1)
                            % OBJECT_EDIT_SELECTIONS.len();
                    }
                }
                KeyCode::Down => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x,
                                        self.map.objects.get(object_id).unwrap().position.y + 1,
                                    ),
                                );
                            }
                            "Color" => {
                                if let Some(object) = self.map.objects.get_mut(object_id) {
                                    if let Some(Color::TrueColor { r, g, b }) =
                                        &mut object.icon.fgcolor
                                    {
                                        match *edit_selection {
                                            0 => *r = r.wrapping_sub(1),
                                            1 => *g = g.wrapping_sub(1),
                                            2 => *b = b.wrapping_sub(1),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        *selection = (*selection + 1) % OBJECT_EDIT_SELECTIONS.len();
                    }
                }
                KeyCode::Left => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x - 1,
                                        self.map.objects.get(object_id).unwrap().position.y,
                                    ),
                                );
                            }
                            "Color" => {
                                *edit_selection = (*edit_selection + 3 - 1) % 3;
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Right => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x + 1,
                                        self.map.objects.get(object_id).unwrap().position.y,
                                    ),
                                );
                            }
                            "Color" => {
                                *edit_selection = (*edit_selection + 1) % 3;
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Enter => {
                    *selected = !*selected;
                    *edit_selection = 0;
                    if OBJECT_EDIT_SELECTIONS[*selection] == "Components" {
                        self.renderer.set_editor_message(EDITING_COMPONENT_MESSAGE);
                        self.state = EditorState::SelectingComponent {
                            object_id: *object_id,
                            selection: 0,
                        }
                    } else if OBJECT_EDIT_SELECTIONS[*selection] == "Camera Operator" {
                        if self.map.camera_operator == *object_id {
                            self.map.camera_operator = 0;
                        } else {
                            self.map.camera_operator = *object_id;
                        }
                        *selected = false;
                    }
                }
                KeyCode::Delete => {
                    self.map.delete_object(*object_id);
                    self.renderer.set_editor_message(BROSWING_MESSAGE);
                    self.state = EditorState::Browsing {
                        cursor: Vector2::new(
                            self.renderer.measurements.screen_size.x / 2,
                            self.renderer.measurements.screen_size.y / 2,
                        ),
                    };
                }
                KeyCode::Esc => {
                    self.renderer.set_editor_message(BROSWING_MESSAGE);
                    self.state = EditorState::Browsing {
                        cursor: Vector2::new(
                            self.renderer.measurements.screen_size.x / 2,
                            self.renderer.measurements.screen_size.y / 2,
                        ),
                    };
                }
                KeyCode::Char(c) => {
                    if *selected && OBJECT_EDIT_SELECTIONS[*selection] == "Icon" {
                        if let Some(object) = self.map.objects.get_mut(object_id) {
                            let color = object.icon.fgcolor.clone();
                            object.icon = c.to_string().custom_color(match color {
                                Some(Color::TrueColor { r, g, b }) => CustomColor::new(r, g, b),
                                _ => CustomColor::new(255, 255, 255),
                            });
                            *selected = false;
                        }
                    }
                }
                _ => {}
            },
            EditorState::SelectingComponent {
                object_id,
                selection,
            } => match key.code {
                KeyCode::Up => {
                    *selection =
                        (*selection + COMPONENT_SELECTIONS.len() - 1) % COMPONENT_SELECTIONS.len();
                }
                KeyCode::Down => {
                    *selection = (*selection + 1) % COMPONENT_SELECTIONS.len();
                }
                KeyCode::Enter => match COMPONENT_SELECTIONS[*selection] {
                    "MoveableComponent" => {
                        self.map.insert_moveable_component(*object_id);
                    }
                    "InputComponent" => {
                        self.map.insert_input_component(*object_id);
                    }
                    "StatsComponent" => {
                        self.map
                            .insert_stats_component(*object_id, StatsComponent::new(0, 0, 0, 0, 0));
                        self.renderer
                            .set_editor_message(EDITING_STATS_COMPONENT_MESSAGE);
                        self.state = EditorState::EditingStatsComponent {
                            object_id: *object_id,
                            selection: 0,
                        }
                    }
                    "EventComponent" => {
                        self.map.insert_event_component(
                            *object_id,
                            vec![EventStep::new(
                                GameEvent::None,
                                EventCondition::None,
                                false,
                                None,
                            )],
                        );
                        self.renderer
                            .set_editor_message(EDITING_EVENT_COMPONENT_MESSAGE);
                        self.state = EditorState::EditingEventComponent {
                            object_id: *object_id,
                            current_step: 0,
                            selection: 0,
                        }
                    }
                    _ => {}
                },
                KeyCode::Delete => match COMPONENT_SELECTIONS[*selection] {
                    "MoveableComponent" => {
                        self.map.moveable_components.remove(object_id);
                    }
                    "InputComponent" => {
                        self.map.input_components.remove(object_id);
                    }
                    "StatsComponent" => {
                        self.map.stats_components.remove(object_id);
                    }
                    "EventComponent" => {
                        self.map.event_components.remove(object_id);
                    }
                    _ => {}
                },
                KeyCode::Esc => {
                    self.renderer.set_editor_message(EDITING_OBJECT_MESSAGE);
                    self.state = EditorState::EditingObject {
                        object_id: *object_id,
                        selection: 3,
                        edit_selection: 0,
                        selected: false,
                    }
                }
                _ => {}
            },
            EditorState::EditingEventComponent {
                object_id,
                current_step,
                selection,
            } => match key.code {
                KeyCode::Up => {
                    let Some(event_comp) = self.map.event_components.get(object_id) else {
                        return true;
                    };
                    *selection = (*selection + 5 - 1) % 5;
                }
                KeyCode::Down => {
                    let Some(event_comp) = self.map.event_components.get(object_id) else {
                        return true;
                    };
                    *selection = (*selection + 1) % 5;
                }
                KeyCode::Left => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    match selection {
                        0 => {
                            *current_step = (*current_step + event_comp.events.len() - 1)
                                % event_comp.events.len();
                        }
                        1 => {
                            event_comp.events[*current_step].event =
                                event_comp.events[*current_step].event.prev()
                        }
                        2 => {
                            event_comp.events[*current_step].requirement =
                                event_comp.events[*current_step].requirement.prev()
                        }
                        3 => {
                            event_comp.events[*current_step].repeat =
                                !event_comp.events[*current_step].repeat
                        }
                        4 => {
                            if let Some(id) = &mut event_comp.events[*current_step].next_event {
                                if *id == 0 {
                                    event_comp.events[*current_step].next_event = None;
                                    return true;
                                }
                                *id -= 1;
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Right => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    match selection {
                        0 => {
                            *current_step = (*current_step + 1) % event_comp.events.len();
                        }
                        1 => {
                            event_comp.events[*current_step].event =
                                event_comp.events[*current_step].event.next()
                        }
                        2 => {
                            event_comp.events[*current_step].requirement =
                                event_comp.events[*current_step].requirement.next()
                        }
                        3 => {
                            event_comp.events[*current_step].repeat =
                                !event_comp.events[*current_step].repeat
                        }
                        4 => {
                            if let Some(id) = &mut event_comp.events[*current_step].next_event {
                                *id += 1;
                            } else {
                                event_comp.events[*current_step].next_event = Some(0);
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    self.renderer.set_editor_message(EDITING_COMPONENT_MESSAGE);
                    self.state = EditorState::SelectingComponent {
                        object_id: *object_id,
                        selection: 2,
                    }
                }
                KeyCode::Char('+') => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    event_comp.events.push(EventStep::new(
                        GameEvent::None,
                        EventCondition::None,
                        false,
                        None,
                    ))
                }
                KeyCode::Delete => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    if event_comp.events.len() <= 1 {
                        self.map.event_components.remove(object_id);
                        self.renderer.set_editor_message(EDITING_COMPONENT_MESSAGE);
                        self.state = EditorState::SelectingComponent {
                            object_id: *object_id,
                            selection: 2,
                        };
                        return true;
                    }
                    event_comp.events.remove(*current_step);
                    if *current_step >= event_comp.events.len() {
                        *current_step = event_comp.events.len() - 1;
                    }
                }
                KeyCode::Enter => match selection {
                    1 => {
                        let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                            return true;
                        };

                        match event_comp.events[*current_step].event {
                            GameEvent::None => {
                                return true;
                            }
                            GameEvent::Dialogue(_) => {
                                self.renderer
                                    .set_editor_message(EDITING_EVENT_MESSAGE_DIALOGUE);
                            }
                            _ => {
                                self.renderer.set_editor_message(EDITING_EVENT_MESSAGE);
                            }
                        }

                        self.state = EditorState::EditingEvent {
                            object_id: *object_id,
                            current_step: *current_step,
                            selection: 0,
                            editing_selection: false,
                            selections_selection: 0,
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            EditorState::EditingEvent {
                object_id,
                current_step,
                selection,
                editing_selection,
                selections_selection,
            } => match key.code {
                KeyCode::Up => {
                    if *editing_selection {
                        let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                            return true;
                        };
                        match &mut event_comp.events[*current_step].event {
                            GameEvent::Combat(combat) => {
                                if let Some(Color::TrueColor { r, g, b }) =
                                    &mut combat.projectile_icon.fgcolor
                                {
                                    match *selections_selection {
                                        0 => *r = r.wrapping_add(1),
                                        1 => *g = g.wrapping_add(1),
                                        2 => *b = b.wrapping_add(1),
                                        _ => {}
                                    }
                                }
                            }
                            GameEvent::Dialogue(dialogue) => {
                                if *selection == 2 {
                                    if let Some(ref mut i) =
                                        dialogue.selections_pointing_event[*selections_selection]
                                    {
                                        *i += 1;
                                    } else {
                                        dialogue.selections_pointing_event[*selections_selection] =
                                            Some(0)
                                    }
                                }
                            }
                            _ => {}
                        }
                        return true;
                    }
                    let len = get_event_field_count(&self.map, *object_id, *current_step);
                    *selection = (*selection + len - 1) % len;
                }
                KeyCode::Down => {
                    if *editing_selection {
                        let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                            return true;
                        };
                        match &mut event_comp.events[*current_step].event {
                            GameEvent::Combat(combat) => {
                                if let Some(Color::TrueColor { r, g, b }) =
                                    &mut combat.projectile_icon.fgcolor
                                {
                                    match *selections_selection {
                                        0 => *r = r.wrapping_sub(1),
                                        1 => *g = g.wrapping_sub(1),
                                        2 => *b = b.wrapping_sub(1),
                                        _ => {}
                                    }
                                }
                            }
                            GameEvent::Dialogue(dialogue) => {
                                if *selection == 2 {
                                    if let Some(ref mut i) =
                                        dialogue.selections_pointing_event[*selections_selection]
                                    {
                                        if *i == 0 {
                                            dialogue.selections_pointing_event
                                                [*selections_selection] = None;
                                            return true;
                                        }
                                        *i -= 1;
                                    }
                                }
                            }
                            _ => {}
                        }
                        return true;
                    }
                    let len = get_event_field_count(&self.map, *object_id, *current_step);
                    *selection = (*selection + 1) % len;
                }
                KeyCode::Right => {
                    if *editing_selection {
                        *selections_selection = (*selections_selection + 1) % 3;
                        return true;
                    }
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    let event = &mut event_comp.events[*current_step];
                    match &mut event.event {
                        GameEvent::Combat(combat) => match *selection {
                            0 => combat.player_goes_first = !combat.player_goes_first,
                            1 => combat.turn_result_time += 1,
                            4 => combat.projectile_damage += 1,
                            5 => combat.projectile_count += 1,
                            6 => combat.projectile_move_time += 1,
                            7 => combat.projectile_spawn_time += 1,
                            8 => combat.delete_when_defeated = !combat.delete_when_defeated,
                            _ => {}
                        },
                        GameEvent::TriggerObjectEvent(id) => match *selection {
                            0 => *id += 1,
                            _ => {}
                        },
                        _ => {}
                    }
                }
                KeyCode::Left => {
                    if *editing_selection {
                        let Some(event_comp) = self.map.event_components.get(object_id) else {
                            return true;
                        };
                        match &event_comp.events[*current_step].event {
                            _ => {}
                        }

                        *selections_selection = (*selections_selection + 3 - 1) % 3;
                        return true;
                    }
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    let event = &mut event_comp.events[*current_step];
                    match &mut event.event {
                        GameEvent::Combat(combat) => match *selection {
                            0 => combat.player_goes_first = !combat.player_goes_first,
                            1 => {
                                combat.turn_result_time = combat.turn_result_time.saturating_sub(1)
                            }
                            4 => {
                                combat.projectile_damage =
                                    combat.projectile_damage.saturating_sub(1)
                            }
                            5 => {
                                combat.projectile_count = combat.projectile_count.saturating_sub(1)
                            }
                            6 => {
                                combat.projectile_move_time =
                                    combat.projectile_move_time.saturating_sub(1)
                            }
                            7 => {
                                combat.projectile_spawn_time =
                                    combat.projectile_spawn_time.saturating_sub(1)
                            }
                            8 => combat.delete_when_defeated = !combat.delete_when_defeated,
                            _ => {}
                        },
                        GameEvent::TriggerObjectEvent(id) => match *selection {
                            0 => *id = id.saturating_sub(1),
                            _ => {}
                        },
                        _ => {}
                    }
                }
                KeyCode::Enter => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    match *selection {
                        1 => {
                            if let GameEvent::Dialogue(dialogue) =
                                &mut event_comp.events[*current_step].event
                                && !dialogue.selections.is_empty()
                            {
                                *editing_selection = !*editing_selection;
                                *selections_selection = 0;
                            }
                        }
                        2 => {
                            if let GameEvent::Dialogue(dialogue) =
                                &mut event_comp.events[*current_step].event
                                && !dialogue.selections_pointing_event.is_empty()
                            {
                                *editing_selection = !*editing_selection;
                                *selections_selection = 0;
                            }
                        }
                        3 => {
                            if let GameEvent::Combat(_) =
                                &mut event_comp.events[*current_step].event
                            {
                                *editing_selection = !*editing_selection;
                                *selections_selection = 0;
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('+') => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    match *selection {
                        1 => {
                            if let GameEvent::Dialogue(dialogue) =
                                &mut event_comp.events[*current_step].event
                            {
                                dialogue.selections.push(String::new());
                            }
                        }
                        2 => {
                            if let GameEvent::Dialogue(dialogue) =
                                &mut event_comp.events[*current_step].event
                            {
                                dialogue.selections_pointing_event.push(None);
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('-') => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    match *selection {
                        1 => {
                            if let GameEvent::Dialogue(dialogue) =
                                &mut event_comp.events[*current_step].event
                            {
                                dialogue.selections.push(String::new());
                            }
                        }
                        2 => {
                            if let GameEvent::Dialogue(dialogue) =
                                &mut event_comp.events[*current_step].event
                            {
                                dialogue.selections_pointing_event.push(None);
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    let event = &mut event_comp.events[*current_step];
                    match &mut event.event {
                        GameEvent::Combat(combat) => {
                            if *selection == 2 {
                                let color = match combat.projectile_icon.fgcolor {
                                    Some(Color::TrueColor { r, g, b }) => CustomColor::new(r, g, b),
                                    _ => CustomColor::new(255, 255, 255),
                                };
                                combat.projectile_icon = c.to_string().custom_color(color);
                            }
                        }
                        GameEvent::Dialogue(dialogue) => match selection {
                            0 => dialogue.text.push(c),
                            1 => {
                                if *editing_selection {
                                    dialogue.selections[*selections_selection].push(c)
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                KeyCode::Backspace => {
                    let Some(event_comp) = self.map.event_components.get_mut(object_id) else {
                        return true;
                    };
                    let event = &mut event_comp.events[*current_step];
                    if let GameEvent::Dialogue(dialogue) = &mut event.event {
                        if *selection == 0 {
                            dialogue.text.pop();
                        } else if *selection == 1 && *editing_selection {
                            dialogue.selections[*selections_selection].pop();
                        }
                    }
                }
                KeyCode::Esc => {
                    *editing_selection = false;
                    self.renderer
                        .set_editor_message(EDITING_EVENT_COMPONENT_MESSAGE);
                    self.state = EditorState::EditingEventComponent {
                        object_id: *object_id,
                        current_step: *current_step,
                        selection: 1,
                    }
                }
                _ => {}
            },
            EditorState::EditingStatsComponent {
                object_id,
                selection,
            } => match key.code {
                KeyCode::Left => match STATS_COMPONENT_SELECTIONS[*selection] {
                    "strength" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.strength = stats.strength.saturating_sub(1);
                        }
                    }
                    "agility" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.agility = stats.agility.saturating_sub(1);
                        }
                    }
                    "defense" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.defense = stats.defense.saturating_sub(1);
                        }
                    }
                    "luck" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.luck = stats.luck.saturating_sub(1);
                        }
                    }
                    "max_health" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.max_health = stats.max_health.saturating_sub(1);
                        }
                    }
                    _ => {}
                },
                KeyCode::Right => match STATS_COMPONENT_SELECTIONS[*selection] {
                    "strength" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.strength += 1;
                        }
                    }
                    "agility" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.agility += 1;
                        }
                    }
                    "defense" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.defense += 1;
                        }
                    }
                    "luck" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.luck += 1;
                        }
                    }
                    "max_health" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.max_health += 1;
                        }
                    }
                    _ => {}
                },
                KeyCode::Up => {
                    *selection = (*selection + STATS_COMPONENT_SELECTIONS.len() - 1)
                        % STATS_COMPONENT_SELECTIONS.len();
                }
                KeyCode::Down => {
                    *selection = (*selection + 1) % STATS_COMPONENT_SELECTIONS.len();
                }
                KeyCode::Esc => {
                    self.renderer.set_editor_message(EDITING_COMPONENT_MESSAGE);
                    self.state = EditorState::SelectingComponent {
                        object_id: *object_id,
                        selection: 3,
                    }
                }
                _ => {}
            },
            EditorState::EditingMeasurements {
                selection,
                selected,
                selections_selection,
            } => match key.code {
                KeyCode::Up => {
                    if *selected {
                        match EDIT_SCREEN_MEASUREMENTS_SELECTIONS[*selection] {
                            "screen_size" => match *selections_selection {
                                0 => self.renderer.measurements.screen_size.x += 1,
                                1 => self.renderer.measurements.screen_size.y += 1,
                                _ => {}
                            },
                            "screen_margins" => match *selections_selection {
                                0 => self.renderer.measurements.screen_margins.x += 1,
                                1 => self.renderer.measurements.screen_margins.y += 1,
                                _ => {}
                            },
                            "dialogue_padding" => {
                                self.renderer.measurements.dialogue_padding =
                                    self.renderer.measurements.dialogue_padding.wrapping_add(1);
                            }
                            "dialogue_text_padding" => {
                                self.renderer.measurements.dialogue_text_padding = self
                                    .renderer
                                    .measurements
                                    .dialogue_text_padding
                                    .wrapping_add(1);
                            }
                            "dialogue_selection_text_padding" => {
                                self.renderer.measurements.dialogue_selection_text_padding = self
                                    .renderer
                                    .measurements
                                    .dialogue_selection_text_padding
                                    .wrapping_add(1);
                            }
                            "dialogue_max_character_count" => {
                                self.renderer.measurements.dialogue_max_character_count = self
                                    .renderer
                                    .measurements
                                    .dialogue_max_character_count
                                    .wrapping_add(1);
                            }
                            "combat_character_padding_y" => {
                                self.renderer.measurements.combat_character_padding_y = self
                                    .renderer
                                    .measurements
                                    .combat_character_padding_y
                                    .wrapping_add(1);
                            }
                            "combat_character_padding_x" => {
                                self.renderer.measurements.combat_character_padding_x = self
                                    .renderer
                                    .measurements
                                    .combat_character_padding_x
                                    .wrapping_add(1);
                            }
                            "combat_characters_distance" => {
                                self.renderer.measurements.combat_characters_distance = self
                                    .renderer
                                    .measurements
                                    .combat_characters_distance
                                    .wrapping_add(1);
                            }
                            "combat_separator_padding_y" => {
                                self.renderer.measurements.combat_separator_padding_y = self
                                    .renderer
                                    .measurements
                                    .combat_separator_padding_y
                                    .wrapping_add(1);
                            }
                            "combat_selection_separator_padding" => {
                                self.renderer
                                    .measurements
                                    .combat_selection_separator_padding = self
                                    .renderer
                                    .measurements
                                    .combat_selection_separator_padding
                                    .wrapping_add(1);
                            }
                            "combat_health_padding_y" => {
                                self.renderer.measurements.combat_health_padding_y = self
                                    .renderer
                                    .measurements
                                    .combat_health_padding_y
                                    .wrapping_add(1);
                            }
                            _ => {}
                        }
                        return true;
                    }
                    *selection = (*selection + EDIT_SCREEN_MEASUREMENTS_SELECTIONS.len() - 1)
                        % EDIT_SCREEN_MEASUREMENTS_SELECTIONS.len();
                }
                KeyCode::Down => {
                    if *selected {
                        match EDIT_SCREEN_MEASUREMENTS_SELECTIONS[*selection] {
                            "screen_size" => match *selections_selection {
                                0 => {
                                    self.renderer.measurements.screen_size.x =
                                        (self.renderer.measurements.screen_size.x - 1).max(0)
                                }
                                1 => {
                                    self.renderer.measurements.screen_size.y =
                                        (self.renderer.measurements.screen_size.y - 1).max(0)
                                }
                                _ => {}
                            },
                            "screen_margins" => match *selections_selection {
                                0 => {
                                    self.renderer.measurements.screen_margins.x =
                                        (self.renderer.measurements.screen_margins.x - 1).max(0)
                                }
                                1 => {
                                    self.renderer.measurements.screen_margins.y =
                                        (self.renderer.measurements.screen_margins.y - 1).max(0)
                                }
                                _ => {}
                            },
                            "dialogue_padding" => {
                                self.renderer.measurements.dialogue_padding =
                                    self.renderer.measurements.dialogue_padding.wrapping_sub(1);
                            }
                            "dialogue_text_padding" => {
                                self.renderer.measurements.dialogue_text_padding = self
                                    .renderer
                                    .measurements
                                    .dialogue_text_padding
                                    .wrapping_sub(1);
                            }
                            "dialogue_selection_text_padding" => {
                                self.renderer.measurements.dialogue_selection_text_padding = self
                                    .renderer
                                    .measurements
                                    .dialogue_selection_text_padding
                                    .wrapping_sub(1);
                            }
                            "dialogue_max_character_count" => {
                                self.renderer.measurements.dialogue_max_character_count = self
                                    .renderer
                                    .measurements
                                    .dialogue_max_character_count
                                    .wrapping_sub(1);
                            }
                            "combat_character_padding_y" => {
                                self.renderer.measurements.combat_character_padding_y = self
                                    .renderer
                                    .measurements
                                    .combat_character_padding_y
                                    .wrapping_sub(1);
                            }
                            "combat_character_padding_x" => {
                                self.renderer.measurements.combat_character_padding_x = self
                                    .renderer
                                    .measurements
                                    .combat_character_padding_x
                                    .wrapping_sub(1);
                            }
                            "combat_characters_distance" => {
                                self.renderer.measurements.combat_characters_distance = self
                                    .renderer
                                    .measurements
                                    .combat_characters_distance
                                    .wrapping_sub(1);
                            }
                            "combat_separator_padding_y" => {
                                self.renderer.measurements.combat_separator_padding_y = self
                                    .renderer
                                    .measurements
                                    .combat_separator_padding_y
                                    .wrapping_sub(1);
                            }
                            "combat_selection_separator_padding" => {
                                self.renderer
                                    .measurements
                                    .combat_selection_separator_padding = self
                                    .renderer
                                    .measurements
                                    .combat_selection_separator_padding
                                    .wrapping_sub(1);
                            }
                            "combat_health_padding_y" => {
                                self.renderer.measurements.combat_health_padding_y = self
                                    .renderer
                                    .measurements
                                    .combat_health_padding_y
                                    .wrapping_sub(1);
                            }
                            _ => {}
                        }
                        return true;
                    }
                    *selection = (*selection + 1) % EDIT_SCREEN_MEASUREMENTS_SELECTIONS.len();
                }
                KeyCode::Left => {
                    if !*selected {
                        *selection = (*selection + EDIT_SCREEN_MEASUREMENTS_SELECTIONS.len() - 1)
                            % EDIT_SCREEN_MEASUREMENTS_SELECTIONS.len();
                        *selections_selection = 0;
                        return true;
                    }
                    let max = match EDIT_SCREEN_MEASUREMENTS_SELECTIONS[*selection] {
                        "screen_size" | "screen_margins" => 2,
                        _ => 1,
                    };
                    *selections_selection = (*selections_selection + max - 1) % max;
                }
                KeyCode::Right => {
                    if !*selected {
                        *selection = (*selection + 1) % EDIT_SCREEN_MEASUREMENTS_SELECTIONS.len();
                        *selections_selection = 0;
                        return true;
                    }
                    let max = match EDIT_SCREEN_MEASUREMENTS_SELECTIONS[*selection] {
                        "screen_size" | "screen_margins" => 2,
                        _ => 1,
                    };
                    *selections_selection = (*selections_selection + 1) % max;
                }
                KeyCode::Enter => *selected = !*selected,
                KeyCode::Esc => {
                    self.renderer.set_editor_message(BROSWING_MESSAGE);
                    self.state = EditorState::Browsing {
                        cursor: Vector2::new(
                            self.renderer.measurements.screen_size.x / 2,
                            self.renderer.measurements.screen_size.y / 2,
                        ),
                    };
                }
                _ => {}
            },
        }
        return true;
    }
}

pub fn get_event_field_count(map: &Map, object_id: GameObjectID, current_step: usize) -> usize {
    let Some(event_comp) = map.event_components.get(&object_id) else {
        return 1;
    };
    match &event_comp.events[current_step].event {
        GameEvent::Dialogue(_) => 3,
        GameEvent::Combat(_) => 9,
        GameEvent::TriggerObjectEvent(_) => 1,
        GameEvent::None => 0,
    }
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

    let mut editor = Editor::new();
    //let mut last_frame = Instant::now();

    loop {
        //let delta_ms = last_frame.elapsed().as_millis() as usize;
        //last_frame = Instant::now();

        execute!(stdout, cursor::MoveTo(0, 0))?;

        editor
            .renderer
            .render_editor(&editor.state, &editor.camera, &editor.map);

        stdout.flush()?;

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(key_event) = event::read()?
        {
            if editor.process_input(key_event) == false && key_event.kind == KeyEventKind::Press {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(32));
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
