use crate::{
    game_object::{
        Combat, CombatPhase, Dialogue, EventComponent, EventCondition, EventStep, GameEvent,
        GameObjectID, StatsComponent,
    },
    level::{
        add_recent_project, data_to_map, load_map, load_measurements, load_recent_projects,
        map_to_data, remove_recent_project, save_map, save_measurements,
    },
    map::Map,
    renderer::{Renderer, ScreenMeasurements},
    utils::{wrap_add, wrap_add_reverse, wrap_remove},
    vector2::Vector2,
};
use colored::*;
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::{
    any::Any,
    collections::HashMap,
    i32,
    io::{self, Write, stdout},
    path::Path,
    u8, usize,
};
use std::{panic, time::Duration};

pub const FILE_SELECTIONS: &[&str] = &["New Project", "Open Project", "Recent Projects"];
pub const BROSWING_MESSAGE: &str =
    "←↑→↓:Move, e:Insert/Edit object, s:Save, CTRL+q:Quit, m:Edit screen measurements";
pub const EDITING_MESSAGE: &str =
    "←↑→↓:Move selection, ENTER:Select/DeSelect property, ESC:Go back, BACKSPACE:DeSelect property";

#[derive(Clone)]
pub enum EditorState {
    SelectingFile {
        file_selection: usize,
        file_input: String,
        file_message: String,
        recent_projects: Vec<String>,
        recent_selection: usize,
    },
    Browsing,
    EditingMeasurements,
    EditingObject(GameObjectID),
    SelectingComponent(GameObjectID),
    EditingEventComponent(GameObjectID),
    EditingDialogueEvent(GameObjectID, usize /* event id */),
    EditingTriggerObjectEvent(GameObjectID, usize /* event id */),
    EditingCombatEvent(GameObjectID, usize /* event id */),
    EditingStatsComponent(GameObjectID),
}

pub struct Editor {
    pub map: Map,
    pub camera: Vector2,
    pub renderer: Renderer,
    pub state: EditorState,
    pub layout: Layout,
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
            layout: Layout::new(),
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
        self.change_state(EditorState::Browsing);
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
                            self.change_state(EditorState::Browsing);
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
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            return false;
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
            EditorState::Browsing => {
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
                        let current_pos =
                            self.camera + (self.renderer.measurements.screen_size / 2);
                        if let Some(object_id) = self.map.positions_hashmap.get(&current_pos) {
                            self.change_state(EditorState::EditingObject(*object_id));
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
                            if let EditorState::SelectingFile { .. } = &mut self.state {
                                self.renderer.editor_message =
                                    format!("Couldnt Save Project: {}", e);
                            }
                            return true;
                        }
                    },
                    KeyCode::Char('m') => {
                        self.change_state(EditorState::EditingMeasurements);
                    }
                    _ => {}
                }
            }
            _ => match key.code {
                KeyCode::Up => {
                    if let Some(button) = self.layout.buttons.get_mut(&self.layout.current_button) {
                        if button.selected {
                            button.button_up();
                        } else {
                            self.layout.current_button = wrap_remove(
                                // this is up because we render the first index first
                                self.layout.current_button,
                                1,
                                self.layout.buttons.len() - 1,
                                0,
                            );
                        }
                    }
                }
                KeyCode::Down => {
                    if let Some(button) = self.layout.buttons.get_mut(&self.layout.current_button) {
                        if button.selected {
                            button.button_down();
                        } else {
                            self.layout.current_button = wrap_add_reverse(
                                self.layout.current_button,
                                1,
                                self.layout.buttons.len(),
                            );
                        }
                    }
                }
                KeyCode::Left => {
                    if let Some(button) = self.layout.buttons.get_mut(&self.layout.current_button) {
                        button.button_left();
                    }
                }
                KeyCode::Right => {
                    if let Some(button) = self.layout.buttons.get_mut(&self.layout.current_button) {
                        button.button_right();
                    }
                }
                KeyCode::Enter => {
                    if let Some(button) = self.layout.buttons.get_mut(&self.layout.current_button) {
                        button.button_selected();
                    }
                }
                KeyCode::Backspace => {
                    if let Some(button) = self.layout.buttons.get_mut(&self.layout.current_button) {
                        button.button_backspace();
                    }
                }
                KeyCode::Char(ch) => {
                    if let Some(button) = self.layout.buttons.get_mut(&self.layout.current_button) {
                        button.button_char(ch);
                    }
                }
                KeyCode::Esc => {
                    self.change_to_last_state();
                }
                _ => {}
            },
        }
        return true;
    }

    pub fn change_state(&mut self, new_state: EditorState) {
        self.layout.buttons.clear();
        self.layout.current_button = 0;
        self.state = new_state;
        match &self.state {
            EditorState::Browsing => {
                self.renderer.set_editor_message(BROSWING_MESSAGE);
            }
            EditorState::EditingMeasurements => {
                self.renderer.set_editor_message(EDITING_MESSAGE);
                self.layout.add_button(
                    "Screen size".to_string(),
                    ButtonValue::Vector2(self.renderer.measurements.screen_size),
                );
                self.layout.add_button(
                    "Screen margins".to_string(),
                    ButtonValue::Vector2(self.renderer.measurements.screen_margins),
                );
                self.layout.add_button(
                    "Dialogue padding".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.dialogue_padding),
                );
                self.layout.add_button(
                    "Dialogue text padding".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.dialogue_text_padding),
                );
                self.layout.add_button(
                    "Dialogue selection text padding".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.dialogue_selection_text_padding),
                );
                self.layout.add_button(
                    "Combat character padding x".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.combat_character_padding_x),
                );
                self.layout.add_button(
                    "Combat character padding y".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.combat_character_padding_y),
                );
                self.layout.add_button(
                    "Combat character distance".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.combat_characters_distance),
                );
                self.layout.add_button(
                    "Combat seperator padding y".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.combat_separator_padding_y),
                );
                self.layout.add_button(
                    "Combat selection seperator padding".to_string(),
                    ButtonValue::Usize(
                        self.renderer
                            .measurements
                            .combat_selection_separator_padding,
                    ),
                );
                self.layout.add_button(
                    "Combat health padding y".to_string(),
                    ButtonValue::Usize(self.renderer.measurements.combat_health_padding_y),
                );
            }
            EditorState::EditingObject(object_id) => {
                self.renderer.set_editor_message(EDITING_MESSAGE);
                let Some(object) = self.map.objects.get(object_id) else {
                    return;
                };
                self.layout.add_button(
                    "Position".to_string(),
                    ButtonValue::Vector2(object.position),
                );
                self.layout.add_button(
                    "Icon".to_string(),
                    ButtonValue::Char(object.icon.to_string().chars().next().unwrap()), // this shit is a mouthfull
                );
                self.layout
                    .add_button("Color".to_string(), ButtonValue::Color(object.icon.fgcolor));
                self.layout.add_button(
                    "Components".to_string(),
                    ButtonValue::StateChange(EditorState::SelectingComponent(*object_id)),
                );
                self.layout.add_button(
                    "Camera Operator".to_string(),
                    ButtonValue::Bool(self.map.camera_operator == *object_id),
                );
                self.layout
                    .add_button("Delete Object".to_string(), ButtonValue::Bool(false));
            }
            EditorState::SelectingComponent(object_id) => {
                self.renderer.set_editor_message(EDITING_MESSAGE);
                self.layout.add_button(
                    "Moveable Component".to_string(),
                    ButtonValue::Bool(self.map.moveable_components.contains_key(object_id)),
                );
                self.layout.add_button(
                    "Input Component".to_string(),
                    ButtonValue::Bool(self.map.input_components.contains_key(object_id)),
                );
                let event_comp_str = if self.map.event_components.contains_key(object_id) {
                    "Event Component  X".to_string()
                } else {
                    "Event Component".to_string()
                };
                self.layout.add_button(
                    event_comp_str,
                    ButtonValue::StateChange(EditorState::EditingEventComponent(*object_id)),
                );
                let stats_comp_str = if self.map.stats_components.contains_key(object_id) {
                    "Stats Component  X".to_string()
                } else {
                    "Stats Component".to_string()
                };
                self.layout.add_button(
                    stats_comp_str,
                    ButtonValue::StateChange(EditorState::EditingStatsComponent(*object_id)),
                );
            }
            EditorState::EditingStatsComponent(object_id) => {
                self.renderer.set_editor_message(EDITING_MESSAGE);

                let Some(stats_comp) = self.map.stats_components.get(object_id) else {
                    return;
                };

                self.layout.add_button(
                    "Strength".to_string(),
                    ButtonValue::Usize(stats_comp.strength),
                );
                self.layout.add_button(
                    "Agility".to_string(),
                    ButtonValue::Usize(stats_comp.agility),
                );
                self.layout.add_button(
                    "Defense".to_string(),
                    ButtonValue::Usize(stats_comp.defense),
                );
                self.layout
                    .add_button("Luck".to_string(), ButtonValue::Usize(stats_comp.luck));
                self.layout.add_button(
                    "Max Health".to_string(),
                    ButtonValue::Usize(stats_comp.max_health),
                );
                self.layout.add_button(
                    "Delete Stats Component".to_string(),
                    ButtonValue::Bool(false),
                );
            }
            EditorState::EditingEventComponent(object_id) => {
                self.renderer.set_editor_message(EDITING_MESSAGE);
                let Some(event_comp) = self.map.event_components.get(object_id) else {
                    return;
                };

                self.layout.add_button(
                    "".to_string(),
                    ButtonValue::IndexSelection(0, event_comp.events.len()),
                );
                self.layout.add_button(
                    "Event :".to_string(),
                    ButtonValue::Enum(event_comp.events[0].event.clone_box()),
                );
                self.layout.add_button(
                    "Event requirement :".to_string(),
                    ButtonValue::Enum(event_comp.events[0].requirement.clone_box()),
                );
                self.layout.add_button(
                    "Repeat if requirement not met".to_string(),
                    ButtonValue::Bool(event_comp.events[0].repeat),
                );
                self.layout.add_button(
                    "Next Event ID".to_string(),
                    ButtonValue::OptionUsize(event_comp.events[0].next_event),
                );
                self.layout
                    .add_button("Add event step".to_string(), ButtonValue::Bool(false));
                self.layout
                    .add_button("Remove event step".to_string(), ButtonValue::Bool(false));
            }
            EditorState::EditingDialogueEvent(object_id, event_id) => {
                let Some(event_comp) = self.map.event_components.get(object_id) else {
                    return;
                };
                let GameEvent::Dialogue(dialogue) = &event_comp.events[*event_id].event else {
                    return;
                };
                self.layout.add_button(
                    "Text:".to_string(),
                    ButtonValue::String(dialogue.text.clone()),
                );

                let mut buttons: Vec<Button> = Vec::new();
                for i in &dialogue.selections {
                    buttons.push(Button::new("".to_string(), ButtonValue::String(i.clone())));
                }
                self.layout.add_button(
                    "Selections".to_string(),
                    ButtonValue::SubButtons(
                        buttons,
                        Box::new(Button::new(
                            "".to_string(),
                            ButtonValue::String("".to_string()),
                        )),
                    ),
                );
                let mut buttons: Vec<Button> = Vec::new();
                for i in &dialogue.selections_pointing_event {
                    buttons.push(Button::new(
                        "".to_string(),
                        ButtonValue::OptionUsize(i.clone()),
                    ));
                }
                self.layout.add_button(
                    "Selections Pointing Event".to_string(),
                    ButtonValue::SubButtons(
                        buttons,
                        Box::new(Button::new("".to_string(), ButtonValue::OptionUsize(None))),
                    ),
                );
            }
            EditorState::EditingTriggerObjectEvent(object_id, event_id) => {
                let Some(event_comp) = self.map.event_components.get(object_id) else {
                    return;
                };
                let GameEvent::TriggerObjectEvent(trigger_object) =
                    &event_comp.events[*event_id].event
                else {
                    return;
                };
                self.layout.add_button(
                    "Target ID:".to_string(),
                    ButtonValue::Usize(*trigger_object),
                );
            }
            EditorState::EditingCombatEvent(object_id, event_id) => {
                let Some(event_comp) = self.map.event_components.get(object_id) else {
                    return;
                };
                let GameEvent::Combat(combat) = &event_comp.events[*event_id].event else {
                    return;
                };
                self.layout.add_button(
                    "Player goes first:".to_string(),
                    ButtonValue::Bool(combat.player_goes_first),
                );
                self.layout.add_button(
                    "Turn result time".to_string(),
                    ButtonValue::Usize(combat.turn_result_time),
                );
                self.layout.add_button(
                    "Projectile icon".to_string(),
                    ButtonValue::Char(combat.projectile_icon.to_string().chars().next().unwrap()),
                );
                self.layout.add_button(
                    "Projectile color".to_string(),
                    ButtonValue::Color(combat.projectile_icon.fgcolor),
                );
                self.layout.add_button(
                    "Projectile damage:".to_string(),
                    ButtonValue::Usize(combat.projectile_damage),
                );
                self.layout.add_button(
                    "Projectile count:".to_string(),
                    ButtonValue::Usize(combat.projectile_count),
                );
                self.layout.add_button(
                    "Projectile move time:".to_string(),
                    ButtonValue::Usize(combat.projectile_move_time),
                );
                self.layout.add_button(
                    "Projectile spawn time:".to_string(),
                    ButtonValue::Usize(combat.projectile_spawn_time),
                );
                self.layout.add_button(
                    "Delete When Defeated".to_string(),
                    ButtonValue::Bool(combat.delete_when_defeated),
                );
            }
            _ => {}
        }
    }

    fn change_to_last_state(&mut self) {
        match &self.state {
            EditorState::EditingObject(_) | EditorState::EditingMeasurements => {
                self.change_state(EditorState::Browsing)
            }
            EditorState::SelectingComponent(object_id) => {
                self.change_state(EditorState::EditingObject(*object_id));
            }
            EditorState::EditingStatsComponent(object_id)
            | EditorState::EditingEventComponent(object_id) => {
                self.change_state(EditorState::SelectingComponent(*object_id));
            }
            EditorState::EditingDialogueEvent(object_id, _)
            | EditorState::EditingTriggerObjectEvent(object_id, _)
            | EditorState::EditingCombatEvent(object_id, _) => {
                self.change_state(EditorState::EditingEventComponent(*object_id));
            }
            _ => {}
        }
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
            .render_editor(&editor.state, &editor.camera, &editor.map, &editor.layout);

        stdout.flush()?;

        if event::poll(Duration::from_millis(0))?
            && let Ok(key_event) = event::read()
            && let Some(event) = key_event.as_key_press_event()
        {
            if editor.process_input(event) {
                match &editor.state {
                    EditorState::EditingObject(object_id) => {
                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && button.value_changed
                        {
                            if let ButtonValue::Vector2(vec) = button.value {
                                editor.map.change_object_position(*object_id, vec);
                                button.value_changed = false;
                            }
                        }
                        let Some(object) = editor.map.objects.get_mut(object_id) else {
                            break;
                        };
                        if let Some(button) = editor.layout.buttons.get_mut(&1)
                            && button.value_changed
                        {
                            if let ButtonValue::Char(ch) = button.value {
                                if let Some(Color::TrueColor { r, g, b }) = &object.icon.fgcolor {
                                    object.icon =
                                        ch.to_string().custom_color(CustomColor::new(*r, *g, *b));
                                    button.value_changed = false;
                                }
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&2)
                            && button.value_changed
                        {
                            if let ButtonValue::Color(color) = button.value {
                                object.icon.fgcolor = color;
                                button.value_changed = false;
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get(&3)
                            && button.selected
                        {
                            if let ButtonValue::StateChange(state) = button.value.clone() {
                                editor.change_state(state.clone());
                                continue;
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&4)
                            && button.value_changed
                        {
                            if let ButtonValue::Bool(value) = &mut button.value {
                                if editor.map.objects.len() > 1 {
                                    if editor.map.camera_operator == *object_id {
                                        editor.map.camera_operator = wrap_remove(
                                            *object_id,
                                            1,
                                            editor.map.objects.len() - 1,
                                            0,
                                        );
                                        *value = false;
                                    } else {
                                        editor.map.camera_operator = *object_id;
                                        *value = true;
                                    }
                                } else {
                                    editor.map.camera_operator = *object_id;
                                    *value = true;
                                }
                                button.value_changed = false;
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&5)
                            && button.value_changed
                        {
                            editor.map.objects.remove(object_id);
                            editor.change_to_last_state();
                            continue;
                        }
                    }
                    EditorState::EditingMeasurements => {
                        let m = &mut editor.renderer.measurements;
                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && button.value_changed
                            && let ButtonValue::Vector2(value) = &mut button.value
                        {
                            m.screen_size = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&1)
                            && button.value_changed
                            && let ButtonValue::Vector2(value) = &mut button.value
                        {
                            m.screen_margins = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&2)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.dialogue_padding = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&3)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.dialogue_text_padding = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&4)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.dialogue_selection_text_padding = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&5)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.combat_character_padding_x = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&6)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.combat_character_padding_y = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&7)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.combat_characters_distance = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&8)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.combat_separator_padding_y = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&9)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.combat_selection_separator_padding = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&10)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            m.combat_health_padding_y = *value;
                            button.value_changed = false;
                        }
                    }
                    EditorState::SelectingComponent(object_id) => {
                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && button.value_changed
                            && let ButtonValue::Bool(value) = &mut button.value
                        {
                            button.value_changed = false;
                            button.selected = false;
                            if editor.map.moveable_components.contains_key(object_id) {
                                editor.map.moveable_components.remove(object_id);
                                *value = false;
                            } else {
                                editor.map.insert_moveable_component(*object_id);
                                *value = true;
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&1)
                            && button.value_changed
                            && let ButtonValue::Bool(value) = &mut button.value
                        {
                            button.value_changed = false;
                            button.selected = false;
                            if editor.map.input_components.contains_key(object_id) {
                                editor.map.input_components.remove(object_id);
                                *value = false;
                            } else {
                                editor.map.insert_input_component(*object_id);
                                *value = true;
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get(&2)
                            && button.selected
                            && let ButtonValue::StateChange(state) = &button.value
                        {
                            editor.map.insert_event_component(
                                *object_id,
                                vec![EventStep::new(
                                    GameEvent::None,
                                    EventCondition::None,
                                    false,
                                    None,
                                )],
                            );
                            editor.change_state(state.clone());
                            continue;
                        }
                        if let Some(button) = editor.layout.buttons.get(&3)
                            && button.selected
                            && let ButtonValue::StateChange(state) = &button.value
                        {
                            editor.map.insert_stats_component(
                                *object_id,
                                StatsComponent::new(0, 0, 0, 0, 0),
                            );
                            editor.change_state(state.clone());
                            continue;
                        }
                    }
                    EditorState::EditingStatsComponent(object_id) => {
                        let Some(stats_comp) = editor.map.stats_components.get_mut(object_id)
                        else {
                            continue;
                        };
                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            stats_comp.strength = *value;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&1)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            stats_comp.agility = *value;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&2)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            stats_comp.defense = *value;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&3)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            stats_comp.luck = *value;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&4)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            stats_comp.max_health = *value;
                        }
                        if let Some(button) = editor.layout.buttons.get(&5)
                            && button.value_changed
                        {
                            editor.map.stats_components.remove(object_id);
                            editor.change_to_last_state();
                            continue;
                        }
                    }
                    EditorState::EditingEventComponent(object_id) => {
                        let Some(event_comp) = editor.map.event_components.get_mut(object_id)
                        else {
                            continue;
                        };
                        let mut current_index = 0;
                        let reset_buttons =
                            |i: usize, layout: &mut Layout, comp: &EventComponent| {
                                if let Some(button) = layout.buttons.get_mut(&0)
                                    && let ButtonValue::IndexSelection(index, length) =
                                        &mut button.value
                                {
                                    *index = i;
                                    *length = comp.events.len();
                                }
                                if let Some(button) = layout.buttons.get_mut(&1)
                                    && let ButtonValue::Enum(value) = &mut button.value
                                {
                                    *value = comp.events[i].event.clone_box();
                                }
                                if let Some(button) = layout.buttons.get_mut(&2)
                                    && let ButtonValue::Enum(value) = &mut button.value
                                {
                                    *value = comp.events[i].requirement.clone_box();
                                }
                                if let Some(button) = layout.buttons.get_mut(&3)
                                    && let ButtonValue::Bool(value) = &mut button.value
                                {
                                    *value = comp.events[i].repeat;
                                }
                                if let Some(button) = layout.buttons.get_mut(&4)
                                    && let ButtonValue::OptionUsize(value) = &mut button.value
                                {
                                    *value = comp.events[i].next_event;
                                }
                            };

                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && let ButtonValue::IndexSelection(index, _) = &mut button.value
                        {
                            current_index = *index;
                            if button.value_changed {
                                button.value_changed = false;
                                reset_buttons(current_index, &mut editor.layout, event_comp);
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&1)
                            && let ButtonValue::Enum(value) = &mut button.value
                            && let Some(game_event) = value.as_any().downcast_ref::<GameEvent>()
                        {
                            if button.selected {
                                match game_event {
                                    GameEvent::Dialogue(_) => {
                                        editor.change_state(EditorState::EditingDialogueEvent(
                                            *object_id,
                                            current_index,
                                        ));
                                        continue;
                                    }
                                    GameEvent::Combat(_) => {
                                        editor.change_state(EditorState::EditingCombatEvent(
                                            *object_id,
                                            current_index,
                                        ));
                                        continue;
                                    }
                                    GameEvent::TriggerObjectEvent(_) => {
                                        editor.change_state(
                                            EditorState::EditingTriggerObjectEvent(
                                                *object_id,
                                                current_index,
                                            ),
                                        );
                                        continue;
                                    }
                                    GameEvent::None => {}
                                }
                            }
                            if button.value_changed {
                                event_comp.events[current_index].event = game_event.clone();
                                button.value_changed = false;
                            }
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&2)
                            && button.value_changed
                            && let ButtonValue::Enum(value) = &mut button.value
                            && let Some(condition) = value.as_any().downcast_ref::<EventCondition>()
                        {
                            event_comp.events[current_index].requirement = condition.clone();
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&3)
                            && button.value_changed
                            && let ButtonValue::Bool(value) = &mut button.value
                        {
                            event_comp.events[current_index].repeat = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&4)
                            && button.value_changed
                            && let ButtonValue::OptionUsize(value) = &mut button.value
                        {
                            event_comp.events[current_index].next_event = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&5)
                            && button.value_changed
                            && let ButtonValue::Bool(value) = &mut button.value
                        {
                            event_comp.events.push(EventStep::new(
                                GameEvent::None,
                                EventCondition::None,
                                false,
                                None,
                            ));
                            button.value_changed = false;
                            *value = false;
                            reset_buttons(current_index + 1, &mut editor.layout, event_comp);
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&6)
                            && button.value_changed
                            && let ButtonValue::Bool(value) = &mut button.value
                        {
                            if event_comp.events.len() > 1 {
                                event_comp.events.remove(current_index);
                                button.value_changed = false;
                                *value = false;
                                current_index =
                                    wrap_remove(current_index, 1, event_comp.events.len() - 1, 0);
                                reset_buttons(current_index, &mut editor.layout, event_comp);
                            } else {
                                editor.map.event_components.remove(object_id);
                                editor.change_to_last_state();
                                continue;
                            }
                        }
                    }
                    EditorState::EditingDialogueEvent(object_id, event_id) => {
                        let Some(event_comp) = editor.map.event_components.get_mut(object_id)
                        else {
                            continue;
                        };
                        let GameEvent::Dialogue(dialogue) = &mut event_comp.events[*event_id].event
                        else {
                            continue;
                        };
                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && button.value_changed
                            && let ButtonValue::String(str) = &mut button.value
                        {
                            dialogue.text = str.clone();
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&1)
                            && button.value_changed
                            && let ButtonValue::SubButtons(buttons, _) = &mut button.value
                        {
                            dialogue.selections.clear();
                            for b in buttons {
                                if let ButtonValue::String(str) = &mut b.value {
                                    dialogue.selections.push(str.clone());
                                }
                            }
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&2)
                            && button.value_changed
                            && let ButtonValue::SubButtons(buttons, _) = &mut button.value
                        {
                            dialogue.selections_pointing_event.clear();
                            for b in buttons {
                                if let ButtonValue::OptionUsize(value) = &mut b.value {
                                    dialogue.selections_pointing_event.push(*value);
                                }
                            }
                            button.value_changed = false;
                        }
                    }
                    EditorState::EditingTriggerObjectEvent(object_id, event_id) => {
                        let Some(event_comp) = editor.map.event_components.get_mut(object_id)
                        else {
                            continue;
                        };
                        let GameEvent::TriggerObjectEvent(trigger_object_id) =
                            &mut event_comp.events[*event_id].event
                        else {
                            continue;
                        };
                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            *trigger_object_id = *value;
                            button.value_changed = false;
                        }
                    }
                    EditorState::EditingCombatEvent(object_id, event_id) => {
                        let Some(event_comp) = editor.map.event_components.get_mut(object_id)
                        else {
                            continue;
                        };
                        let GameEvent::Combat(combat) = &mut event_comp.events[*event_id].event
                        else {
                            continue;
                        };
                        if let Some(button) = editor.layout.buttons.get_mut(&0)
                            && button.value_changed
                            && let ButtonValue::Bool(value) = &mut button.value
                        {
                            combat.player_goes_first = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&1)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            combat.turn_result_time = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&2)
                            && button.value_changed
                            && let ButtonValue::Char(ch) = &mut button.value
                        {
                            if let Some(Color::TrueColor { r, g, b }) =
                                &combat.projectile_icon.fgcolor
                            {
                                combat.projectile_icon =
                                    ch.to_string().custom_color(CustomColor::new(*r, *g, *b));
                            }
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&3)
                            && button.value_changed
                            && let ButtonValue::Color(color) = &mut button.value
                        {
                            combat.projectile_icon.fgcolor = *color;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&4)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            combat.projectile_damage = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&5)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            combat.projectile_count = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&6)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            combat.projectile_move_time = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&7)
                            && button.value_changed
                            && let ButtonValue::Usize(value) = &mut button.value
                        {
                            combat.projectile_spawn_time = *value;
                            button.value_changed = false;
                        }
                        if let Some(button) = editor.layout.buttons.get_mut(&8)
                            && button.value_changed
                            && let ButtonValue::Bool(value) = &mut button.value
                        {
                            combat.delete_when_defeated = *value;
                            button.value_changed = false;
                        }
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(32));
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}

pub trait UiEnum {
    fn name(&self) -> &str;
    fn index(&self) -> usize;
    fn next(&self) -> Box<dyn UiEnum>;
    fn prev(&self) -> Box<dyn UiEnum>;
    fn clone_box(&self) -> Box<dyn UiEnum>;
    fn as_any(&self) -> &dyn Any;
}
impl UiEnum for GameEvent {
    fn name(&self) -> &str {
        match self {
            GameEvent::None => "None",
            GameEvent::Dialogue(_) => "Dialogue",
            GameEvent::Combat(_) => "Combat",
            GameEvent::TriggerObjectEvent(_) => "Trigger Object Event",
        }
    }
    fn index(&self) -> usize {
        match self {
            GameEvent::None => 0,
            GameEvent::Dialogue(_) => 1,
            GameEvent::Combat(_) => 2,
            GameEvent::TriggerObjectEvent(_) => 3,
        }
    }
    fn next(&self) -> Box<dyn UiEnum> {
        Box::new(match self {
            GameEvent::None => GameEvent::Dialogue(Dialogue::new(String::new(), vec![], vec![], 0)),
            GameEvent::Dialogue(_) => GameEvent::Combat(Combat::new(
                CombatPhase::PlayerTurn,
                false,
                false,
                1,
                "#".custom_color(CustomColor::new(255, 255, 255)),
                1,
                1,
                1,
                1,
                false,
            )),
            GameEvent::Combat(_) => GameEvent::TriggerObjectEvent(0),
            GameEvent::TriggerObjectEvent(_) => GameEvent::None,
        })
    }
    fn prev(&self) -> Box<dyn UiEnum> {
        Box::new(match self {
            GameEvent::None => GameEvent::TriggerObjectEvent(0),
            GameEvent::Dialogue(_) => GameEvent::None,
            GameEvent::Combat(_) => {
                GameEvent::Dialogue(Dialogue::new(String::new(), vec![], vec![], 0))
            }
            GameEvent::TriggerObjectEvent(_) => GameEvent::Combat(Combat::new(
                CombatPhase::PlayerTurn,
                false,
                false,
                1,
                "#".custom_color(CustomColor::new(255, 255, 255)),
                1,
                1,
                1,
                1,
                false,
            )),
        })
    }
    fn clone_box(&self) -> Box<dyn UiEnum> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl UiEnum for EventCondition {
    fn name(&self) -> &str {
        match self {
            EventCondition::None => "None",
        }
    }
    fn index(&self) -> usize {
        match self {
            EventCondition::None => 0,
        }
    }
    fn next(&self) -> Box<dyn UiEnum> {
        Box::new(match self {
            EventCondition::None => EventCondition::None,
        })
    }
    fn prev(&self) -> Box<dyn UiEnum> {
        Box::new(match self {
            EventCondition::None => EventCondition::None,
        })
    }
    fn clone_box(&self) -> Box<dyn UiEnum> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub enum ButtonValue {
    Vector2(Vector2),
    Color(Option<Color>),
    Usize(usize),
    OptionUsize(Option<usize>),
    I32(i32),
    Bool(bool),
    Char(char),
    String(String),
    StateChange(EditorState),
    IndexSelection(usize /*Index*/, usize /*Length*/),
    Enum(Box<dyn UiEnum>),
    SubButtons(Vec<Button>, Box<Button> /*Clone of First Button*/),
}
impl Clone for ButtonValue {
    fn clone(&self) -> Self {
        match self {
            ButtonValue::Vector2(v) => ButtonValue::Vector2(v.clone()),
            ButtonValue::Color(c) => ButtonValue::Color(c.clone()),
            ButtonValue::Usize(u) => ButtonValue::Usize(*u),
            ButtonValue::OptionUsize(u) => ButtonValue::OptionUsize(*u),
            ButtonValue::I32(i) => ButtonValue::I32(*i),
            ButtonValue::Bool(b) => ButtonValue::Bool(*b),
            ButtonValue::Char(c) => ButtonValue::Char(*c),
            ButtonValue::String(s) => ButtonValue::String(s.clone()),
            ButtonValue::StateChange(s) => ButtonValue::StateChange(s.clone()),
            ButtonValue::Enum(e) => ButtonValue::Enum(e.clone_box()),
            ButtonValue::IndexSelection(i, l) => ButtonValue::IndexSelection(*i, *l),
            ButtonValue::SubButtons(b, fb) => ButtonValue::SubButtons(b.clone(), fb.clone()),
        }
    }
}
#[derive(Clone)]
pub struct Button {
    pub value: ButtonValue,
    pub name: String,
    pub selected: bool,
    pub value_index: usize,
    pub value_changed: bool,
}
impl Button {
    pub fn new(name: String, value: ButtonValue) -> Self {
        Self {
            value,
            name,
            selected: false,
            value_index: 0,
            value_changed: false,
        }
    }
    pub fn button_backspace(&mut self) {
        match &mut self.value {
            ButtonValue::SubButtons(buttons, _) => {
                self.selected = false;
                if let Some(button) = buttons.get_mut(self.value_index) {
                    button.selected = self.selected;
                }
            }
            ButtonValue::Vector2(_) => {
                self.selected = false;
            }
            _ => {}
        }
    }
    pub fn button_selected(&mut self) {
        match &mut self.value {
            ButtonValue::Bool(value) => {
                *value = !*value;
                self.value_changed = true;
            }
            ButtonValue::Vector2(_)
            | ButtonValue::StateChange(_)
            | ButtonValue::Color(_)
            | ButtonValue::Char(_)
            | ButtonValue::Enum(_)
            | ButtonValue::String(_) => {
                self.selected = !self.selected;
            }
            ButtonValue::SubButtons(buttons, first_button) => {
                if self.value_index == buttons.len() - 1 && buttons.len() > 2 && self.selected {
                    if buttons.len() > 2 {
                        buttons.remove(buttons.len() - 3);
                        self.value_index = buttons.len() - 1;
                        self.value_changed = true;
                        return;
                    }
                    self.selected = false;
                    return;
                } else if self.value_index == buttons.len() - 2 && self.selected {
                    buttons.insert(buttons.len() - 2, *first_button.clone());
                    self.value_index = buttons.len() - 2;
                    self.value_changed = true;
                    return;
                }
                self.selected = !self.selected;
                if let Some(button) = buttons.get_mut(self.value_index) {
                    button.selected = self.selected;
                }
            }
            _ => {}
        }
    }
    pub fn button_right(&mut self) {
        match &mut self.value {
            ButtonValue::Vector2(_) => {
                if self.selected {
                    self.value_index = wrap_add(self.value_index, 1, 1, 0);
                    self.value_changed = true;
                }
            }
            ButtonValue::Color(_) => {
                if self.selected {
                    self.value_index = wrap_add(self.value_index, 1, 2, 0);
                    self.value_changed = true;
                }
            }
            ButtonValue::Usize(value) => {
                *value = wrap_add(*value, 1, usize::MAX, 0);
                self.value_changed = true;
            }
            ButtonValue::OptionUsize(value) => {
                if let Some(val) = value {
                    *val += 1;
                } else {
                    *value = Some(0);
                }
                self.value_changed = true;
            }
            ButtonValue::I32(value) => {
                *value = wrap_add(*value, 1, i32::MAX, i32::MIN);
                self.value_changed = true;
            }
            ButtonValue::Bool(_) => {}
            ButtonValue::Char(_) => {}
            ButtonValue::String(_) => {}
            ButtonValue::StateChange(_) => {}
            ButtonValue::IndexSelection(index, length) => {
                *index = wrap_add(*index, 1, *length - 1, 0);
                self.value_changed = true;
            }
            ButtonValue::Enum(value) => {
                *value = value.next();
                self.value_changed = true;
            }
            ButtonValue::SubButtons(buttons, _) => {
                if self.selected {
                    if let Some(button) = buttons.get_mut(self.value_index) {
                        button.selected = true;
                    }
                    self.value_index = wrap_add(self.value_index, 1, buttons.len() - 1, 0);
                    if let Some(button) = buttons.get_mut(self.value_index) {
                        button.selected = true;
                    }
                }
            }
        }
    }
    pub fn button_left(&mut self) {
        match &mut self.value {
            ButtonValue::Vector2(_) => {
                if self.selected {
                    self.value_index = wrap_remove(self.value_index, 1, 1, 0);
                    self.value_changed = true;
                }
            }
            ButtonValue::Color(_) => {
                if self.selected {
                    self.value_index = wrap_remove(self.value_index, 1, 2, 0);
                    self.value_changed = true;
                }
            }
            ButtonValue::Usize(value) => {
                *value = wrap_remove(*value, 1, usize::MAX, 0);
                self.value_changed = true;
            }
            ButtonValue::OptionUsize(value) => {
                if let Some(val) = value {
                    if *val == 0 {
                        *value = None;
                        return;
                    }
                    *val -= 1;
                    self.value_changed = true;
                }
            }
            ButtonValue::I32(value) => {
                *value = wrap_remove(*value, 1, i32::MAX, i32::MIN);
                self.value_changed = true;
            }
            ButtonValue::Bool(_) => {}
            ButtonValue::Char(_) => {}
            ButtonValue::String(_) => {}
            ButtonValue::StateChange(_) => {}
            ButtonValue::IndexSelection(index, length) => {
                *index = wrap_remove(*index, 1, *length - 1, 0);
                self.value_changed = true;
            }
            ButtonValue::Enum(value) => {
                *value = value.prev();
                self.value_changed = true;
            }
            ButtonValue::SubButtons(buttons, _) => {
                if self.selected {
                    if let Some(button) = buttons.get_mut(self.value_index) {
                        button.selected = true;
                    }
                    self.value_index = wrap_remove(self.value_index, 1, buttons.len() - 1, 0);
                    if let Some(button) = buttons.get_mut(self.value_index) {
                        button.selected = true;
                    }
                }
            }
        }
    }
    pub fn button_up(&mut self) {
        match &mut self.value {
            ButtonValue::Vector2(value) => {
                if self.selected {
                    value[self.value_index] =
                        wrap_add(value[self.value_index], 1, i32::MAX, i32::MIN);
                    self.value_changed = true;
                }
            }
            ButtonValue::Color(value) => {
                if self.selected
                    && let Some(Color::TrueColor { r, g, b }) = value
                {
                    match &self.value_index {
                        0 => *r = wrap_add(*r, 1, u8::MAX, 0),
                        1 => *g = wrap_add(*g, 1, u8::MAX, 0),
                        2 => *b = wrap_add(*b, 1, u8::MAX, 0),
                        _ => {}
                    }
                    self.value_changed = true;
                }
            }
            ButtonValue::Usize(_) => {}
            ButtonValue::OptionUsize(_) => {}
            ButtonValue::I32(_) => {}
            ButtonValue::Bool(_) => {}
            ButtonValue::Char(_) => {}
            ButtonValue::String(_) => {}
            ButtonValue::StateChange(_) => {}
            ButtonValue::IndexSelection(_, _) => {}
            ButtonValue::Enum(_) => {}
            ButtonValue::SubButtons(buttons, _) => {
                if self.selected
                    && let Some(button) = buttons.get_mut(self.value_index)
                {
                    button.button_right();
                    self.value_changed = true;
                }
            }
        }
    }
    pub fn button_down(&mut self) {
        match &mut self.value {
            ButtonValue::Vector2(value) => {
                if self.selected {
                    value[self.value_index] =
                        wrap_remove(value[self.value_index], 1, i32::MAX, i32::MIN);
                    self.value_changed = true;
                }
            }
            ButtonValue::Color(value) => {
                if self.selected
                    && let Some(Color::TrueColor { r, g, b }) = value
                {
                    match &self.value_index {
                        0 => *r = wrap_remove(*r, 1, u8::MAX, u8::MIN),
                        1 => *g = wrap_remove(*g, 1, u8::MAX, u8::MIN),
                        2 => *b = wrap_remove(*b, 1, u8::MAX, u8::MIN),
                        _ => {}
                    }
                    self.value_changed = true;
                }
            }
            ButtonValue::Usize(_) => {}
            ButtonValue::OptionUsize(_) => {}
            ButtonValue::I32(_) => {}
            ButtonValue::Bool(_) => {}
            ButtonValue::Char(_) => {}
            ButtonValue::String(_) => {}
            ButtonValue::StateChange(_) => {}
            ButtonValue::IndexSelection(_, _) => {}
            ButtonValue::Enum(_) => {}
            ButtonValue::SubButtons(buttons, _) => {
                if self.selected
                    && let Some(button) = buttons.get_mut(self.value_index)
                {
                    button.button_left();
                    self.value_changed = true;
                }
            }
        }
    }
    pub fn button_char(&mut self, ch: char) {
        if !self.selected {
            return;
        }
        match &mut self.value {
            ButtonValue::Char(value) => {
                *value = ch;
                self.selected = false;
                self.value_changed = true;
            }
            ButtonValue::String(value) => {
                value.push(ch);
                self.value_changed = true;
            }
            ButtonValue::SubButtons(buttons, _) => {
                if self.selected
                    && let Some(button) = buttons.get_mut(self.value_index)
                {
                    button.button_char(ch);
                    self.value_changed = true;
                }
            }
            _ => {}
        }
    }
}
pub struct Layout {
    pub buttons: HashMap<usize, Button>,
    pub current_button: usize,
}
impl Layout {
    pub fn new() -> Self {
        Self {
            buttons: HashMap::new(),
            current_button: 0,
        }
    }
    pub fn add_button(&mut self, name: String, value: ButtonValue) {
        if self.buttons.contains_key(&self.buttons.len()) {
            return;
        }

        let mut val = value;

        if let ButtonValue::SubButtons(buttons, _) = &mut val {
            buttons.push(Button::new("Add".to_string(), ButtonValue::Bool(false)));
            buttons.push(Button::new("Remove".to_string(), ButtonValue::Bool(false)));
        }
        self.buttons
            .insert(self.buttons.len(), Button::new(name, val));
    }
}
