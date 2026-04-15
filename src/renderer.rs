use colored::*;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "editor"))]
use crate::game::GameState;
#[cfg(feature = "editor")]
use crate::game_object::GameEvent;
#[cfg(not(feature = "editor"))]
use crate::game_object::{COMBAT_SELECTIONS, CombatPhase, GameEvent};
use crate::map::Map;
use crate::vector2::Vector2;

#[cfg(feature = "editor")]
use crate::editor::*;

#[derive(Serialize, Deserialize)]
pub struct ScreenMeasurements {
    // game screen measurements
    pub screen_size: Vector2,
    pub screen_margins: Vector2,
    // Dialogue
    /// distance between the game world and the seperators (|) and the distance between seperators and the dialogue text
    pub dialogue_padding: usize,
    /// distance between the top of the screen and the dialogue text
    pub dialogue_text_padding: usize,
    /// distance between the dialogue text and the selections
    pub dialogue_selection_text_padding: usize,
    /// max number of character to render while in dialogue
    pub dialogue_max_character_count: usize,
    // Combat
    /// distance between the top of the screen and the characters that is in combat
    pub combat_character_padding_y: usize,
    /// distance between the right side of the screen and the first character
    pub combat_character_padding_x: usize,
    /// distance between the characters
    pub combat_characters_distance: usize,
    /// distance between the separators (-) and the characters in the y axis
    pub combat_separator_padding_y: usize,
    /// distance between the separators and the combat selections
    pub combat_selection_separator_padding: usize,
    /// distance between the top of the screen and the characters health indicator
    pub combat_health_padding_y: usize,
}
impl ScreenMeasurements {
    pub fn new(
        screen_size: Vector2,
        screen_margins: Vector2,
        dialogue_padding: usize,
        dialogue_text_padding: usize,
        dialogue_selection_text_padding: usize,
        dialogue_max_character_count: usize,
        combat_character_padding_y: usize,
        combat_character_padding_x: usize,
        combat_characters_distance: usize,
        combat_separator_padding_y: usize,
        combat_selection_separator_padding: usize,
        combat_health_padding_y: usize,
    ) -> Self {
        ScreenMeasurements {
            screen_size,
            screen_margins,
            dialogue_padding,
            dialogue_text_padding,
            dialogue_selection_text_padding,
            dialogue_max_character_count,
            combat_character_padding_y,
            combat_character_padding_x,
            combat_characters_distance,
            combat_separator_padding_y,
            combat_selection_separator_padding,
            combat_health_padding_y,
        }
    }
}

pub struct Renderer {
    pub measurements: ScreenMeasurements,
    #[cfg(not(feature = "editor"))]
    pub combat_message: String,
    #[cfg(feature = "editor")]
    pub editor_message: String,
    line_length: Vec<usize>,
}

impl Renderer {
    pub fn new(measurements: ScreenMeasurements) -> Self {
        let y = measurements.screen_size.y as usize;
        Renderer {
            measurements,
            #[cfg(not(feature = "editor"))]
            combat_message: String::from(""),
            #[cfg(feature = "editor")]
            editor_message: String::from(""),
            line_length: vec![0; y + 1],
        }
    }

    fn pad_line(&mut self, buffer: &mut String, index: usize, raw_len: usize) {
        if index >= self.line_length.len() {
            self.line_length.push(raw_len);
            return;
        }
        let padding_amount = self.line_length[index].saturating_sub(raw_len);
        buffer.push_str(&" ".repeat(padding_amount));
        self.line_length[index] = raw_len;
    }

    #[cfg(feature = "editor")]
    pub fn set_editor_message(&mut self, message: &str) {
        self.editor_message = message.to_string();
    }

    #[cfg(feature = "editor")]
    pub fn render_editor(
        &mut self,
        state: &EditorState,
        camera: &Vector2,
        map: &Map,
        layout: &Layout,
    ) {
        let mut buffer = String::with_capacity(
            (self.measurements.screen_size.x * self.measurements.screen_size.y * 15) as usize,
        );

        match &state {
            EditorState::SelectingFile {
                file_selection,
                file_input,
                file_message,
                recent_projects,
                recent_selection,
            } => {
                let mut len = 0;
                self.pad_line(&mut buffer, 0, len);
                buffer.push_str("\r\n");

                for (i, selection) in FILE_SELECTIONS.iter().enumerate() {
                    if i == *file_selection {
                        buffer.push_str(
                            &selection
                                .custom_color(CustomColor::new(255, 0, 0))
                                .to_string(),
                        );
                    } else {
                        buffer.push_str(selection);
                    }
                    buffer.push_str("  ");
                    len += selection.len() + 2;
                }
                self.pad_line(&mut buffer, 1, len);
                buffer.push_str("\r\n");

                len = 0;

                self.pad_line(&mut buffer, 2, len);
                buffer.push_str("\r\n");

                self.pad_line(&mut buffer, 3, len);
                buffer.push_str("\r\n");

                if FILE_SELECTIONS[*file_selection] == "Recent Projects" {
                    self.pad_line(&mut buffer, 4, len);
                    buffer.push_str("\r\n");

                    buffer.push_str(&file_message);
                    len = file_message.len();
                    self.pad_line(&mut buffer, 5, len);
                    buffer.push_str("\r\n");

                    for (i, path) in recent_projects.iter().enumerate() {
                        len = path.len();
                        if i == *recent_selection {
                            buffer.push_str(
                                &path.custom_color(CustomColor::new(255, 0, 0)).to_string(),
                            );
                        } else {
                            buffer.push_str(&path);
                        }
                        self.pad_line(&mut buffer, 6 + i, len);
                        buffer.push_str("\r\n");
                    }
                } else {
                    buffer.push_str(&format!("location: {}", file_input));
                    len = "location: ".len() + file_input.len();
                    self.pad_line(&mut buffer, 4, len);
                    buffer.push_str("\r\n");

                    buffer.push_str(&file_message);
                    len = file_message.len();
                    self.pad_line(&mut buffer, 5, len);
                    buffer.push_str("\r\n");

                    len = 0;
                    for i in 6..(self.line_length.len()) {
                        self.pad_line(&mut buffer, i, len);
                        buffer.push_str("\r\n");
                    }
                }
            }
            EditorState::Browsing => {
                for y in 0..self.measurements.screen_size.y {
                    self.render_editor_map_line(state, camera, map, &mut buffer, y);
                    let raw_len = self.measurements.screen_size.x as usize;
                    self.pad_line(&mut buffer, y as usize, raw_len);
                    buffer.push_str("\r\n");
                }
            }
            _ => {
                for y in 0..self.measurements.screen_size.y {
                    self.render_editor_map_line(state, camera, map, &mut buffer, y);
                    buffer.push_str("  ");
                    let mut raw_len = self.measurements.screen_size.x as usize + 2;

                    if let Some(button) = layout.buttons.get(&(y as usize)) {
                        self.render_button(
                            &mut buffer,
                            &mut raw_len,
                            button,
                            y as usize,
                            layout.current_button,
                            255,
                            127,
                        );
                    }

                    self.pad_line(&mut buffer, y as usize, raw_len);
                    buffer.push_str("\r\n");
                }
            }
        }

        buffer.push_str(&self.editor_message);
        self.pad_line(
            &mut buffer,
            self.measurements.screen_size.y as usize,
            self.editor_message.chars().count(),
        );

        if self.line_length.len() > self.measurements.screen_size.y as usize {
            for i in self.measurements.screen_size.y as usize..self.line_length.len() {
                buffer.push_str(&" ".repeat(self.line_length[i]));
                buffer.push_str("\r\n");
            }
        }

        print!("{}", buffer);
    }

    #[cfg(feature = "editor")]
    fn render_button(
        &self,
        buffer: &mut String,
        raw_len: &mut usize,
        button: &Button,
        y: usize,
        current_button: usize,
        highlight_color_1: u8,
        highlight_color_2: u8,
    ) {
        *raw_len += button.name.len();

        let colored_name;
        buffer.push_str(if y == current_button {
            colored_name = button
                .name
                .custom_color(CustomColor::new(
                    if button.selected {
                        highlight_color_1
                    } else {
                        highlight_color_2
                    },
                    0,
                    0,
                ))
                .to_string();
            &colored_name
        } else {
            &button.name
        });
        let mut current_str = String::new();
        match &button.value {
            ButtonValue::Bool(value) => {
                current_str.push_str("  ");
                if *value {
                    current_str.push_str("X");
                }
            }
            ButtonValue::Usize(value) => {
                current_str.push_str("  ");
                current_str.push_str(&value.to_string());
            }
            ButtonValue::OptionUsize(value) => {
                current_str.push_str("  ");
                if let Some(u) = value {
                    current_str.push_str(&u.to_string());
                } else {
                    current_str.push_str("None");
                }
            }
            ButtonValue::IndexSelection(index, length) => {
                current_str.push_str(&format!("({}/{})", index + 1, length));
            }
            ButtonValue::Enum(value) => {
                current_str.push_str("  ");
                current_str.push_str(value.name());
            }
            ButtonValue::String(value) => {
                current_str.push_str("  ");
                current_str.push_str(value);
            }
            ButtonValue::SubButtons(buttons, _) => {
                for (i, b) in buttons.iter().enumerate() {
                    buffer.push(' ');
                    *raw_len += 1;
                    if i as i32 <= (buttons.len() as i32) - 3 {
                        if y == current_button {
                            buffer.push_str(
                                &i.to_string()
                                    .custom_color(CustomColor::new(
                                        if i == button.value_index && button.selected {
                                            highlight_color_1
                                        } else {
                                            highlight_color_2
                                        },
                                        0,
                                        0,
                                    ))
                                    .to_string(),
                            );
                        } else {
                            buffer.push_str(&i.to_string());
                        }
                        *raw_len += 1;
                    }
                    if i == button.value_index {
                        self.render_button(
                            buffer,
                            raw_len,
                            b,
                            y as usize,
                            current_button,
                            255,
                            127,
                        );
                    } else {
                        self.render_button(
                            buffer,
                            raw_len,
                            b,
                            y as usize,
                            current_button,
                            127,
                            127,
                        );
                    }
                    buffer.push(' ');
                    *raw_len += 1;
                }
            }
            _ => {}
        }
        *raw_len += current_str.len();
        if y as usize == current_button {
            buffer.push_str(
                &current_str
                    .custom_color(CustomColor::new(
                        if button.selected {
                            highlight_color_1
                        } else {
                            highlight_color_2
                        },
                        0,
                        0,
                    ))
                    .to_string(),
            );
        } else {
            buffer.push_str(&current_str);
        }

        if y as usize == current_button && button.selected {
            match &button.value {
                ButtonValue::Bool(_) => {}
                ButtonValue::Color(value) => {
                    let color = match value {
                        Some(Color::TrueColor { r, g, b }) => CustomColor::new(*r, *g, *b),
                        _ => CustomColor::new(255, 255, 255),
                    };

                    let r_text = format!("  (r:{} ", color.r);
                    let g_text = format!("g:{} ", color.g);
                    let b_text = format!("b:{})", color.b);
                    *raw_len += r_text.len() + g_text.len() + b_text.len();

                    buffer.push_str(
                        &r_text
                            .custom_color(CustomColor::new(
                                255,
                                if button.value_index == 0 { 255 } else { 127 },
                                0,
                            ))
                            .to_string(),
                    );
                    buffer.push_str(
                        &g_text
                            .custom_color(CustomColor::new(
                                255,
                                if button.value_index == 1 { 255 } else { 127 },
                                0,
                            ))
                            .to_string(),
                    );
                    buffer.push_str(
                        &b_text
                            .custom_color(CustomColor::new(
                                255,
                                if button.value_index == 2 { 255 } else { 127 },
                                0,
                            ))
                            .to_string(),
                    );
                }
                ButtonValue::I32(_) => {}
                ButtonValue::StateChange(_) => {}
                ButtonValue::Usize(_) => {}
                ButtonValue::OptionUsize(_) => {}
                ButtonValue::Vector2(value) => {
                    buffer.push_str("  ");
                    let value_text = value.to_colored_string(
                        CustomColor::new(255, if button.value_index == 0 { 255 } else { 127 }, 0),
                        CustomColor::new(255, if button.value_index == 1 { 255 } else { 127 }, 0),
                        CustomColor::new(255, if button.value_index == 2 { 255 } else { 127 }, 0),
                    );
                    buffer.push_str(&value_text);
                    *raw_len += value_text.len() + 2;
                }
                ButtonValue::Char(_) => {}
                ButtonValue::String(_) => {}
                ButtonValue::IndexSelection(_, _) => {}
                ButtonValue::Enum(_) => {}
                ButtonValue::SubButtons(buttons, _) => {}
            }
        }
    }

    #[cfg(feature = "editor")]
    fn render_editor_map_line(
        &self,
        state: &EditorState,
        camera: &Vector2,
        map: &Map,
        buffer: &mut String,
        y: i32,
    ) {
        let cursor_screen_pos = self.measurements.screen_size / 2;

        for x in 0..self.measurements.screen_size.x {
            let current_point = get_point_from_world_to_screen(&camera, &Vector2::new(x, y));

            if map.is_out_of_bounds(current_point) {
                buffer.push_str(" ");
                continue;
            }
            if let Some(id) = map.positions_hashmap.get(&current_point)
                && let Some(object) = map.objects.get(id)
            {
                buffer.push_str(&object.icon.to_string());
            } else if cursor_screen_pos.x == x && cursor_screen_pos.y == y {
                buffer.push_str(&" ".on_white().to_string());
            } else {
                buffer.push_str(&map.ground_icon.to_string());
            }
        }
    }

    #[cfg(not(feature = "editor"))]
    pub fn render(&mut self, map: &Map, camera: &Vector2, state: &GameState) {
        if cfg!(debug_assertions) {
            let Some(cam) = map.objects.get(&map.camera_operator) else {
                return;
            };
            print!("{}\r\n", cam.position);
        }

        let mut buffer = String::with_capacity(
            (self.measurements.screen_size.x * self.measurements.screen_size.y * 15) as usize,
        );

        for y in 0..self.measurements.screen_size.y {
            let mut size: usize = 0;
            match state {
                GameState::Normal => {
                    self.render_map_line(map, camera, &mut buffer, y);
                    size = self.measurements.screen_size.x as usize;
                }
                GameState::Combat => {
                    self.render_combat_line(map, &mut buffer, y);
                }
                GameState::Dialogue => {
                    size = self.measurements.screen_size.x as usize;
                    self.render_map_line(map, camera, &mut buffer, y);
                    self.render_dialogue_line(map, &mut buffer, y, &mut size);
                }
                _ => {}
            }
            self.pad_line(&mut buffer, y as usize, size);
            buffer.push_str("\r\n");
        }

        print!("{}", buffer);
    }

    fn render_map_line(&self, map: &Map, camera: &Vector2, buffer: &mut String, y: i32) {
        for x in 0..self.measurements.screen_size.x {
            let current_point = get_point_from_world_to_screen(camera, &Vector2::new(x, y));
            if map.is_out_of_bounds(current_point) {
                buffer.push_str(" ");
                continue;
            }
            if let Some(id) = map.positions_hashmap.get(&current_point)
                && let Some(object) = map.objects.get(id)
            {
                buffer.push_str(&object.icon.to_string());
            } else {
                buffer.push_str(&map.ground_icon.to_string());
            }
        }
    }

    fn render_dialogue_line(
        &self,
        map: &Map,
        buffer: &mut String,
        y: i32,
        raw_len: &mut usize,
    ) -> Option<()> {
        buffer.push_str(&" ".repeat(self.measurements.dialogue_padding));
        buffer.push_str("|");
        buffer.push_str(&" ".repeat(self.measurements.dialogue_padding));
        *raw_len += self.measurements.dialogue_padding + self.measurements.dialogue_padding + 1;

        let Some(event_id) = map.current_event_id else {
            return None;
        };
        let event = map.event_components.get(&event_id)?;
        let GameEvent::Dialogue(dialogue) = &event.events[event.current_index].event else {
            return None;
        };

        let dialogue_line_index = (y - self.measurements.dialogue_text_padding as i32) as usize;

        let text_chars = dialogue.text.chars().count();
        let text_line_count = (text_chars + self.measurements.dialogue_max_character_count - 1)
            / self.measurements.dialogue_max_character_count;

        if dialogue_line_index < text_line_count {
            let start = dialogue_line_index * self.measurements.dialogue_max_character_count;
            let line_text: String = dialogue
                .text
                .chars()
                .skip(start)
                .take(self.measurements.dialogue_max_character_count)
                .collect();
            buffer.push_str(&line_text);
            *raw_len += line_text.len()
        } else if dialogue_line_index
            >= text_line_count + self.measurements.dialogue_selection_text_padding
        {
            let selection_line_index = dialogue_line_index
                - text_line_count
                - self.measurements.dialogue_selection_text_padding;
            let Some(selection_text) = dialogue.selections.get(selection_line_index) else {
                return None;
            };

            if dialogue.current_selection == selection_line_index {
                buffer.push_str(
                    &selection_text
                        .custom_color(CustomColor::new(255, 0, 0))
                        .to_string(),
                );
            } else {
                buffer.push_str(&selection_text);
            }
            *raw_len += selection_text.len()
        }

        return None;
    }

    #[cfg(not(feature = "editor"))]
    fn render_combat_line(&self, map: &Map, buffer: &mut String, y: i32) {
        let Some(event_id) = map.current_event_id else {
            return;
        };
        let Some(event) = map.event_components.get(&event_id) else {
            return;
        };
        let GameEvent::Combat(combat) = &event.events[event.current_index].event else {
            return;
        };
        let Some(player_obj) = map.objects.get(&map.camera_operator) else {
            return;
        };
        let Some(enemy_obj) = map.objects.get(&event_id) else {
            return;
        };

        if y == self.measurements.combat_health_padding_y as i32 {
            let player_stats = map.stats_components.get(&map.camera_operator);
            let enemy_stats = map.stats_components.get(&event_id);

            let player_hp_text = format!("hp:{}", player_stats.map_or(0, |s| s.health()));
            let enemy_hp_text = format!("hp:{}", enemy_stats.map_or(0, |s| s.health()));

            let to_custom = |color: &Option<Color>| -> CustomColor {
                match color {
                    Some(Color::TrueColor { r, g, b }) => CustomColor::new(*r, *g, *b),
                    _ => CustomColor::new(255, 255, 255),
                }
            };

            let player_color = to_custom(&player_obj.icon.fgcolor);
            let enemy_color = to_custom(&enemy_obj.icon.fgcolor);

            let enemy_col = self.measurements.combat_character_padding_x
                + self.measurements.combat_characters_distance
                + 1;
            let used = enemy_col + enemy_hp_text.len();

            buffer.push_str(&" ".repeat(self.measurements.combat_character_padding_x));
            buffer.push_str(&player_hp_text.custom_color(player_color).to_string());
            buffer.push_str(&" ".repeat(
                enemy_col - self.measurements.combat_character_padding_x - player_hp_text.len(),
            ));
            buffer.push_str(&enemy_hp_text.custom_color(enemy_color).to_string());
            buffer.push_str(&" ".repeat(self.measurements.screen_size.x as usize - used));
            return;
        }

        match &combat.current_phase {
            CombatPhase::EnemyAttack(enemy_attack) => {
                let base = self.measurements.combat_character_padding_x
                    + self.measurements.combat_characters_distance
                    + 1;
                let line_width = self.measurements.screen_size.x as usize;

                if y == self.measurements.combat_character_padding_y as i32 - 2 {
                    buffer.push_str(&" ".repeat(base));
                    buffer.push_str(&enemy_obj.icon.to_string());
                    let used = base + 1;
                    if used < line_width {
                        buffer.push_str(&" ".repeat(line_width - used));
                    }
                    return;
                }

                let player_col = if y
                    == (self.measurements.combat_character_padding_y - 1 + combat.player_row) as i32
                {
                    Some(self.measurements.combat_character_padding_x)
                } else {
                    None
                };

                let mut row_projectiles: Vec<_> = enemy_attack
                    .projectiles
                    .iter()
                    .filter(|p| {
                        y == (self.measurements.combat_character_padding_y - 1 + p.row) as i32
                    })
                    .collect();
                row_projectiles.sort_by(|a, b| b.x.cmp(&a.x));

                let mut last_pos: usize = 0;

                for projectile in &row_projectiles {
                    if projectile.x >= base {
                        continue;
                    }
                    let col = base - projectile.x;
                    if col >= line_width {
                        continue;
                    }

                    if let Some(pcol) = player_col {
                        if pcol >= last_pos && pcol < col {
                            buffer.push_str(&" ".repeat(pcol - last_pos));
                            buffer.push_str(&player_obj.icon.to_string());
                            last_pos = pcol + 1;
                        }
                    }

                    if col >= last_pos {
                        buffer.push_str(&" ".repeat(col - last_pos));
                        buffer.push_str(&combat.projectile_icon.to_string());
                        last_pos = col + 1;
                    }
                }

                if let Some(pcol) = player_col {
                    if pcol >= last_pos && pcol < line_width {
                        buffer.push_str(&" ".repeat(pcol - last_pos));
                        buffer.push_str(&player_obj.icon.to_string());
                        last_pos = pcol + 1;
                    }
                }

                if last_pos < line_width {
                    buffer.push_str(&" ".repeat(line_width - last_pos));
                }
                return;
            }
            _ => {}
        }

        if y == self.measurements.combat_character_padding_y as i32
            && !matches!(combat.current_phase, CombatPhase::EnemyAttack(_))
        {
            buffer.push_str(&" ".repeat(self.measurements.combat_character_padding_x));
            buffer.push_str(&player_obj.icon.to_string());
            buffer.push_str(&" ".repeat(self.measurements.combat_characters_distance));
            buffer.push_str(&enemy_obj.icon.to_string());
        } else if y
            == (self.measurements.combat_character_padding_y
                + self.measurements.combat_separator_padding_y) as i32
        {
            buffer.push_str(&"-".repeat(self.measurements.screen_size.x as usize));
            return;
        } else if y
            == (self.measurements.combat_character_padding_y
                + self.measurements.combat_separator_padding_y
                + self.measurements.combat_selection_separator_padding) as i32
        {
            let mut raw_len: usize = 0;

            match &combat.current_phase {
                CombatPhase::PlayerTurn => {
                    let selections_text = COMBAT_SELECTIONS
                        .iter()
                        .enumerate()
                        .map(|(i, selection)| {
                            let text = if i == combat.current_selection {
                                selection
                                    .custom_color(CustomColor::new(255, 0, 0))
                                    .to_string()
                            } else {
                                selection.to_string()
                            };
                            format!("{}  ", text)
                        })
                        .collect::<String>();

                    raw_len = COMBAT_SELECTIONS.iter().map(|s| s.len() + 2).sum::<usize>()
                        + self.combat_message.len();

                    buffer.push_str(&selections_text);
                }
                _ => {}
            }

            buffer.push_str(
                &self
                    .combat_message
                    .custom_color(CustomColor::new(255, 255, 0))
                    .to_string(),
            );
            buffer.push_str(&" ".repeat(self.measurements.screen_size.x as usize - raw_len));

            return;
        }
        buffer.push_str(&" ".repeat(self.measurements.screen_size.x as usize));
    }
}

fn get_point_from_world_to_screen(game_origin: &Vector2, screen_coordinate: &Vector2) -> Vector2 {
    return game_origin + screen_coordinate;
}
